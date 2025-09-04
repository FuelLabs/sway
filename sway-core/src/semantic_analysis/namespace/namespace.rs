use crate::{
    decl_engine::DeclRef,
    language::{parsed::*, Visibility},
    ty::{self, TyDecl},
    Engines, Ident,
};

use super::{
    module::Module, package::Package, trait_map::TraitMap, ModuleName, ModulePath, ModulePathBuf,
    ResolvedDeclaration,
};

use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{
    constants::{CONTRACT_ID, PRELUDE, STD},
    span::Span,
    Spanned,
};
use sway_utils::iter_prefixes;

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug)]
pub struct Namespace {
    /// The current package, containing all the bindings found so far during compilation.
    ///
    /// The `Package` object should be supplied to `new` in order to be properly initialized. Note
    /// also the existence of `contract_helpers::package_with_contract_id`.
    pub(crate) current_package: Package,
    /// An absolute path to the current module within the current package.
    ///
    /// The path of the root module in a package is `[package_name]`. If a module `X` is a submodule
    /// of module `Y` which is a submodule of the root module in the package `P`, then the path is
    /// `[P, Y, X]`.
    pub(crate) current_mod_path: ModulePathBuf,
}

impl Namespace {
    /// Initialize the namespace
    /// See also `contract_helpers::package_with_contract_id`.
    ///
    /// If `import_std_prelude_into_root` is true then std::prelude::* will be imported into the
    /// root module, provided std is available in the external modules.
    pub fn new(
        handler: &Handler,
        engines: &Engines,
        package: Package,
        import_std_prelude_into_root: bool,
    ) -> Result<Self, ErrorEmitted> {
        let name = package.name().clone();
        let mut res = Self {
            current_package: package,
            current_mod_path: vec![name],
        };

        if import_std_prelude_into_root {
            res.import_implicits(handler, engines)?;
        }
        Ok(res)
    }

    pub fn current_package(self) -> Package {
        self.current_package
    }

    pub fn current_package_ref(&self) -> &Package {
        &self.current_package
    }

    fn module_in_current_package(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(self.current_package.check_path_is_in_package(mod_path));
        self.current_package.module_from_absolute_path(mod_path)
    }

    pub fn current_module(&self) -> &Module {
        self.module_in_current_package(&self.current_mod_path)
            .unwrap_or_else(|| {
                panic!(
                    "Could not retrieve submodule for mod_path: {:?}",
                    self.current_mod_path
                );
            })
    }

    pub fn current_module_mut(&mut self) -> &mut Module {
        let package_relative_path = Package::package_relative_path(&self.current_mod_path);
        self.current_package
            .root_module_mut()
            .submodule_mut(package_relative_path)
            .unwrap_or_else(|| {
                panic!("Could not retrieve submodule for mod_path: {package_relative_path:?}");
            })
    }

    pub(crate) fn current_module_has_submodule(&self, submod_name: &Ident) -> bool {
        self.current_module()
            .submodule(&[submod_name.clone()])
            .is_some()
    }

    pub fn current_package_name(&self) -> &Ident {
        self.current_package.name()
    }

    /// A reference to the path of the module currently being processed.
    pub fn current_mod_path(&self) -> &ModulePathBuf {
        &self.current_mod_path
    }

