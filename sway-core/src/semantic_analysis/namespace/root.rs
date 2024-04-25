use super::{
    module::Module,
    namespace::Namespace,
    trait_map::TraitMap,
    Ident, ResolvedTraitImplItem
};
use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    language::{
	CallPath,
	parsed::*,
	ty::{self, TyDecl, TyTraitItem}
    },
    namespace::ModulePath,
    TypeId, TypeInfo,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;
use sway_utils::iter_prefixes;

pub enum ResolvedDeclaration {
    Parsed(Declaration),
    Typed(ty::TyDecl),
}

impl ResolvedDeclaration {
    pub fn expect_typed(self) -> ty::TyDecl {
        match self {
            ResolvedDeclaration::Parsed(_) => panic!(),
            ResolvedDeclaration::Typed(ty_decl) => ty_decl,
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

        let decl_engine = engines.de();

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let mut symbols_and_decls = vec![];
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(decl_engine).is_public() {
                symbols_and_decls.push((symbol.clone(), decl.clone()));
            }
        }

        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines); // TODO: No difference made between imported and declared items
        for symbol_and_decl in symbols_and_decls {
            dst_mod.current_items_mut().use_glob_synonyms.insert(
                // TODO: No difference made between imported and declared items
                symbol_and_decl.0,
                (src.to_vec(), symbol_and_decl.1),
            );
        }

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

