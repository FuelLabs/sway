use crate::{
    declaration_engine::declaration_engine::DeclarationEngine,
    error::*,
    parse_tree::{Declaration, Visibility},
    semantic_analysis::{
        ast_node::{TypedAstNode, TypedAstNodeContent, TypedVariableDeclaration},
        declaration::VariableMutability,
        TypeCheckContext,
    },
    AstNode, AstNodeContent, CompileResult, Ident, Namespace, TypedDeclaration,
};

use super::{
    items::{Items, SymbolMap},
    root::Root,
    ModuleName, Path,
};

use std::collections::BTreeMap;
use sway_ast::ItemConst;
use sway_parse::{handler::Handler, lex, Parser};
use sway_types::{span::Span, ConfigTimeConstant, Spanned};

/// A single `Module` within a Sway project.
///
/// A `Module` is most commonly associated with an individual file of Sway code, e.g. a top-level
/// script/predicate/contract file or some library dependency whether introduced via `dep` or the
/// `[dependencies]` table of a `forc` manifest.
///
/// A `Module` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with a map of each of its submodules.
#[derive(Clone, Debug, Default)]
pub struct Module {
    /// Submodules of the current module represented as an ordered map from each submodule's name
    /// to the associated `Module`.
    ///
    /// Submodules are normally introduced in Sway code with the `dep foo;` syntax where `foo` is
    /// some library dependency that we include as a submodule.
    ///
    /// Note that we *require* this map to be ordered to produce deterministic codegen results.
    pub(crate) submodules: im::OrdMap<ModuleName, Module>,
    /// The set of symbols, implementations, synonyms and aliases present within this module.
    items: Items,
}

impl Module {
    pub fn default_with_constants(
        constants: BTreeMap<String, ConfigTimeConstant>,
    ) -> Result<Self, vec1::Vec1<CompileError>> {
        let res = Module::default_with_constants_inner(constants);
        match res.value {
            Some(x) => Ok(x),
            None => {
                let mut errs = res.errors;
                // it is an invariant that if `.value` is `None` then there's at least one
                // error
                assert!(!errs.is_empty());
                let first_err = errs.pop().unwrap();
                let mut errs_1 = vec1::vec1![first_err];
                errs_1.append(&mut errs);
                Err(errs_1)
            }
        }
    }