    /// Prepends the module path into the prefixes.
    pub fn prepend_module_path<'a>(
        &'a self,
        prefixes: impl IntoIterator<Item = &'a Ident>,
    ) -> ModulePathBuf {
        self.current_mod_path
            .iter()
            .chain(prefixes)
            .cloned()
            .collect()
    }

    /// Convert a parsed path to a full path.
    pub fn parsed_path_to_full_path(
        &self,
        _engines: &Engines,
        parsed_path: &ModulePathBuf,
        is_relative_to_package_root: bool,
    ) -> ModulePathBuf {
        if is_relative_to_package_root {
            // Path is relative to the root module in the current package. Prepend the package name
            let mut path = vec![self.current_package_name().clone()];
            for ident in parsed_path.iter() {
                path.push(ident.clone())
            }
            path
        } else if self.current_module_has_submodule(&parsed_path[0]) {
            // The first identifier is a submodule of the current module
            // The path is therefore assumed to be relative to the current module, so prepend the current module path.
            self.prepend_module_path(parsed_path)
        } else if self.module_is_external(parsed_path) {
            // The path refers to an external module, so the path is already a full path.
            parsed_path.to_vec()
        } else {
            // The first identifier is neither a submodule nor an external package. It must
            // therefore refer to a binding in the local environment
            self.prepend_module_path(parsed_path)
        }
    }

    pub fn current_package_root_module(&self) -> &Module {
        self.current_package.root_module()
    }

    pub fn external_packages(
        &self,
    ) -> &im::HashMap<ModuleName, Package, BuildHasherDefault<FxHasher>> {
        &self.current_package.external_packages
    }

    pub(crate) fn get_external_package(&self, package_name: &str) -> Option<&Package> {
        self.current_package.external_packages.get(package_name)
    }

    pub(super) fn exists_as_external(&self, package_name: &str) -> bool {
        self.get_external_package(package_name).is_some()
    }

    pub fn module_from_absolute_path(&self, path: &[Ident]) -> Option<&Module> {
        if path.is_empty() {
            None
        } else {
            self.current_package.module_from_absolute_path(path)
        }
    }

    // Like module_from_absolute_path, but throws an error if the module is not found
    pub fn require_module_from_absolute_path(
        &self,
        handler: &Handler,
        path: &[Ident],
    ) -> Result<&Module, ErrorEmitted> {
        if path.is_empty() {
            return Err(handler.emit_err(CompileError::Internal(
                "Found empty absolute mod path",
                Span::dummy(),
            )));
        }
        let is_in_current_package = self.current_package.check_path_is_in_package(path);
        match self.module_from_absolute_path(path) {
            Some(module) => Ok(module),
            None => Err(handler.emit_err(crate::namespace::module::module_not_found(
                path,
                is_in_current_package,
            ))),
        }
    }

    /// Returns true if the current module being checked is a direct or indirect submodule of
    /// the module given by the `absolute_module_path`.
    ///
    /// The current module being checked is determined by `current_mod_path`.
    ///
    /// E.g., the mod_path `[fist, second, third]` of the root `foo` is a submodule of the module
    /// `[foo, first]`.
    ///
    /// If the current module being checked is the same as the module given by the
    /// `absolute_module_path`, the `true_if_same` is returned.
    pub(crate) fn module_is_submodule_of(
        &self,
        absolute_module_path: &ModulePath,
        true_if_same: bool,
    ) -> bool {
        if self.current_mod_path.len() < absolute_module_path.len() {
            return false;
        }

        let is_submodule = absolute_module_path
            .iter()
            .zip(self.current_mod_path.iter())
            .all(|(left, right)| left == right);

        if is_submodule {
            if self.current_mod_path.len() == absolute_module_path.len() {
                true_if_same
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Returns true if the module given by the `absolute_module_path` is external
    /// to the current package. External modules are imported in the `Forc.toml` file.
    pub(crate) fn module_is_external(&self, absolute_module_path: &ModulePath) -> bool {
        assert!(!absolute_module_path.is_empty(), "Absolute module path must have at least one element, because it always contains the package name.");

        self.current_package_name() != &absolute_module_path[0]
    }

    pub fn package_exists(&self, name: &Ident) -> bool {
        self.module_from_absolute_path(&[name.clone()]).is_some()
    }

    pub(crate) fn module_has_binding(
        &self,
        engines: &Engines,
        mod_path: &ModulePathBuf,
        symbol: &Ident,
    ) -> bool {
        let dummy_handler = Handler::default();
        if let Some(module) = self.module_from_absolute_path(mod_path) {
            module
                .resolve_symbol(&dummy_handler, engines, symbol)
                .is_ok()
        } else {
            false
        }
    }

    // Import std::prelude::* and ::CONTRACT_ID as appropriate into the current module
    fn import_implicits(
        &mut self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        // Import preludes
        let package_name = self.current_package_name().to_string();
        let prelude_ident = Ident::new_no_span(PRELUDE.to_string());

        if package_name == STD {
            // Do nothing
        } else {
            // Import std::prelude::*
            let std_string = STD.to_string();
            // Only import std::prelude::* if std exists as a dependency
            if self.exists_as_external(&std_string) {
                self.prelude_import(
                    handler,
                    engines,
                    &[Ident::new_no_span(std_string), prelude_ident],
                )?
            }
        }

        // Import contract id. CONTRACT_ID is declared in the root module, so only import it into
        // non-root modules
        if self.current_package.is_contract_package() && self.current_mod_path.len() > 1 {
            // import ::CONTRACT_ID
            self.item_import_to_current_module(
                handler,
                engines,
                &[Ident::new_no_span(package_name)],
                &Ident::new_no_span(CONTRACT_ID.to_string()),
                None,
                Visibility::Private,
            )?
        }

        Ok(())
    }

    pub(crate) fn enter_submodule(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        check_implicits: bool,
    ) -> Result<(), ErrorEmitted> {
        let mut import_implicits = false;

        // Ensure the new module exists and is initialized properly
        if !self
            .current_module()
            .submodules()
            .contains_key(&mod_name.to_string())
            && check_implicits
        {
            // Entering a new module. Add a new one.
            self.current_module_mut()
                .add_new_submodule(&mod_name, visibility, Some(module_span));
            import_implicits = true;
        }

        // Update self to point to the new module
        self.current_mod_path.push(mod_name.clone());

        // Import implicits into the newly created module.
        if import_implicits {
            self.import_implicits(handler, engines)?;
        }

        Ok(())
    }

    /// Pushes a new submodule to the namespace's module hierarchy.
    pub fn push_submodule(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        check_implicits: bool,
    ) -> Result<(), ErrorEmitted> {
        match self.enter_submodule(
            handler,
            engines,
            mod_name,
            visibility,
            module_span,
            check_implicits,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Pops the current submodule from the namespace's module hierarchy.
    pub fn pop_submodule(&mut self) {
        self.current_mod_path.pop();
    }

    ////// IMPORT //////

    /// Given a path to a prelude in the standard library, create synonyms to every symbol in that
    /// prelude to the current module.
    ///
    /// This is used when a new module is created in order to pupulate the module with implicit
    /// imports from the standard library preludes.
    ///
    /// `src` is assumed to be absolute.
    fn prelude_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        let src_mod = self.require_module_from_absolute_path(handler, src)?;

        let mut imports = vec![];

        // A prelude should not declare its own items
        assert!(src_mod.root_items().symbols.is_empty());

        // Collect those item-imported items that the source module reexports
        let mut symbols = src_mod
            .root_items()
            .use_item_synonyms
            .keys()
            .clone()
            .collect::<Vec<_>>();
        symbols.sort();
        for symbol in symbols {
            let (_, path, decl, src_visibility) = &src_mod.root_items().use_item_synonyms[symbol];
            // Preludes reexport all their imports
            assert!(matches!(src_visibility, Visibility::Public));
            imports.push((symbol.clone(), decl.clone(), path.clone()))
        }

        // Collect those glob-imported items that the source module reexports.  There should be no
        // name clashes in a prelude, so item reexports and glob reexports can be treated the same
        // way.
        let mut symbols = src_mod
            .root_items()
            .use_glob_synonyms
            .keys()
            .clone()
            .collect::<Vec<_>>();
        symbols.sort();
        for symbol in symbols {
            let bindings = &src_mod.root_items().use_glob_synonyms[symbol];
            for (path, decl, src_visibility) in bindings.iter() {
                // Preludes reexport all their imports.
                assert!(matches!(src_visibility, Visibility::Public));
                imports.push((symbol.clone(), decl.clone(), path.clone()))
            }
        }

        let implemented_traits = src_mod.root_items().implemented_traits.clone();
        let dst_mod = self.current_module_mut();

        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines);

        let dst_prelude_synonyms = &mut dst_mod.current_items_mut().prelude_synonyms;
        imports.iter().for_each(|(symbol, decl, path)| {
            // Preludes should not contain name clashes
            assert!(!dst_prelude_synonyms.contains_key(symbol));
            dst_prelude_synonyms.insert(symbol.clone(), (path.clone(), decl.clone()));
        });

        Ok(())
    }

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the
    /// current module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// `src` is assumed to be absolute.
    pub(crate) fn star_import_to_current_module(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_visibility(handler, src)?;

        let src_mod = self.require_module_from_absolute_path(handler, src)?;

        let mut decls_and_item_imports = vec![];

        // Collect all items declared in the source module
        let mut symbols = src_mod
            .root_items()
            .symbols
            .keys()
            .clone()
            .collect::<Vec<_>>();
        symbols.sort();
        for symbol in symbols {
            let decl = &src_mod.root_items().symbols[symbol];
            if self.is_ancestor_of_current_module(src) || decl.visibility(engines).is_public() {
                decls_and_item_imports.push((symbol.clone(), decl.clone(), src.to_vec()));
            }
        }
        // Collect those item-imported items that the source module reexports
        // These live in the same namespace as local declarations, so no shadowing is possible
        let mut symbols = src_mod
            .root_items()
            .use_item_synonyms
            .keys()
            .clone()
            .collect::<Vec<_>>();
        symbols.sort();
        for symbol in symbols {
            let (_, path, decl, src_visibility) = &src_mod.root_items().use_item_synonyms[symbol];
            if src_visibility.is_public() {
                decls_and_item_imports.push((symbol.clone(), decl.clone(), path.clone()))
            }
        }

        // Collect those glob-imported items that the source module reexports. These may be shadowed
        // by local declarations and item imports in the source module, so they are treated
        // separately.
        let mut glob_imports = vec![];
        let mut symbols = src_mod
            .root_items()
            .use_glob_synonyms
            .keys()
            .clone()
            .collect::<Vec<_>>();
        symbols.sort();
        for symbol in symbols {
            let bindings = &src_mod.root_items().use_glob_synonyms[symbol];
            // Ignore if the symbol is shadowed by a local declaration or an item import in the source module
            if !decls_and_item_imports
                .iter()
                .any(|(other_symbol, _, _)| symbol == other_symbol)
            {
                for (path, decl, src_visibility) in bindings.iter() {
                    if src_visibility.is_public() {
                        glob_imports.push((symbol.clone(), decl.clone(), path.clone()))
                    }
                }
            }
        }

        let implemented_traits = src_mod.root_items().implemented_traits.clone();
        let dst_mod = self.current_module_mut();

        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines);

        decls_and_item_imports
            .iter()
            .chain(glob_imports.iter())
            .for_each(|(symbol, decl, path)| {
                dst_mod.current_items_mut().insert_glob_use_symbol(
                    engines,
                    symbol.clone(),
                    path.clone(),
                    decl,
                    visibility,
                )
            });

        Ok(())
    }

    /// Pull all variants from the enum `enum_name` from the given `src` module and import them all into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    pub(crate) fn variant_star_import_to_current_module(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        enum_name: &Ident,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_visibility(handler, src)?;

        let parsed_decl_engine = engines.pe();
        let decl_engine = engines.de();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, false)?;

        match decl {
            ResolvedDeclaration::Parsed(Declaration::EnumDeclaration(decl_id)) => {
                let enum_decl = parsed_decl_engine.get_enum(&decl_id);

                for variant in enum_decl.variants.iter() {
                    let variant_name = &variant.name;
                    let variant_decl =
                        Declaration::EnumVariantDeclaration(EnumVariantDeclaration {
                            enum_ref: decl_id,
                            variant_name: variant_name.clone(),
                            variant_decl_span: variant.span.clone(),
                        });

                    // import it this way.
                    self.current_module_mut()
                        .current_items_mut()
                        .insert_glob_use_symbol(
                            engines,
                            variant_name.clone(),
                            path.clone(),
                            &ResolvedDeclaration::Parsed(variant_decl),
                            visibility,
                        );
                }
            }
            ResolvedDeclaration::Typed(TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. })) => {
                let enum_decl = decl_engine.get_enum(&decl_id);
                let enum_ref = DeclRef::new(
                    enum_decl.call_path.suffix.clone(),
                    decl_id,
                    enum_decl.span(),
                );

                for variant_decl in enum_decl.variants.iter() {
                    let variant_name = &variant_decl.name;
                    let decl =
                        ResolvedDeclaration::Typed(TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                            enum_ref: enum_ref.clone(),
                            variant_name: variant_name.clone(),
                            variant_decl_span: variant_decl.span.clone(),
                        }));

                    // import it this way.
                    self.current_module_mut()
                        .current_items_mut()
                        .insert_glob_use_symbol(
                            engines,
                            variant_name.clone(),
                            path.clone(),
                            &decl,
                            visibility,
                        );
                }
            }
            _ => {
                return Err(handler.emit_err(CompileError::Internal(
                    "Attempting to import variants of something that isn't an enum",
                    enum_name.span(),
                )));
            }
        };

        Ok(())
    }

    /// Pull a single item from a `src` module and import it into the current module.
    ///
    /// The item we want to import is the last item in path because this is a `self` import.
    pub(crate) fn self_import_to_current_module(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import_to_current_module(handler, engines, src, last_item, alias, visibility)
    }

    /// Pull a single `item` from the given `src` module and import it into the current module.
    ///
    /// `src` is assumed to be absolute.
    pub(crate) fn item_import_to_current_module(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        item: &Ident,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_visibility(handler, src)?;

        let src_mod = self.require_module_from_absolute_path(handler, src)?;

        let (decl, path) = self.item_lookup(handler, engines, item, src, false)?;

        let mut impls_to_insert = TraitMap::default();
        if decl.is_typed() {
            // We only handle trait imports when handling typed declarations,
            // that is, when performing type-checking, and not when collecting.
            // Update this once the type system is updated to refer to parsed
            // declarations.
            //  if this is an enum or struct or function, import its implementations
            if let Ok(type_id) = decl.return_type(&Handler::default(), engines) {
                impls_to_insert.extend(
                    src_mod
                        .root_items()
                        .implemented_traits
                        .filter_by_type_item_import(type_id, engines),
                    engines,
                );
            }
            // if this is a trait, import its implementations
            let decl_span = decl.span(engines);
            if decl.is_trait() {
                // TODO: we only import local impls from the source namespace
                // this is okay for now but we'll need to device some mechanism to collect all available trait impls
                impls_to_insert.extend(
                    src_mod
                        .root_items()
                        .implemented_traits
                        .filter_by_trait_decl_span(decl_span),
                    engines,
                );
            }
        }

        // no matter what, import it this way though.
        let dst_mod = self.current_module_mut();
        let check_name_clash = |name| {
            if dst_mod.current_items().use_item_synonyms.contains_key(name) {
                handler.emit_err(CompileError::ShadowsOtherSymbol { name: name.into() });
            }
        };
        match alias {
            Some(alias) => {
                check_name_clash(&alias);
                dst_mod
                    .current_items_mut()
                    .use_item_synonyms
                    .insert(alias.clone(), (Some(item.clone()), path, decl, visibility))
            }
            None => {
                check_name_clash(item);
                dst_mod
                    .current_items_mut()
                    .use_item_synonyms
                    .insert(item.clone(), (None, path, decl, visibility))
            }
        };

        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(impls_to_insert, engines);

        Ok(())
    }

    /// Pull a single variant `variant` from the enum `enum_name` from the given `src` module and
    /// import it into the current module.
    ///
    /// `src` is assumed to be absolute.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn variant_import_to_current_module(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        enum_name: &Ident,
        variant_name: &Ident,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_visibility(handler, src)?;

        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, false)?;

        match decl {
            ResolvedDeclaration::Parsed(decl) => {
                if let Declaration::EnumDeclaration(decl_id) = decl {
                    let enum_decl = parsed_decl_engine.get_enum(&decl_id);

                    if let Some(variant_decl) =
                        enum_decl.variants.iter().find(|v| v.name == *variant_name)
                    {
                        // import it this way.
                        let dst_mod = self.current_module_mut();
                        let check_name_clash = |name| {
                            if dst_mod.current_items().use_item_synonyms.contains_key(name) {
                                handler.emit_err(CompileError::ShadowsOtherSymbol {
                                    name: name.into(),
                                });
                            }
                        };

                        match alias {
                            Some(alias) => {
                                check_name_clash(&alias);
                                dst_mod.current_items_mut().use_item_synonyms.insert(
                                    alias.clone(),
                                    (
                                        Some(variant_name.clone()),
                                        path,
                                        ResolvedDeclaration::Parsed(
                                            Declaration::EnumVariantDeclaration(
                                                EnumVariantDeclaration {
                                                    enum_ref: decl_id,
                                                    variant_name: variant_name.clone(),
                                                    variant_decl_span: variant_decl.span.clone(),
                                                },
                                            ),
                                        ),
                                        visibility,
                                    ),
                                );
                            }
                            None => {
                                check_name_clash(variant_name);
                                dst_mod.current_items_mut().use_item_synonyms.insert(
                                    variant_name.clone(),
                                    (
                                        None,
                                        path,
                                        ResolvedDeclaration::Parsed(
                                            Declaration::EnumVariantDeclaration(
                                                EnumVariantDeclaration {
                                                    enum_ref: decl_id,
                                                    variant_name: variant_name.clone(),
                                                    variant_decl_span: variant_decl.span.clone(),
                                                },
                                            ),
                                        ),
                                        visibility,
                                    ),
                                );
                            }
                        };
                    } else {
                        return Err(handler.emit_err(CompileError::SymbolNotFound {
                            name: variant_name.clone(),
                            span: variant_name.span(),
                        }));
                    }
                }
            }
            ResolvedDeclaration::Typed(decl) => {
                if let TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) = decl {
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
                        let dst_mod = self.current_module_mut();
                        let check_name_clash = |name| {
                            if dst_mod.current_items().use_item_synonyms.contains_key(name) {
                                handler.emit_err(CompileError::ShadowsOtherSymbol {
                                    name: name.into(),
                                });
                            }
                        };

                        match alias {
                            Some(alias) => {
                                check_name_clash(&alias);
                                dst_mod.current_items_mut().use_item_synonyms.insert(
                                    alias.clone(),
                                    (
                                        Some(variant_name.clone()),
                                        path,
                                        ResolvedDeclaration::Typed(TyDecl::EnumVariantDecl(
                                            ty::EnumVariantDecl {
                                                enum_ref: enum_ref.clone(),
                                                variant_name: variant_name.clone(),
                                                variant_decl_span: variant_decl.span.clone(),
                                            },
                                        )),
                                        visibility,
                                    ),
                                );
                            }
                            None => {
                                check_name_clash(variant_name);
                                dst_mod.current_items_mut().use_item_synonyms.insert(
                                    variant_name.clone(),
                                    (
                                        None,
                                        path,
                                        ResolvedDeclaration::Typed(TyDecl::EnumVariantDecl(
                                            ty::EnumVariantDecl {
                                                enum_ref: enum_ref.clone(),
                                                variant_name: variant_name.clone(),
                                                variant_decl_span: variant_decl.span.clone(),
                                            },
                                        )),
                                        visibility,
                                    ),
                                );
                            }
                        };
                    } else {
                        return Err(handler.emit_err(CompileError::SymbolNotFound {
                            name: variant_name.clone(),
                            span: variant_name.span(),
                        }));
                    }
                } else {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    )));
                }
            }
        };

        Ok(())
    }

    /// Look up an item in the `src` module. Visibility is checked (if not ignored) from the current
    /// module.
    fn item_lookup(
        &self,
        handler: &Handler,
        engines: &Engines,
        item: &Ident,
        src: &ModulePath,
        ignore_visibility: bool,
    ) -> Result<(ResolvedDeclaration, ModulePathBuf), ErrorEmitted> {
        let src_mod = self.require_module_from_absolute_path(handler, src)?;
        let src_items = src_mod.root_items();

        let (decl, path, src_visibility) = if let Some(decl) = src_items.symbols.get(item) {
            let visibility = if self.is_ancestor_of_current_module(src) {
                Visibility::Public
            } else {
                decl.visibility(engines)
            };
            (decl.clone(), src.to_vec(), visibility)
        } else if let Some((_, path, decl, reexport)) = src_items.use_item_synonyms.get(item) {
            (decl.clone(), path.clone(), *reexport)
        } else if let Some(decls) = src_items.use_glob_synonyms.get(item) {
            if decls.len() == 1 {
                let (path, decl, reexport) = &decls[0];
                (decl.clone(), path.clone(), *reexport)
            } else if decls.is_empty() {
                return Err(handler.emit_err(CompileError::Internal(
            "The name {symbol} was bound in a star import, but no corresponding module paths were found",
            item.span(),
                    )));
            } else {
                return Err(handler.emit_err(CompileError::SymbolWithMultipleBindings {
                    name: item.clone(),
                    paths: decls
                        .iter()
                        .map(|(path, decl, _)| {
                            let mut path_strs = super::lexical_scope::get_path_for_decl(
                                path,
                                decl,
                                engines,
                                self.current_package_name(),
                            );
                            // Add the enum name to the path if the decl is an enum variant.
                            if let TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                enum_ref, ..
                            }) = decl.expect_typed_ref()
                            {
                                path_strs.push(enum_ref.name().to_string())
                            };
                            path_strs.join("::")
                        })
                        .collect(),
                    span: item.span(),
                }));
            }
        } else {
            // Symbol not found
            return Err(handler.emit_err(CompileError::SymbolNotFound {
                name: item.clone(),
                span: item.span(),
            }));
        };

        if !ignore_visibility && !src_visibility.is_public() {
            handler.emit_err(CompileError::ImportPrivateSymbol {
                name: item.clone(),
                span: item.span(),
            });
        }

        Ok((decl, path))
    }

    /// Check that all accessed modules in the src path are visible from the current module.
    ///
    /// Only the module part of the src path will be checked. If the src path contains identifiers
    /// that refer to non-modules, e.g., enum names or associated types, then the visibility of
    /// those items will not be checked.
    ///
    /// If src and the current module have a common ancestor module that is private, this privacy
    /// modifier is ignored for visibility purposes, since src and the current module are both
    /// behind that private visibility modifier.  Additionally, items in a private module are
    /// visible to its immediate parent.
    pub(crate) fn check_module_visibility(
        &self,
        handler: &Handler,
        src: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        let dst = &self.current_mod_path;

        // Calculate the number of src prefixes whose visibility is ignored.
        let mut ignored_prefixes = 0;

        // Ignore visibility of common ancestors
        ignored_prefixes += src
            .iter()
            .zip(dst)
            .position(|(src_id, dst_id)| src_id != dst_id)
            .unwrap_or(dst.len());

        // Ignore visibility of direct submodules of the destination module
        if dst.len() == ignored_prefixes {
            ignored_prefixes += 1;
        }

        // Check visibility of remaining submodules in the source path
        for prefix in iter_prefixes(src).skip(ignored_prefixes) {
            if let Some(module) = self.module_from_absolute_path(prefix) {
                if module.visibility().is_private() {
                    let prefix_last = prefix[prefix.len() - 1].clone();
                    handler.emit_err(CompileError::ImportPrivateModule {
                        span: prefix_last.span(),
                        name: prefix_last,
                    });
                }
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    fn is_ancestor_of_current_module(&self, src: &ModulePath) -> bool {
        let dst = &self.current_mod_path;
        dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
    }
}
