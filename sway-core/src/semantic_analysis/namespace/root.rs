use std::fmt;

use super::{module::Module, trait_map::TraitMap, Ident};
use crate::{
    decl_engine::{DeclEngine, DeclRef},
    engine_threading::*,
    language::{
        parsed::*,
        ty::{self, StructDecl, TyDecl},
        CallPath, Visibility,
    },
    namespace::{ModulePath, ModulePathBuf},
    semantic_analysis::type_resolve::{resolve_associated_item, resolve_associated_type},
    TypeId,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;
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

/// The root module, from which all other modules can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug, Default)]
pub struct Root {
    pub(crate) module: Module,
}

impl Root {
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
        self.check_module_privacy(handler, engines, src, dst)?;

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let mut decls_and_item_imports = vec![];

        let get_path = |mod_path: Vec<Ident>| {
            let mut is_external = false;
            if let Some(submodule) = src_mod.submodule(engines, &[mod_path[0].clone()]) {
                is_external = submodule.is_external
            };

            let mut path = src[..1].to_vec();
            if is_external {
                path = mod_path;
            } else {
                path.extend(mod_path);
            }

            path
        };

        // Collect all items declared in the source module
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(engines).is_public() {
                decls_and_item_imports.push((symbol.clone(), decl.clone(), src.to_vec()));
            }
        }
        // Collect those item-imported items that the source module reexports
        // These live in the same namespace as local declarations, so no shadowing is possible
        for (symbol, (_, path, decl, src_visibility)) in
            src_mod.current_items().use_item_synonyms.iter()
        {
            if src_visibility.is_public() {
                decls_and_item_imports.push((symbol.clone(), decl.clone(), get_path(path.clone())))
            }
        }

        // Collect those glob-imported items that the source module reexports. These may be shadowed
        // by local declarations and item imports in the source module, so they are treated
        // separately.
        let mut glob_imports = vec![];
        for (symbol, bindings) in src_mod.current_items().use_glob_synonyms.iter() {
            // Ignore if the symbol is shadowed by a local declaration or an item import in the source module
            if !decls_and_item_imports
                .iter()
                .any(|(other_symbol, _, _)| symbol == other_symbol)
            {
                for (path, decl, src_visibility) in bindings.iter() {
                    if src_visibility.is_public() {
                        glob_imports.push((symbol.clone(), decl.clone(), get_path(path.clone())))
                    }
                }
            }
        }

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
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

    fn item_lookup(
        &self,
        handler: &Handler,
        engines: &Engines,
        item: &Ident,
        src: &ModulePath,
        dst: &ModulePath,
    ) -> Result<(ResolvedDeclaration, ModulePathBuf), ErrorEmitted> {
        let src_mod = self.module.lookup_submodule(handler, engines, src)?;
        let src_items = src_mod.current_items();

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
                            let mut path_strs = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                            // Add the enum name to the path if the decl is an enum variant.
                            if let TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                enum_ref, ..
                            }) = decl.expect_typed_ref()
                            {
                                path_strs.push(enum_ref.name().as_str())
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

        if !src_visibility.is_public() {
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
        self.check_module_privacy(handler, engines, src, dst)?;
        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let (decl, path) = self.item_lookup(handler, engines, item, src, dst)?;

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
                        .current_items()
                        .implemented_traits
                        .filter_by_type_item_import(
                            type_id,
                            engines,
                            super::CodeBlockFirstPass::No,
                        ),
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
                        .current_items()
                        .implemented_traits
                        .filter_by_trait_decl_span(decl_span),
                    engines,
                );
            }
        }

        // no matter what, import it this way though.
        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
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
        self.check_module_privacy(handler, engines, src, dst)?;

        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, dst)?;

        match decl {
            ResolvedDeclaration::Parsed(decl) => {
                if let Declaration::EnumDeclaration(decl_id) = decl {
                    let enum_decl = parsed_decl_engine.get_enum(&decl_id);

                    if let Some(variant_decl) =
                        enum_decl.variants.iter().find(|v| v.name == *variant_name)
                    {
                        // import it this way.
                        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
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
                        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
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
        self.check_module_privacy(handler, engines, src, dst)?;

        let parsed_decl_engine = engines.pe();
        let decl_engine = engines.de();

        let (decl, path) = self.item_lookup(handler, engines, enum_name, src, dst)?;

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
                    self.module
                        .lookup_submodule_mut(handler, engines, dst)?
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
                    self.module
                        .lookup_submodule_mut(handler, engines, dst)?
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

    fn check_module_privacy(
        &self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        // you are always allowed to access your ancestor's symbols
        if !is_ancestor(src, dst) {
            // we don't check the first prefix because direct children are always accessible
            for prefix in iter_prefixes(src).skip(1) {
                let module = self.module.lookup_submodule(handler, engines, prefix)?;
                if module.visibility().is_private() {
                    let prefix_last = prefix[prefix.len() - 1].clone();
                    handler.emit_err(CompileError::ImportPrivateModule {
                        span: prefix_last.span(),
                        name: prefix_last,
                    });
                }
            }
        }
        Ok(())
    }

    ////// NAME RESOLUTION //////

    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &ModulePath,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let (decl, _) =
            self.resolve_call_path_and_mod_path(handler, engines, mod_path, call_path, self_type)?;
        Ok(decl)
    }

    pub(crate) fn resolve_call_path_and_mod_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &ModulePath,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<(ResolvedDeclaration, ModulePathBuf), ErrorEmitted> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol_and_mod_path(
            handler,
            engines,
            &symbol_path,
            &call_path.suffix,
            self_type,
        )
    }

    /// Given a path to a module and the identifier of a symbol within that module, resolve its
    /// declaration.
    ///
    /// If the symbol is within the given module's namespace via import, we recursively traverse
    /// imports until we find the original declaration.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &ModulePath,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let (decl, _) =
            self.resolve_symbol_and_mod_path(handler, engines, mod_path, symbol, self_type)?;
        Ok(decl)
    }

    fn resolve_symbol_and_mod_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &ModulePath,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<(ResolvedDeclaration, Vec<Ident>), ErrorEmitted> {
        // This block tries to resolve associated types
        let mut module = &self.module;
        let mut current_mod_path = vec![];
        let mut decl_opt = None;
        for ident in mod_path.iter() {
            if let Some(decl) = decl_opt {
                decl_opt = Some(resolve_associated_type(
                    handler, engines, module, ident, decl, None, self_type,
                )?);
            } else {
                match module.submodules.get(ident.as_str()) {
                    Some(ns) => {
                        module = ns;
                        current_mod_path.push(ident.clone());
                    }
                    None => {
                        decl_opt = Some(
                            module
                                .current_lexical_scope()
                                .items
                                .resolve_symbol(handler, engines, ident)?,
                        );
                    }
                }
            }
        }
        if let Some(decl) = decl_opt {
            let decl =
                resolve_associated_item(handler, engines, module, symbol, decl, None, self_type)?;
            return Ok((decl, current_mod_path));
        }

        self.module
            .lookup_submodule(handler, engines, mod_path)
            .and_then(|module| {
                let decl = module
                    .current_lexical_scope()
                    .items
                    .resolve_symbol(handler, engines, symbol)?;
                Ok((decl, mod_path.to_vec()))
            })
    }
}

impl From<Module> for Root {
    fn from(module: Module) -> Self {
        Root { module }
    }
}

fn is_ancestor(src: &ModulePath, dst: &ModulePath) -> bool {
    dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
}
