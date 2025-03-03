use std::fmt;

use super::{module::Module, trait_map::TraitMap, Ident, ModuleName};
use crate::{
    decl_engine::{DeclEngine, DeclRef},
    engine_threading::*,
    language::{
        parsed::*,
        ty::{self, StructDecl, TyDecl},
        Visibility,
    },
    namespace::{ModulePath, ModulePathBuf},
    TypeId,
};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, ProgramId, Spanned};
use sway_utils::iter_prefixes;

#[derive(Clone, Debug)]
pub enum ResolvedDeclaration {
    Parsed(Declaration),
    Typed(ty::TyDecl),
}

impl DisplayWithEngines for ResolvedDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            ResolvedDeclaration::Parsed(decl) => DisplayWithEngines::fmt(decl, f, engines),
            ResolvedDeclaration::Typed(decl) => DisplayWithEngines::fmt(decl, f, engines),
        }
    }
}

impl DebugWithEngines for ResolvedDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            ResolvedDeclaration::Parsed(decl) => DebugWithEngines::fmt(decl, f, engines),
            ResolvedDeclaration::Typed(decl) => DebugWithEngines::fmt(decl, f, engines),
        }
    }
}

impl PartialEqWithEngines for ResolvedDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (ResolvedDeclaration::Parsed(lhs), ResolvedDeclaration::Parsed(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ResolvedDeclaration::Typed(lhs), ResolvedDeclaration::Typed(rhs)) => lhs.eq(rhs, ctx),
            // TODO: Right now we consider differently represented resolved declarations to not be
            // equal. This is only used for comparing paths when doing imports, and we will be able
            // to safely remove it once we introduce normalized paths.
            (ResolvedDeclaration::Parsed(_lhs), ResolvedDeclaration::Typed(_rhs)) => false,
            (ResolvedDeclaration::Typed(_lhs), ResolvedDeclaration::Parsed(_rhs)) => false,
        }
    }
}

impl ResolvedDeclaration {
    pub fn is_typed(&self) -> bool {
        match self {
            ResolvedDeclaration::Parsed(_) => false,
            ResolvedDeclaration::Typed(_) => true,
        }
    }

    pub fn resolve_parsed(self, decl_engine: &DeclEngine) -> Declaration {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl,
            ResolvedDeclaration::Typed(ty_decl) => ty_decl
                .get_parsed_decl(decl_engine)
                .expect("expecting valid parsed declaration"),
        }
    }

    pub fn expect_parsed(self) -> Declaration {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl,
            ResolvedDeclaration::Typed(_ty_decl) => panic!(),
        }
    }

    pub fn expect_typed(self) -> ty::TyDecl {
        match self {
            ResolvedDeclaration::Parsed(_) => panic!(),
            ResolvedDeclaration::Typed(ty_decl) => ty_decl,
        }
    }

    pub fn expect_typed_ref(&self) -> &ty::TyDecl {
        match self {
            ResolvedDeclaration::Parsed(_) => panic!(),
            ResolvedDeclaration::Typed(ty_decl) => ty_decl,
        }
    }

    pub(crate) fn to_struct_decl(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl
                .to_struct_decl(handler, engines)
                .map(|id| ResolvedDeclaration::Parsed(Declaration::StructDeclaration(id))),
            ResolvedDeclaration::Typed(decl) => decl.to_struct_decl(handler, engines).map(|id| {
                ResolvedDeclaration::Typed(TyDecl::StructDecl(StructDecl { decl_id: id }))
            }),
        }
    }

    pub(crate) fn visibility(&self, engines: &Engines) -> Visibility {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl.visibility(engines.pe()),
            ResolvedDeclaration::Typed(decl) => decl.visibility(engines.de()),
        }
    }

    fn span(&self, engines: &Engines) -> sway_types::Span {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl.span(engines),
            ResolvedDeclaration::Typed(decl) => decl.span(engines),
        }
    }

    pub(crate) fn return_type(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<TypeId, ErrorEmitted> {
        match self {
            ResolvedDeclaration::Parsed(_decl) => unreachable!(),
            ResolvedDeclaration::Typed(decl) => decl.return_type(handler, engines),
        }
    }

    fn is_trait(&self) -> bool {
        match self {
            ResolvedDeclaration::Parsed(decl) => {
                matches!(decl, Declaration::TraitDeclaration(_))
            }
            ResolvedDeclaration::Typed(decl) => {
                matches!(decl, TyDecl::TraitDecl(_))
            }
        }
    }
}

