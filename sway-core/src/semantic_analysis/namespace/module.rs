use crate::{
    decl_engine::DeclRef,
    engine_threading::Engines,
    error::*,
    language::{
        parsed::*,
        ty::{self, TyDecl},
        Visibility,
    },
    semantic_analysis::*,
    transform::to_parsed_lang,
    Ident, Namespace,
};

use super::{
    items::{GlobImport, Items, SymbolMap},
    root::Root,
    trait_map::TraitMap,
    ModuleName, Path, PathBuf,
};

use sway_ast::ItemConst;
use sway_error::handler::Handler;
use sway_error::{error::CompileError, handler::ErrorEmitted};
use sway_parse::{lex, Parser};
use sway_types::{span::Span, Spanned};
use sway_utils::iter_prefixes;

/// A single `Module` within a Sway project.
///
/// A `Module` is most commonly associated with an individual file of Sway code, e.g. a top-level
/// script/predicate/contract file or some library dependency whether introduced via `mod` or the
/// `[dependencies]` table of a `forc` manifest.
///
/// A `Module` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with a map of each of its submodules.
#[derive(Clone, Debug)]
pub struct Module {
    /// Submodules of the current module represented as an ordered map from each submodule's name
    /// to the associated `Module`.
    ///
    /// Submodules are normally introduced in Sway code with the `mod foo;` syntax where `foo` is
    /// some library dependency that we include as a submodule.
    ///
    /// Note that we *require* this map to be ordered to produce deterministic codegen results.
    pub(crate) submodules: im::OrdMap<ModuleName, Module>,
    /// The set of symbols, implementations, synonyms and aliases present within this module.
    items: Items,
    /// Name of the module, package name for root module, module name for other modules.
    /// Module name used is the same as declared in `mod name;`.
    pub name: Option<Ident>,
    /// Whether or not this is a `pub` module
    pub visibility: Visibility,
    /// Empty span at the beginning of the file implementing the module
    pub span: Option<Span>,
    /// Indicates whether the module is external to the current package. External modules are
    /// imported in the `Forc.toml` file.
    pub is_external: bool,
    /// An absolute path from the `root` that represents the module location.
    ///
    /// When this is the root module, this is equal to `[]`. When this is a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub mod_path: PathBuf,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            visibility: Visibility::Private,
            submodules: Default::default(),
            items: Default::default(),
            name: Default::default(),
            span: Default::default(),
            is_external: Default::default(),
            mod_path: Default::default(),
        }
    }
}

impl Module {
    /// `contract_id_value` is injected here via forc-pkg when producing the `dependency_namespace` for a contract which has tests enabled.
    /// This allows us to provide a contract's `CONTRACT_ID` constant to its own unit tests.
    ///
    /// This will eventually be refactored out of `sway-core` in favor of creating temporary package dependencies for providing these
    /// `CONTRACT_ID`-containing modules: https://github.com/FuelLabs/sway/issues/3077
    pub fn default_with_contract_id(
        engines: &Engines,
        name: Option<Ident>,
        contract_id_value: String,
    ) -> Result<Self, vec1::Vec1<CompileError>> {
        let handler = <_>::default();
        Module::default_with_contract_id_inner(&handler, engines, name, contract_id_value).map_err(
            |_| {
                let (errors, warnings) = handler.consume();
                assert!(warnings.is_empty());

                // Invariant: `.value == None` => `!errors.is_empty()`.
                vec1::Vec1::try_from_vec(errors).unwrap()
            },
        )
    }

