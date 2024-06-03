use std::fmt;

use super::{module::Module, namespace::Namespace, Ident, ResolvedTraitImplItem};
use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    language::{
        parsed::*,
        ty::{self, TyDecl, TyTraitItem},
        CallPath, Visibility,
    },
    namespace::ModulePath,
    TypeId, TypeInfo,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Named, Spanned};
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

impl ResolvedDeclaration {
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

    pub(crate) fn visibility(&self, engines: &Engines) -> Visibility {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl.visibility(engines.pe()),
            ResolvedDeclaration::Typed(decl) => decl.visibility(engines.de()),
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
#[derive(Clone, Debug)]
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
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, engines, src)?;

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let mut symbols_and_decls = vec![];
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(engines).is_public() {
                symbols_and_decls.push((symbol.clone(), decl.clone()));
            }
        }

        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines);

        symbols_and_decls.iter().for_each(|(symbol, decl)| {
            dst_mod.current_items_mut().insert_glob_use_symbol(
                engines,
                symbol.clone(),
                src.to_vec(),
                decl.expect_typed_ref(),
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
    ) -> Result<(), ErrorEmitted> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(handler, engines, src, last_item, dst, alias)
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
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, engines, src)?;

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;
        match src_mod.current_items().symbols.get(item).cloned() {
            Some(decl) => {
                if !decl.visibility(engines).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: item.clone(),
                        span: item.span(),
                    });
                }

                // no matter what, import it this way though.
                let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
                let check_name_clash = |name| {
                    if let Some((_, _, _)) = dst_mod.current_items().use_item_synonyms.get(name) {
                        handler.emit_err(CompileError::ShadowsOtherSymbol { name: name.into() });
                    }
                };
                let decl = decl.expect_typed();
                match alias {
                    Some(alias) => {
                        check_name_clash(&alias);
                        dst_mod
                            .current_items_mut()
                            .use_item_synonyms
                            .insert(alias.clone(), (Some(item.clone()), src.to_vec(), decl))
                    }
                    None => {
                        check_name_clash(item);
                        dst_mod
                            .current_items_mut()
                            .use_item_synonyms
                            .insert(item.clone(), (None, src.to_vec(), decl))
                    }
                };
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: item.clone(),
                    span: item.span(),
                }));
            }
        };

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
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, engines, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;
        match src_mod.current_items().symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(engines).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) = decl.expect_typed() {
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
                                        src.to_vec(),
                                        TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                            enum_ref: enum_ref.clone(),
                                            variant_name: variant_name.clone(),
                                            variant_decl_span: variant_decl.span.clone(),
                                        }),
                                    ),
                                );
                            }
                            None => {
                                check_name_clash(variant_name);
                                dst_mod.current_items_mut().use_item_synonyms.insert(
                                    variant_name.clone(),
                                    (
                                        None,
                                        src.to_vec(),
                                        TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                            enum_ref: enum_ref.clone(),
                                            variant_name: variant_name.clone(),
                                            variant_decl_span: variant_decl.span.clone(),
                                        }),
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
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                }));
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
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, engines, src)?;

        let decl_engine = engines.de();

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;
        match src_mod.current_items().symbols.get(enum_name).cloned() {
            Some(decl) => {
                if !decl.visibility(engines).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: enum_name.clone(),
                        span: enum_name.span(),
                    });
                }

                if let TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) = decl.expect_typed() {
                    let enum_decl = decl_engine.get_enum(&decl_id);
                    let enum_ref = DeclRef::new(
                        enum_decl.call_path.suffix.clone(),
                        decl_id,
                        enum_decl.span(),
                    );

                    for variant_decl in enum_decl.variants.iter() {
                        let variant_name = &variant_decl.name;
                        let decl = TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                            enum_ref: enum_ref.clone(),
                            variant_name: variant_name.clone(),
                            variant_decl_span: variant_decl.span.clone(),
                        });

                        // import it this way.
                        self.module
                            .lookup_submodule_mut(handler, engines, dst)?
                            .current_items_mut()
                            .insert_glob_use_symbol(
                                engines,
                                variant_name.clone(),
                                src.to_vec(),
                                &decl,
                            );
                    }
                } else {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Attempting to import variants of something that isn't an enum",
                        enum_name.span(),
                    )));
                }
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: enum_name.clone(),
                    span: enum_name.span(),
                }));
            }
        };

        Ok(())
    }

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be absolute.
    pub fn star_import_with_reexports(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
        dst: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        self.check_module_privacy(handler, engines, src)?;

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let use_item_synonyms = src_mod.current_items().use_item_synonyms.clone();
        let use_glob_synonyms = src_mod.current_items().use_glob_synonyms.clone();

        // collect all declared and reexported symbols from the source module
        let mut all_symbols_and_decls = vec![];
        for (symbol, decls) in src_mod.current_items().use_glob_synonyms.iter() {
            decls
                .iter()
                .for_each(|(_, decl)| all_symbols_and_decls.push((symbol.clone(), decl.clone())));
        }
        for (symbol, (_, _, decl)) in src_mod.current_items().use_item_synonyms.iter() {
            all_symbols_and_decls.push((symbol.clone(), decl.clone()));
        }
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(engines).is_public() {
                all_symbols_and_decls.push((symbol.clone(), decl.clone().expect_typed()));
            }
        }

        let mut symbols_paths_and_decls = vec![];
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

        for (symbol, (_, mod_path, decl)) in use_item_synonyms {
            symbols_paths_and_decls.push((symbol, get_path(mod_path), decl));
        }
        for (symbol, decls) in use_glob_synonyms {
            decls.iter().for_each(|(mod_path, decl)| {
                symbols_paths_and_decls.push((
                    symbol.clone(),
                    get_path(mod_path.clone()),
                    decl.clone(),
                ))
            });
        }

        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines);

        let mut try_add = |symbol, path, decl: ty::TyDecl| {
            dst_mod
                .current_items_mut()
                .insert_glob_use_symbol(engines, symbol, path, &decl);
        };

        for (symbol, decl) in all_symbols_and_decls {
            try_add(symbol.clone(), src.to_vec(), decl);
        }

        for (symbol, path, decl) in symbols_paths_and_decls {
            try_add(symbol.clone(), path, decl);
        }

        Ok(())
    }

    fn check_module_privacy(
        &self,
        handler: &Handler,
        engines: &Engines,
        src: &ModulePath,
    ) -> Result<(), ErrorEmitted> {
        let dst = self.module.mod_path();
        // you are always allowed to access your ancestor's symbols
        if !is_ancestor(src, dst) {
            // we don't check the first prefix because direct children are always accessible
            for prefix in iter_prefixes(src).skip(1) {
                let module = self.module.lookup_submodule(handler, engines, prefix)?;
                if module.visibility.is_private() {
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
    ) -> Result<(ResolvedDeclaration, Vec<Ident>), ErrorEmitted> {
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_call_path_and_root_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        module: &Module,
        root_type_id: TypeId,
        mut as_trait: Option<CallPath>,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        // This block tries to resolve associated types
        let mut decl_opt = None;
        let mut type_id_opt = Some(root_type_id);
        for ident in call_path.prefixes.iter() {
            if let Some(type_id) = type_id_opt {
                type_id_opt = None;
                decl_opt = Some(self.resolve_associated_type_from_type_id(
                    handler,
                    engines,
                    module,
                    ident,
                    type_id,
                    as_trait.clone(),
                    self_type,
                )?);
                as_trait = None;
            } else if let Some(decl) = decl_opt {
                decl_opt = Some(self.resolve_associated_type(
                    handler,
                    engines,
                    module,
                    ident,
                    decl,
                    as_trait.clone(),
                    self_type,
                )?);
                as_trait = None;
            }
        }
        if let Some(type_id) = type_id_opt {
            let decl = self.resolve_associated_type_from_type_id(
                handler,
                engines,
                module,
                &call_path.suffix,
                type_id,
                as_trait,
                self_type,
            )?;
            return Ok(decl);
        }
        if let Some(decl) = decl_opt {
            let decl = self.resolve_associated_item(
                handler,
                engines,
                module,
                &call_path.suffix,
                decl,
                as_trait,
                self_type,
            )?;
            Ok(decl)
        } else {
            Err(handler.emit_err(CompileError::Internal("Unexpected error", call_path.span())))
        }
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
                decl_opt = Some(self.resolve_associated_type(
                    handler, engines, module, ident, decl, None, self_type,
                )?);
            } else {
                match module.submodules.get(ident.as_str()) {
                    Some(ns) => {
                        module = ns;
                        current_mod_path.push(ident.clone());
                    }
                    None => {
                        decl_opt = Some(self.resolve_symbol_helper(handler, ident, module)?);
                    }
                }
            }
        }
        if let Some(decl) = decl_opt {
            let decl = self
                .resolve_associated_item(handler, engines, module, symbol, decl, None, self_type)?;
            return Ok((decl, current_mod_path));
        }

        self.module
            .lookup_submodule(handler, engines, mod_path)
            .and_then(|module| {
                let decl = self.resolve_symbol_helper(handler, symbol, module)?;
                Ok((decl, mod_path.to_vec()))
            })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_associated_type(
        &self,
        handler: &Handler,
        engines: &Engines,
        module: &Module,
        symbol: &Ident,
        decl: ResolvedDeclaration,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let type_info = self.decl_to_type_info(handler, engines, symbol, decl)?;

        self.resolve_associated_type_from_type_id(
            handler,
            engines,
            module,
            symbol,
            engines
                .te()
                .insert(engines, type_info, symbol.span().source_id()),
            as_trait,
            self_type,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_associated_item(
        &self,
        handler: &Handler,
        engines: &Engines,
        module: &Module,
        symbol: &Ident,
        decl: ResolvedDeclaration,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let type_info = self.decl_to_type_info(handler, engines, symbol, decl)?;

        self.resolve_associated_item_from_type_id(
            handler,
            engines,
            module,
            symbol,
            engines
                .te()
                .insert(engines, type_info, symbol.span().source_id()),
            as_trait,
            self_type,
        )
    }

    fn decl_to_type_info(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        decl: ResolvedDeclaration,
    ) -> Result<TypeInfo, ErrorEmitted> {
        match decl {
            ResolvedDeclaration::Parsed(_decl) => todo!(),
            ResolvedDeclaration::Typed(decl) => Ok(match decl.clone() {
                ty::TyDecl::StructDecl(struct_ty_decl) => {
                    let struct_decl = engines.de().get_struct(&struct_ty_decl.decl_id);
                    TypeInfo::Struct(DeclRef::new(
                        struct_decl.name().clone(),
                        struct_ty_decl.decl_id,
                        struct_decl.span().clone(),
                    ))
                }
                ty::TyDecl::EnumDecl(enum_ty_decl) => {
                    let enum_decl = engines.de().get_enum(&enum_ty_decl.decl_id);
                    TypeInfo::Enum(DeclRef::new(
                        enum_decl.name().clone(),
                        enum_ty_decl.decl_id,
                        enum_decl.span().clone(),
                    ))
                }
                ty::TyDecl::TraitTypeDecl(type_decl) => {
                    let type_decl = engines.de().get_type(&type_decl.decl_id);
                    (*engines.te().get(type_decl.ty.clone().unwrap().type_id)).clone()
                }
                _ => {
                    return Err(handler.emit_err(CompileError::SymbolNotFound {
                        name: symbol.clone(),
                        span: symbol.span(),
                    }))
                }
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_associated_type_from_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        module: &Module,
        symbol: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let item_decl = self.resolve_associated_item_from_type_id(
            handler, engines, module, symbol, type_id, as_trait, self_type,
        )?;
        Ok(item_decl)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_associated_item_from_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        module: &Module,
        symbol: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let type_id = if engines.te().get(type_id).is_self_type() {
            if let Some(self_type) = self_type {
                self_type
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Self type not provided.",
                    symbol.span(),
                )));
            }
        } else {
            type_id
        };
        let item_ref = module
            .current_items()
            .implemented_traits
            .get_trait_item_for_type(handler, engines, symbol, type_id, as_trait)?;
        match item_ref {
            ResolvedTraitImplItem::Parsed(_item) => todo!(),
            ResolvedTraitImplItem::Typed(item) => match item {
                TyTraitItem::Fn(fn_ref) => Ok(ResolvedDeclaration::Typed(fn_ref.into())),
                TyTraitItem::Constant(const_ref) => {
                    Ok(ResolvedDeclaration::Typed(const_ref.into()))
                }
                TyTraitItem::Type(type_ref) => Ok(ResolvedDeclaration::Typed(type_ref.into())),
            },
        }
    }

    fn resolve_symbol_helper(
        &self,
        handler: &Handler,
        symbol: &Ident,
        module: &Module,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        // Check locally declared items. Any name clash with imports will have already been reported as an error.
        if let Some(decl) = module.current_items().symbols.get(symbol) {
            return Ok(decl.clone());
        }
        // Check item imports
        if let Some((_, _, decl)) = module.current_items().use_item_synonyms.get(symbol) {
            return Ok(ResolvedDeclaration::Typed(decl.clone()));
        }
        // Check glob imports
        if let Some(decls) = module.current_items().use_glob_synonyms.get(symbol) {
            if decls.len() == 1 {
                return Ok(ResolvedDeclaration::Typed(decls[0].1.clone()));
            } else if decls.is_empty() {
                return Err(handler.emit_err(CompileError::Internal(
                    "The name {symbol} was bound in a star import, but no corresponding module paths were found",
                    symbol.span(),
                )));
            } else {
                // Symbol not found
                return Err(handler.emit_err(CompileError::SymbolWithMultipleBindings {
                    name: symbol.clone(),
                    paths: decls
                        .iter()
                        .map(|(path, decl)| {
                            let mut path_strs = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                            // Add the enum name to the path if the decl is an enum variant.
                            if let TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                enum_ref, ..
                            }) = decl
                            {
                                path_strs.push(enum_ref.name().as_str())
                            };
                            path_strs.join("::")
                        })
                        .collect(),
                    span: symbol.span(),
                }));
            }
        }
        // Symbol not found
        Err(handler.emit_err(CompileError::SymbolNotFound {
            name: symbol.clone(),
            span: symbol.span(),
        }))
    }
}

impl From<Module> for Root {
    fn from(module: Module) -> Self {
        Root { module }
    }
}

impl From<Namespace> for Root {
    fn from(namespace: Namespace) -> Self {
        namespace.root
    }
}

fn is_ancestor(src: &ModulePath, dst: &ModulePath) -> bool {
    dst.len() >= src.len() && src.iter().zip(dst).all(|(src, dst)| src == dst)
}