        let decl_engine = engines.de();

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;
        let mut impls_to_insert = TraitMap::default();
        match src_mod.current_items().symbols.get(item).cloned() {
            Some(decl) => {
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
                        name: item.clone(),
                        span: item.span(),
                    });
                }

                //  if this is an enum or struct or function, import its implementations
                if let Ok(type_id) = decl.return_type(&Handler::default(), engines) {
                    impls_to_insert.extend(
                        src_mod
                            .current_items()
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
                        src_mod
                            .current_items()
                            .implemented_traits
                            .filter_by_trait_decl_span(decl_span),
                        engines,
                    );
                }
                // no matter what, import it this way though.
                let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
                let add_synonym = |name| {
                    if let Some((_, _)) = dst_mod.current_items().use_item_synonyms.get(name) {
                        handler.emit_err(CompileError::ShadowsOtherSymbol { name: name.into() });
                    }
                    dst_mod.current_items_mut().use_item_synonyms.insert(
                        // TODO: No difference made between imported and declared items
                        name.clone(),
                        (src.to_vec(), decl),
                    );
                };
                match alias {
                    Some(alias) => {
                        add_synonym(&alias);
                        dst_mod
                            .current_items_mut()
                            .use_aliases
                            .insert(alias.as_str().to_string(), item.clone()); // TODO: No difference made between imported and declared items
                    }
                    None => add_synonym(item),
                };
            }
            None => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: item.clone(),
                    span: item.span(),
                }));
            }
        };

        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(impls_to_insert, engines); // TODO: No difference made between imported and declared items

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
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
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
                        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
                        let mut add_synonym = |name| {
                            if let Some((_, _)) =
                                dst_mod.current_items().use_item_synonyms.get(name)
                            {
                                handler.emit_err(CompileError::ShadowsOtherSymbol {
                                    name: name.into(),
                                });
                            }
                            dst_mod.current_items_mut().use_item_synonyms.insert(
                                // TODO: No difference made between imported and declared items
                                name.clone(),
                                (
                                    src.to_vec(),
                                    TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                        enum_ref: enum_ref.clone(),
                                        variant_name: variant_name.clone(),
                                        variant_decl_span: variant_decl.span.clone(),
                                    }),
                                ),
                            );
                        };
                        match alias {
                            Some(alias) => {
                                add_synonym(&alias);
                                dst_mod
                                    .current_items_mut()
                                    .use_aliases
                                    .insert(alias.as_str().to_string(), variant_name.clone());
                                // TODO: No difference made between imported and declared items
                            }
                            None => add_synonym(variant_name),
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
                if !decl.visibility(decl_engine).is_public() && !is_ancestor(src, dst) {
                    handler.emit_err(CompileError::ImportPrivateSymbol {
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

                    for variant_decl in enum_decl.variants.iter() {
                        let variant_name = &variant_decl.name;

                        // import it this way.
                        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
                        dst_mod.current_items_mut().use_glob_synonyms.insert(
                            // TODO: No difference made between imported and declared items
                            variant_name.clone(),
                            (
                                src.to_vec(),
                                TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
                                    enum_ref: enum_ref.clone(),
                                    variant_name: variant_name.clone(),
                                    variant_decl_span: variant_decl.span.clone(),
                                }),
                            ),
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

        let decl_engine = engines.de();

        let src_mod = self.module.lookup_submodule(handler, engines, src)?;

        let implemented_traits = src_mod.current_items().implemented_traits.clone();
        let use_item_synonyms = src_mod.current_items().use_item_synonyms.clone();
        let use_glob_synonyms = src_mod.current_items().use_glob_synonyms.clone();

        // collect all declared and reexported symbols from the source module
        let mut all_symbols_and_decls = vec![];
        for (symbol, (_, decl)) in src_mod.current_items().use_glob_synonyms.iter() {
            all_symbols_and_decls.push((symbol.clone(), decl.clone()));
        }
        for (symbol, (_, decl)) in src_mod.current_items().use_item_synonyms.iter() {
            all_symbols_and_decls.push((symbol.clone(), decl.clone()));
        }
        for (symbol, decl) in src_mod.current_items().symbols.iter() {
            if is_ancestor(src, dst) || decl.visibility(decl_engine).is_public() {
                all_symbols_and_decls.push((symbol.clone(), decl.clone()));
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

        for (symbol, (mod_path, decl)) in use_item_synonyms {
            symbols_paths_and_decls.push((symbol, get_path(mod_path), decl));
        }
        for (symbol, (mod_path, decl)) in use_glob_synonyms {
            symbols_paths_and_decls.push((symbol, get_path(mod_path), decl));
        }

        let dst_mod = self.module.lookup_submodule_mut(handler, engines, dst)?;
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(implemented_traits, engines); // TODO: No difference made between imported and declared items

        let mut try_add = |symbol, path, decl: ty::TyDecl| {
            dst_mod
                .current_items_mut()
                .use_glob_synonyms
                .insert(symbol, (path, decl));
        };

        for (symbol, decl) in all_symbols_and_decls {
            try_add(symbol, src.to_vec(), decl);
        }

        for (symbol, path, decl) in symbols_paths_and_decls {
            try_add(symbol.clone(), path, decl.clone());
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
                decl_opt = Some(
                    self.resolve_associated_type(handler, engines, &module, ident, decl, None, self_type)?,
                );
            } else {
                match module.submodules.get(ident.as_str()) {
                    Some(ns) => {
                        module = ns;
                        current_mod_path.push(ident.clone());
                    }
                    None => {
                        decl_opt = Some(self.resolve_symbol_helper(
                            handler,
                            engines,
                            ident,
                            module,
                            self_type,
                        )?);
                    }
                }
            }
        }
        if let Some(decl) = decl_opt {
            let decl =
                self.resolve_associated_item(handler, engines, &module, symbol, decl, None, self_type)?;
            return Ok((decl, current_mod_path));
        }

        self.module.lookup_submodule(handler, engines, mod_path)
            .and_then(|module| {
                let decl = self
                    .resolve_symbol_helper(handler, engines, symbol, module, self_type)?;
                Ok((decl, mod_path.to_vec()))
            })
    }

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
                ty::TyDecl::StructDecl(struct_decl) => TypeInfo::Struct(DeclRef::new(
                    struct_decl.name.clone(),
                    struct_decl.decl_id,
                    struct_decl.name.span(),
                )),
                ty::TyDecl::EnumDecl(enum_decl) => TypeInfo::Enum(DeclRef::new(
                    enum_decl.name.clone(),
                    enum_decl.decl_id,
                    enum_decl.name.span(),
                )),
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
        engines: &Engines,
        symbol: &Ident,
        module: &Module,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let true_symbol = module
            .current_items()
            .use_aliases
            .get(symbol.as_str())
            .unwrap_or(symbol);
        // Check locally declared items. Any name clash with imports will have already been reported as an error.
        if let Some(decl) = module.current_items().symbols.get(true_symbol) {
            return Ok(ResolvedDeclaration::Typed(decl.clone()));
        }
        // Check item imports
        if let Some((_, decl @ ty::TyDecl::EnumVariantDecl { .. })) =
            module.current_items().use_item_synonyms.get(symbol)
        {
            return Ok(ResolvedDeclaration::Typed(decl.clone()));
        }
        if let Some((src_path, _)) = module.current_items().use_item_synonyms.get(symbol) {
            return self.resolve_symbol(handler, engines, src_path, true_symbol, self_type);
        }
        // Check glob imports
        if let Some((_, decl)) = module.current_items().use_glob_synonyms.get(symbol) {
            return Ok(ResolvedDeclaration::Typed(decl.clone()));
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