/// The root module, from which all other module dependencies can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug)]
pub struct Root {
    // The contents of the package being compiled.
    current_package: Module,
    // Program id for the package.
    program_id: ProgramId,
    // True if the current package is a contract, false otherwise.
    is_contract_package: bool,
    // The external dependencies of the current package. Note that an external package is
    // represented as a `Root` object. This is because external packages may have their own external
    // dependencies which are needed for lookups, but which are not directly accessible to the
    // current package.
    external_packages: im::HashMap<ModuleName, Root, BuildHasherDefault<FxHasher>>,
}

impl Root {
    // Create a new root object with a root module in the current package.
    //
    // To ensure the correct initialization the factory functions `package_root_without_contract_id`
    // and `package_root_with_contract_id` are supplied in `contract_helpers`.
    //
    // External packages must be added afterwards by calling `add_external`
    pub fn new(
        package_name: Ident,
        span: Option<Span>,
        program_id: ProgramId,
        is_contract_package: bool,
    ) -> Self {
        // The root module must be public
        let module = Module::new(package_name, Visibility::Public, span, &vec![]);
        Self {
            current_package: module,
            program_id,
            is_contract_package,
            external_packages: Default::default(),
        }
    }

    // Add an external package to this package. The package name must be supplied, since the package
    // may be referred to by a different name in the forc.toml file than the actual name of the
    // package.
    pub fn add_external(&mut self, package_name: String, external_package: Root) {
        // This should be ensured by the package manager
        assert!(!self.external_packages.contains_key(&package_name));
        self.external_packages
            .insert(package_name, external_package);
    }

    pub(crate) fn get_external_package(&self, package_name: &String) -> Option<&Root> {
        self.external_packages.get(package_name)
    }

    pub(super) fn exists_as_external(&self, package_name: &String) -> bool {
        self.get_external_package(package_name).is_some()
    }

    pub fn external_packages(
        &self,
    ) -> &im::HashMap<ModuleName, Root, BuildHasherDefault<FxHasher>> {
        &self.external_packages
    }

    pub fn current_package_root_module(&self) -> &Module {
        &self.current_package
    }

    pub fn current_package_root_module_mut(&mut self) -> &mut Module {
        &mut self.current_package
    }

    pub fn current_package_name(&self) -> &Ident {
        self.current_package.name()
    }

    pub fn program_id(&self) -> ProgramId {
        self.program_id
    }

    fn check_path_is_in_current_package(&self, mod_path: &ModulePathBuf) -> bool {
        !mod_path.is_empty() && mod_path[0] == *self.current_package.name()
    }

    fn package_relative_path(mod_path: &ModulePathBuf) -> ModulePathBuf {
        mod_path[1..].to_vec()
    }

    pub(super) fn is_contract_package(&self) -> bool {
        self.is_contract_package
    }

    // Find module in the current environment. `mod_path` must be a fully qualified path
    pub fn module_from_absolute_path(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if mod_path[0] == *self.current_package.name() {
            self.current_package.submodule(&package_relative_path)
        } else if let Some(external_package) = self.external_packages.get(&mod_path[0].to_string())
        {
            external_package
                .current_package_root_module()
                .submodule(&package_relative_path)
        } else {
            None
        }
    }

    // Find module in the current environment. `mod_path` must be a fully qualified path.
    // Throw an error if the module doesn't exist
    pub(crate) fn require_module(
        &self,
        handler: &Handler,
        mod_path: &ModulePathBuf,
    ) -> Result<&Module, ErrorEmitted> {
        if mod_path.is_empty() {
            return Err(handler.emit_err(CompileError::Internal(
                "Found empty absolute mod path",
                Span::dummy(),
            )));
        }
        let is_in_current_package = self.check_path_is_in_current_package(mod_path);
        match self.module_from_absolute_path(mod_path) {
            Some(module) => Ok(module),
            None => Err(handler.emit_err(crate::namespace::module::module_not_found(
                mod_path,
                is_in_current_package,
            ))),
        }
    }

    // Find a module in the current package. `mod_path` must be a fully qualified path
    pub(super) fn module_in_current_package(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(self.check_path_is_in_current_package(mod_path));
        self.module_from_absolute_path(mod_path)
    }