    fn default_with_contract_id_inner(
        handler: &Handler,
        engines: &Engines,
        ns_name: Option<Ident>,
        contract_id_value: String,
    ) -> Result<Self, ErrorEmitted> {
        // it would be nice to one day maintain a span from the manifest file, but
        // we don't keep that around so we just use the span from the generated const decl instead.
        let mut compiled_constants: SymbolMap = Default::default();
        // this for loop performs a miniature compilation of each const item in the config
        // FIXME(Centril): Stop parsing. Construct AST directly instead!
        // parser config
        let const_item = format!("pub const CONTRACT_ID: b256 = {contract_id_value};");
        let const_item_len = const_item.len();
        let input_arc = std::sync::Arc::from(const_item);
        let token_stream = lex(handler, &input_arc, 0, const_item_len, None).unwrap();
        let mut parser = Parser::new(handler, &token_stream);
        // perform the parse
        let const_item: ItemConst = parser.parse()?;
        let const_item_span = const_item.span();

        // perform the conversions from parser code to parse tree types
        let name = const_item.name.clone();
        let attributes = Default::default();
        // convert to const decl
        let const_decl = to_parsed_lang::item_const_to_constant_declaration(
            &mut to_parsed_lang::Context::default(),
            handler,
            engines,
            const_item,
            attributes,
            true,
        )?;

        // Temporarily disallow non-literals. See https://github.com/FuelLabs/sway/issues/2647.
        let has_literal = match &const_decl.value {
            Some(value) => {
                matches!(value.kind, ExpressionKind::Literal(_))
            }
            None => false,
        };

        if !has_literal {
            return Err(handler.emit_err(CompileError::ContractIdValueNotALiteral {
                span: const_item_span,
            }));
        }

        let ast_node = AstNode {
            content: AstNodeContent::Declaration(Declaration::ConstantDeclaration(const_decl)),
            span: const_item_span.clone(),
        };
        let mut ns = Namespace::init_root(Default::default());
        // This is pretty hacky but that's okay because of this code is being removed pretty soon
        ns.root.module.name = ns_name;
        ns.root.module.is_external = true;
        ns.root.module.visibility = Visibility::Public;
        let type_check_ctx = TypeCheckContext::from_root(&mut ns, engines);
        let typed_node =
            ty::TyAstNode::type_check(type_check_ctx, ast_node).unwrap(&mut vec![], &mut vec![]);
        // get the decl out of the typed node:
        // we know as an invariant this must be a const decl, as we hardcoded a const decl in
        // the above `format!`.  if it isn't we report an
        // error that only constant items are alowed, defensive programming etc...
        let typed_decl = match typed_node.content {
            ty::TyAstNodeContent::Declaration(decl) => decl,
            _ => {
                return Err(
                    handler.emit_err(CompileError::ContractIdConstantNotAConstDecl {
                        span: const_item_span,
                    }),
                );
            }
        };
        compiled_constants.insert(name, typed_decl);

        let mut ret = Self::default();
        ret.items.symbols = compiled_constants;
        Ok(ret)
    }

    /// Immutable access to this module's submodules.
    pub fn submodules(&self) -> &im::OrdMap<ModuleName, Module> {
        &self.submodules
    }

    /// Insert a submodule into this `Module`.
    pub fn insert_submodule(&mut self, name: String, submodule: Module) {
        self.submodules.insert(name, submodule);
    }

    /// Lookup the submodule at the given path.
    pub fn submodule(&self, path: &Path) -> Option<&Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Unique access to the submodule at the given path.
    pub fn submodule_mut(&mut self, path: &Path) -> Option<&mut Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get_mut(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn check_submodule(&self, path: &[Ident]) -> CompileResult<&Module> {
        match self.submodule(path) {
            None => err(vec![], vec![module_not_found(path)]),
            Some(module) => ok(module, vec![], vec![]),
        }
    }

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn star_import(
        &mut self,
        src: &Path,
        dst: &Path,
        engines: &Engines,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        check!(
            self.check_module_privacy(src, dst),
            return err(warnings, errors),
            warnings,
            errors
        );

        let decl_engine = engines.de();

        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );

        let implemented_traits = src_ns.implemented_traits.clone();
        let mut symbols_and_decls = vec![];
        for (symbol, decl) in src_ns.symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(decl_engine).is_public() {
                symbols_and_decls.push((symbol.clone(), decl.clone()));
            }
        }

        let dst_ns = &mut self[dst];
        dst_ns
            .implemented_traits
            .extend(implemented_traits, engines);
        for symbol_and_decl in symbols_and_decls {
            dst_ns.use_synonyms.insert(
                symbol_and_decl.0,
                (
                    src.to_vec(),
                    GlobImport::Yes,
                    symbol_and_decl.1,
                    is_src_absolute,
                ),
            );
        }

