use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;

use crate::{
    decl_engine::DeclRef,
    language::{
        ty::{self, TyTraitItem},
        CallPath,
    },
    Engines, Ident, TypeId, TypeInfo,
};

use super::{module::Module, namespace::Namespace, Path};

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
    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let (decl, _) =
            self.resolve_call_path_and_mod_path(handler, engines, mod_path, call_path, self_type)?;
        Ok(decl)
    }

    pub(crate) fn resolve_call_path_and_mod_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<(ty::TyDecl, Vec<Ident>), ErrorEmitted> {
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
        root_type_id: TypeId,
        mut as_trait: Option<CallPath>,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        // This block tries to resolve associated types
        let mut decl_opt = None;
        let mut type_id_opt = Some(root_type_id);
        for ident in call_path.prefixes.iter() {
            if let Some(type_id) = type_id_opt {
                type_id_opt = None;
                decl_opt = Some(self.resolve_associated_type_from_type_id(
                    handler,
                    engines,
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
        mod_path: &Path,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let (decl, _) =
            self.resolve_symbol_and_mod_path(handler, engines, mod_path, symbol, self_type)?;
        Ok(decl)
    }

    fn resolve_symbol_and_mod_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<(ty::TyDecl, Vec<Ident>), ErrorEmitted> {
        // This block tries to resolve associated types
        let mut module = &self.module;
        let mut current_mod_path = vec![];
        let mut decl_opt = None;
        for ident in mod_path.iter() {
            if let Some(decl) = decl_opt {
                decl_opt = Some(
                    self.resolve_associated_type(handler, engines, ident, decl, None, self_type)?,
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
                            &current_mod_path,
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
                self.resolve_associated_item(handler, engines, symbol, decl, None, self_type)?;
            return Ok((decl, current_mod_path));
        }

        self.check_submodule(handler, mod_path).and_then(|module| {
            let decl =
                self.resolve_symbol_helper(handler, engines, mod_path, symbol, module, self_type)?;
            Ok((decl, mod_path.to_vec()))
        })
    }

    fn resolve_associated_type(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        decl: ty::TyDecl,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let type_info = self.decl_to_type_info(handler, engines, symbol, decl)?;

        self.resolve_associated_type_from_type_id(
            handler,
            engines,
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
        symbol: &Ident,
        decl: ty::TyDecl,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let type_info = self.decl_to_type_info(handler, engines, symbol, decl)?;

        self.resolve_associated_item_from_type_id(
            handler,
            engines,
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
        decl: ty::TyDecl,
    ) -> Result<TypeInfo, ErrorEmitted> {
        Ok(match decl.clone() {
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
                dbg!(1);
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: symbol.clone(),
                    span: symbol.span(),
                }))
            }
        })
    }

    fn resolve_associated_type_from_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let item_decl = self.resolve_associated_item_from_type_id(
            handler, engines, symbol, type_id, as_trait, self_type,
        )?;
        if !matches!(item_decl, ty::TyDecl::TraitTypeDecl(_)) {
            return Err(handler.emit_err(CompileError::Internal(
                "Expecting associated type",
                item_decl.span(),
            )));
        }
        Ok(item_decl)
    }

    fn resolve_associated_item_from_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
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
        let item_ref = self
            .implemented_traits
            .get_trait_item_for_type(handler, engines, symbol, type_id, as_trait)?;
        match item_ref {
            TyTraitItem::Fn(fn_ref) => Ok(fn_ref.into()),
            TyTraitItem::Constant(const_ref) => Ok(const_ref.into()),
            TyTraitItem::Type(type_ref) => Ok(type_ref.into()),
        }
    }

    fn resolve_symbol_helper(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        symbol: &Ident,
        module: &Module,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let true_symbol = self[mod_path]
            .use_aliases
            .get(symbol.as_str())
            .unwrap_or(symbol);
        match module.use_synonyms.get(symbol) {
            Some((_, _, decl @ ty::TyDecl::EnumVariantDecl { .. }, _)) => Ok(decl.clone()),
            Some((src_path, _, _, _)) if mod_path != src_path => {
                // If the symbol is imported, before resolving to it,
                // we need to check if there is a local symbol withing the module with
                // the same name, and if yes resolve to the local symbol, because it
                // shadows the import.
                // Note that we can have two situations here:
                // - glob-import, in which case the local symbol simply shadows the glob-imported one.
                // - non-glob import, in which case we will already have a name clash reported
                //   as an error, but still have to resolve to the local module symbol
                //   if it exists.
                match module.symbols.get(true_symbol) {
                    Some(decl) => Ok(decl.clone()),
                    None => self.resolve_symbol(handler, engines, src_path, true_symbol, self_type),
                }
            }
            _ => {
                // dbg!(
                //     mod_path,
                //     symbol,
                //     module,
                //     self_type,
                // );
                module
                    .check_symbol(true_symbol)
                    .map_err(|e| handler.emit_err(e))
                    .cloned()
            },
        }
    }
}

impl std::ops::Deref for Root {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl std::ops::DerefMut for Root {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
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