    // Find mutable module in the current environment. `mod_path` must be a fully qualified path
    pub(super) fn module_mut_from_absolute_path(
        &mut self,
        mod_path: &ModulePathBuf,
    ) -> Option<&mut Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if *self.current_package.name() == mod_path[0] {
            self.current_package.submodule_mut(&package_relative_path)
        } else if let Some(external_package) =
            self.external_packages.get_mut(&mod_path[0].to_string())
        {
            external_package.module_mut_in_current_package(&package_relative_path)
        } else {
            None
        }
    }

    // Find mutable module in the current environment. `mod_path` must be a fully qualified path.
    // Throw an error if the module doesn't exist
    pub(super) fn require_module_mut(
        &mut self,
        handler: &Handler,
        mod_path: &ModulePathBuf,
    ) -> Result<&mut Module, ErrorEmitted> {
        let is_in_current_package = self.check_path_is_in_current_package(mod_path);
        match self.module_mut_from_absolute_path(mod_path) {
            Some(module) => Ok(module),
            None => Err(handler.emit_err(crate::namespace::module::module_not_found(
                mod_path,
                is_in_current_package,
            ))),
        }
    }

    // Find a mutable module in the current package. `mod_path` must be a fully qualified path
    pub(super) fn module_mut_in_current_package(
        &mut self,
        mod_path: &ModulePathBuf,
    ) -> Option<&mut Module> {
        assert!(self.check_path_is_in_current_package(mod_path));
        self.module_mut_from_absolute_path(mod_path)
    }

    // Find a mutable module in the current package. `mod_path` must be a fully qualified path
    // Throw an error if the module doesn't exist
    pub(super) fn require_module_mut_in_current_package(
        &mut self,
        handler: &Handler,
        mod_path: &ModulePathBuf,
    ) -> Result<&mut Module, ErrorEmitted> {
        assert!(self.check_path_is_in_current_package(mod_path));
        self.require_module_mut(handler, mod_path)
    }

    ////// IMPORT //////

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be absolute.
    pub fn star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src, dst)?;

        let src_mod = self.require_module(handler, &src.to_vec())?;

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
            if is_ancestor(src, dst) || decl.visibility(engines).is_public() {
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
        let dst_mod = self.require_module_mut_in_current_package(handler, &dst.to_vec())?;

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

    /// Pull a single item from a `src` module and import it into the `dst` module.
    ///
    /// The item we want to import is basically the last item in path because this is a `self`
    /// import.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(handler, engines, src, last_item, dst, alias, visibility)
    }

    pub(super) fn item_lookup(
        &self,
        handler: &Handler,
        engines: &Engines,
        item: &Ident,
        src: &ModulePath,
        dst: &ModulePath,
        ignore_visibility: bool,
    ) -> Result<(ResolvedDeclaration, ModulePathBuf), ErrorEmitted> {
        let src_mod = self.require_module(handler, &src.to_vec())?;
        let src_items = src_mod.root_items();

        let (decl, path, src_visibility) = if let Some(decl) = src_items.symbols.get(item) {
            let visibility = if is_ancestor(src, dst) {
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

    /// Pull a single `item` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        item: &Ident,
        dst: &ModulePath,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src, dst)?;
        let src_mod = self.require_module(handler, &src.to_vec())?;

        let (decl, path) = self.item_lookup(handler, engines, item, src, dst, false)?;

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
        let dst_mod = self.require_module_mut_in_current_package(handler, &dst.to_vec())?;
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

    /// Pull a single variant `variant` from the enum `enum_name` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        enum_name: &Ident,
        variant_name: &Ident,
        dst: &ModulePath,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src, dst)?;

        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, dst, false)?;

        match decl {
            ResolvedDeclaration::Parsed(decl) => {
                if let Declaration::EnumDeclaration(decl_id) = decl {
                    let enum_decl = parsed_decl_engine.get_enum(&decl_id);

                    if let Some(variant_decl) =
                        enum_decl.variants.iter().find(|v| v.name == *variant_name)
                    {
                        // import it this way.
                        let dst_mod =
                            self.require_module_mut_in_current_package(handler, &dst.to_vec())?;
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
                        let dst_mod =
                            self.require_module_mut_in_current_package(handler, &dst.to_vec())?;
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

    /// Pull all variants from the enum `enum_name` from the given `src` module and import them all into the `dst` module.
    ///
    /// Paths are assumed to be absolute.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
        enum_name: &Ident,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, src, dst)?;

        let parsed_decl_engine = engines.pe();
        let decl_engine = engines.de();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, dst, false)?;

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
                    self.require_module_mut_in_current_package(handler, &dst.to_vec())?
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
                    self.require_module_mut_in_current_package(handler, &dst.to_vec())?
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

    /// Check that all accessed modules in the src path are visible from the dst path.
    ///
    /// Only the module part of the src path will be checked. If the src path contains identifiers
    /// that refer to non-modules, e.g., enum names or associated types, then the visibility of
    /// those items will not be checked.
    ///
    /// If src and dst have a common ancestor module that is private, this privacy modifier is
    /// ignored for visibility purposes, since src and dst are both behind that private visibility
    /// modifier.  Additionally, items in a private module are visible to its immediate parent.
    ///
    /// The returned path is the part of the src path that refers to modules.
    pub(crate) fn check_module_privacy(
        &self,
        handler: &Handler,
        src: &ModulePath,
        dst: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
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
            if let Some(module) = self.module_from_absolute_path(&prefix.to_vec()) {
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
}

fn is_ancestor(src: &ModulePath, dst: &ModulePath) -> bool {
    dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
}