        ok((), warnings, errors)
    }

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be relative to `self`.
    pub fn star_import_with_reexports(
        &mut self,
        src: &Path,
        dst: &Path,
        engines: &Engines,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        check!(
            self.check_module_privacy(src, dst),
            return err(warnings, errors),
            warnings,
            errors
        );

        let decl_engine = engines.de();

        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );

        let implemented_traits = src_ns.implemented_traits.clone();
        let use_synonyms = src_ns.use_synonyms.clone();
        let mut symbols_and_decls = src_ns
            .use_synonyms
            .iter()
            .map(|(symbol, (_, _, decl, _))| (symbol.clone(), decl.clone()))
            .collect::<Vec<_>>();
        for (symbol, decl) in src_ns.symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(decl_engine).is_public() {
                symbols_and_decls.push((symbol.clone(), decl.clone()));
            }
        }

        let mut symbols_paths_and_decls = vec![];
        for (symbol, (mod_path, _, decl, _)) in use_synonyms {
            let mut is_external = false;
            let submodule = src_ns.submodule(&[mod_path[0].clone()]);
            if let Some(submodule) = submodule {
                is_external = submodule.is_external
            };

            let mut path = src[..1].to_vec();
            if is_external {
                path = mod_path;
            } else {
                path.extend(mod_path);
            }

            symbols_paths_and_decls.push((symbol, path, decl));
        }

        let dst_ns = &mut self[dst];
        dst_ns
            .implemented_traits
            .extend(implemented_traits, engines);

        let mut try_add = |symbol, path, decl: ty::TyDecl| {
            dst_ns
                .use_synonyms
                .insert(symbol, (path, GlobImport::Yes, decl, is_src_absolute));
        };

        for (symbol, decl) in symbols_and_decls {
            try_add(symbol, src.to_vec(), decl);
        }

        for (symbol, path, decl) in symbols_paths_and_decls {
            try_add(symbol, path, decl);
        }

        ok((), warnings, errors)
    }

    /// Pull a single item from a `src` module and import it into the `dst` module.
    ///
    /// The item we want to import is basically the last item in path because this is a `self`
    /// import.
    pub(crate) fn self_import(
        &mut self,
        engines: &Engines,
        src: &Path,
        dst: &Path,
        alias: Option<Ident>,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(engines, src, last_item, dst, alias, is_src_absolute)
    }

    /// Pull a single `item` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn item_import(
        &mut self,
        engines: &Engines,
        src: &Path,
        item: &Ident,
        dst: &Path,
        alias: Option<Ident>,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        check!(
            self.check_module_privacy(src, dst),
            return err(warnings, errors),
            warnings,
            errors
        );

        let decl_engine = engines.de();

        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = TraitMap::default();
        match src_ns.symbols.get(item).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: item.clone(),
                        span: item.span(),
                    });
                }

                let type_id = decl.return_type(engines).value;
                //  if this is an enum or struct or function, import its implementations
                if let Some(type_id) = type_id {
                    impls_to_insert.extend(
                        src_ns
                            .implemented_traits
                            .filter_by_type_item_import(type_id, engines),
                        engines,
                    );
                }
                // if this is a trait, import its implementations
                let decl_span = decl.span();
                if let TyDecl::TraitDecl(_) = &decl {
                    // TODO: we only import local impls from the source namespace
                    // this is okay for now but we'll need to device some mechanism to collect all available trait impls
                    impls_to_insert.extend(
                        src_ns
                            .implemented_traits
                            .filter_by_trait_decl_span(decl_span),
                        engines,
                    );
                }
                // no matter what, import it this way though.
                let dst_ns = &mut self[dst];
                let add_synonym = |name| {
                    if let Some((_, GlobImport::No, _, _)) = dst_ns.use_synonyms.get(name) {
                        errors.push(CompileError::ShadowsOtherSymbol { name: name.clone() });
                    }
                    dst_ns.use_synonyms.insert(
                        name.clone(),
                        (src.to_vec(), GlobImport::No, decl, is_src_absolute),
                    );
                };
                match alias {
                    Some(alias) => {
                        add_synonym(&alias);
                        dst_ns
                            .use_aliases
                            .insert(alias.as_str().to_string(), item.clone());
                    }
                    None => add_synonym(item),
                };
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: item.clone(),
                    span: item.span(),
                });
                return err(warnings, errors);
            }
        };

        let dst_ns = &mut self[dst];
        dst_ns.implemented_traits.extend(impls_to_insert, engines);

        ok((), warnings, errors)
    }

    /// Pull a single variant `variant` from the enum `enum_name` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be relative to `self`.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn variant_import(
        &mut self,
        engines: &Engines,
        src: &Path,
        enum_name: &Ident,
        variant_name: &Ident,
        dst: &Path,
        alias: Option<Ident>,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        check!(
            self.check_module_privacy(src, dst),
            return err(warnings, errors),
            warnings,
            errors
        );

        let decl_engine = engines.de();

        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        match src_ns.symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl {
                    decl_id,
                    subst_list: _,
                    ..
                }) = decl
                {
                    let enum_decl = decl_engine.get_enum(&decl_id);
                    let enum_ref = DeclRef::new(
                        enum_decl.call_path.suffix.clone(),
                        decl_id,
                        enum_decl.span(),
                    );

                    if let Some(variant_decl) =
                        enum_decl.variants.iter().find(|v| v.name == *variant_name)
                    {
                        // import it this way.
                        let dst_ns = &mut self[dst];
                        let mut add_synonym = |name| {
                            if let Some((_, GlobImport::No, _, _)) = dst_ns.use_synonyms.get(name) {
                                errors
                                    .push(CompileError::ShadowsOtherSymbol { name: name.clone() });
                            }
                            dst_ns.use_synonyms.insert(
                                name.clone(),
                                (
                                    src.to_vec(),
                                    GlobImport::No,
                                    TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                        enum_ref: enum_ref.clone(),
                                        variant_name: variant_name.clone(),
                                        variant_decl_span: variant_decl.span.clone(),
                                    }),
                                    is_src_absolute,
                                ),
                            );
                        };
                        match alias {
                            Some(alias) => {
                                add_synonym(&alias);
                                dst_ns
                                    .use_aliases
                                    .insert(alias.as_str().to_string(), variant_name.clone());
                            }
                            None => add_synonym(variant_name),
                        };
                    } else {
                        errors.push(CompileError::SymbolNotFound {
                            name: variant_name.clone(),
                            span: variant_name.span(),
                        });
                        return err(warnings, errors);
                    }
                } else {
                    errors.push(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    ));
                    return err(warnings, errors);
                }
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                });
                return err(warnings, errors);
            }
        };

        ok((), warnings, errors)
    }

    /// Pull all variants from the enum `enum_name` from the given `src` module and import them all into the `dst` module.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn variant_star_import(
        &mut self,
        src: &Path,
        dst: &Path,
        engines: &Engines,
        enum_name: &Ident,
        is_src_absolute: bool,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        check!(
            self.check_module_privacy(src, dst),
            return err(warnings, errors),
            warnings,
            errors
        );

        let decl_engine = engines.de();

        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        match src_ns.symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl {
                    decl_id,
                    subst_list: _,
                    ..
                }) = decl
                {
                    let enum_decl = decl_engine.get_enum(&decl_id);
                    let enum_ref = DeclRef::new(
                        enum_decl.call_path.suffix.clone(),
                        decl_id,
                        enum_decl.span(),
                    );

                    for variant_decl in enum_decl.variants {
                        let variant_name = variant_decl.name;

                        // import it this way.
                        let dst_ns = &mut self[dst];
                        dst_ns.use_synonyms.insert(
                            variant_name.clone(),
                            (
                                src.to_vec(),
                                GlobImport::Yes,
                                TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                    enum_ref: enum_ref.clone(),
                                    variant_name,
                                    variant_decl_span: variant_decl.span.clone(),
                                }),
                                is_src_absolute,
                            ),
                        );
                    }
                } else {
                    errors.push(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    ));
                    return err(warnings, errors);
                }
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                });
                return err(warnings, errors);
            }
        };

        ok((), warnings, errors)
    }

    fn check_module_privacy(&self, src: &Path, dst: &Path) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // you are always allowed to access your ancestor's symbols
        if !is_ancestor(src, dst) {
            // we don't check the first prefix because direct children are always accessible
            for prefix in iter_prefixes(src).skip(1) {
                let module = check!(
                    self.check_submodule(prefix),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if module.visibility.is_private() {
                    let prefix_last = prefix[prefix.len() - 1].clone();
                    errors.push(CompileError::ImportPrivateModule {
                        span: prefix_last.span(),
                        name: prefix_last,
                    });
                }
            }
        }
        ok((), warnings, errors)
    }
}

impl std::ops::Deref for Module {
    type Target = Items;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl std::ops::DerefMut for Module {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<'a> std::ops::Index<&'a Path> for Module {
    type Output = Module;
    fn index(&self, path: &'a Path) -> &Self::Output {
        self.submodule(path)
            .unwrap_or_else(|| panic!("no module for the given path {path:?}"))
    }
}

impl<'a> std::ops::IndexMut<&'a Path> for Module {
    fn index_mut(&mut self, path: &'a Path) -> &mut Self::Output {
        self.submodule_mut(path)
            .unwrap_or_else(|| panic!("no module for the given path {path:?}"))
    }
}

impl From<Root> for Module {
    fn from(root: Root) -> Self {
        root.module
    }
}

fn module_not_found(path: &[Ident]) -> CompileError {
    CompileError::ModuleNotFound {
        span: path.iter().fold(path[0].span(), |acc, this_one| {
            if acc.source_id() == this_one.span().source_id() {
                Span::join(acc, this_one.span())
            } else {
                acc
            }
        }),
        name: path
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}

fn is_ancestor(src: &Path, dst: &Path) -> bool {
    dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
}