    fn default_with_constants_inner(
        constants: BTreeMap<String, ConfigTimeConstant>,
    ) -> CompileResult<Self> {
        // it would be nice to one day maintain a span from the manifest file, but
        // we don't keep that around so we just use the span from the generated const decl instead.
        let mut compiled_constants: SymbolMap = Default::default();
        let mut ec: crate::convert_parse_tree::ErrorContext = Default::default();
        let ec = &mut ec;
        let mut warnings = vec![];
        let mut errors = vec![];
        // this for loop performs a miniature compilation of each const item in the config
        for (name, ConfigTimeConstant { r#type, value }) in constants.into_iter() {
            // parser config
            let const_item = format!("const {name}: {type} = {value};");
            let const_item_len = const_item.len();
            let input_arc = std::sync::Arc::from(const_item);
            let token_stream = lex(&input_arc, 0, const_item_len, None).unwrap();
            let handler = Handler::default();
            let mut parser = Parser::new(&token_stream, &handler);
            // perform the parse
            let const_item: ItemConst = match parser.parse() {
                Ok(o) => o,
                Err(_emit_signal) => {
                    // if an error was emitted, grab errors from the error context
                    errors.append(&mut ec.errors.clone());
                    warnings.append(&mut ec.warnings.clone());

                    return err(warnings, errors);
                }
            };
            let const_item_span = const_item.span().clone();

            // perform the conversions from parser code to parse tree types
            let name = const_item.name.clone();
            // convert to const decl
            let const_decl =
                match crate::convert_parse_tree::item_const_to_constant_declaration(ec, const_item)
                {
                    Ok(o) => o,
                    Err(_emit_signal) => {
                        // if an error was emitted, grab errors from the error context
                        errors.append(&mut ec.errors.clone());
                        warnings.append(&mut ec.warnings.clone());

                        return err(warnings, errors);
                    }
                };
            let ast_node = AstNode {
                content: AstNodeContent::Declaration(Declaration::ConstantDeclaration(const_decl)),
                span: const_item_span.clone(),
            };
            let mut declaration_engine = DeclarationEngine::new();
            let mut ns = Namespace::init_root(Default::default());
            let type_check_ctx = TypeCheckContext::from_root(&mut ns, &mut declaration_engine);
            let typed_node =
                TypedAstNode::type_check(type_check_ctx, ast_node).unwrap(&mut vec![], &mut vec![]);
            // get the decl out of the typed node:
            // we know as an invariant this must be a const decl, as we hardcoded a const decl in
            // the above `format!`.  if it isn't we report an
            // error that only constant items are alowed, defensive programming etc...
            let typed_decl = match typed_node.content {
                TypedAstNodeContent::Declaration(decl) => decl,
                _ => {
                    errors.push(CompileError::ConfigTimeConstantNotAConstDecl {
                        span: const_item_span,
                    });
                    return err(warnings, errors);
                }
            };
            compiled_constants.insert(name, typed_decl);
        }
        ok(
            Self {
                items: Items {
                    symbols: compiled_constants,
                    ..Default::default()
                },
                ..Default::default()
            },
            warnings,
            errors,
        )
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
    pub(crate) fn star_import(&mut self, src: &Path, dst: &Path) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        let implemented_traits = src_ns.implemented_traits.clone();
        let symbols = src_ns
            .symbols
            .iter()
            .filter_map(|(symbol, decl)| {
                if decl.visibility() == Visibility::Public {
                    Some(symbol.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let dst_ns = &mut self[dst];
        dst_ns.implemented_traits.extend(implemented_traits);
        for symbol in symbols {
            if dst_ns.use_synonyms.contains_key(&symbol) {
                errors.push(CompileError::StarImportShadowsOtherSymbol {
                    name: symbol.clone(),
                });
            }
            dst_ns.use_synonyms.insert(symbol, src.to_vec());
        }
        ok((), warnings, errors)
    }

    /// Pull a single item from a `src` module and import it into the `dst` module.
    ///
    /// The item we want to import is basically the last item in path because this is a `self`
    /// import.
    pub(crate) fn self_import(
        &mut self,
        src: &Path,
        dst: &Path,
        alias: Option<Ident>,
        de: &DeclarationEngine,
    ) -> CompileResult<()> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(src, last_item, dst, alias, de)
    }

    /// Pull a single `item` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn item_import(
        &mut self,
        src: &Path,
        item: &Ident,
        dst: &Path,
        alias: Option<Ident>,
        de: &DeclarationEngine,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = vec![];
        match src_ns.symbols.get(item).cloned() {
            Some(decl) => {
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol { name: item.clone() });
                }
                // if this is a const, insert it into the local namespace directly
                if let TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    mutability: VariableMutability::ExportedConst,
                    ref name,
                    ..
                }) = decl
                {
                    self[dst].insert_symbol(alias.unwrap_or_else(|| name.clone()), decl.clone());
                    return ok((), warnings, errors);
                }
                let a = decl.return_type().value;
                //  if this is an enum or struct, import its implementations
                let mut res = match a {
                    Some(a) => src_ns.implemented_traits.get_call_path_and_type_info(a, de),
                    None => vec![],
                };
                impls_to_insert.append(&mut res);
                // no matter what, import it this way though.
                let dst_ns = &mut self[dst];
                match alias {
                    Some(alias) => {
                        if dst_ns.use_synonyms.contains_key(&alias) {
                            errors.push(CompileError::ShadowsOtherSymbol {
                                name: alias.clone(),
                            });
                        }
                        dst_ns.use_synonyms.insert(alias.clone(), src.to_vec());
                        dst_ns
                            .use_aliases
                            .insert(alias.as_str().to_string(), item.clone());
                    }
                    None => {
                        if dst_ns.use_synonyms.contains_key(item) {
                            errors.push(CompileError::ShadowsOtherSymbol { name: item.clone() });
                        }
                        dst_ns.use_synonyms.insert(item.clone(), src.to_vec());
                    }
                };
            }
            None => {
                errors.push(CompileError::SymbolNotFound { name: item.clone() });
                return err(warnings, errors);
            }
        };

        let dst_ns = &mut self[dst];
        impls_to_insert
            .into_iter()
            .for_each(|((call_path, type_info), methods)| {
                dst_ns
                    .implemented_traits
                    .insert(call_path, type_info, methods);
            });

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
            .unwrap_or_else(|| panic!("no module for the given path {:?}", path))
    }
}

impl<'a> std::ops::IndexMut<&'a Path> for Module {
    fn index_mut(&mut self, path: &'a Path) -> &mut Self::Output {
        self.submodule_mut(path)
            .unwrap_or_else(|| panic!("no module for the given path {:?}", path))
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
            if acc.path() == this_one.span().path() {
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
