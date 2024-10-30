use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};
use sway_utils::iter_prefixes;

use crate::{
    language::{
        ty::{self, TyTraitItem},
        CallPath, QualifiedCallPath,
    },
    monomorphization::type_decl_opt_to_type_id,
    namespace::{Module, ModulePath, ResolvedDeclaration, ResolvedTraitImplItem},
    type_system::SubstTypes,
    EnforceTypeArguments, Engines, Namespace, SubstTypesContext, TypeId, TypeInfo,
};

/// Resolve the type of the given [TypeId], replacing any instances of
/// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
/// enum, or a reference to a type parameter.
#[allow(clippy::too_many_arguments)]
pub fn resolve_type(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    type_id: TypeId,
    span: &Span,
    enforce_type_arguments: EnforceTypeArguments,
    type_info_prefix: Option<&ModulePath>,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<TypeId, ErrorEmitted> {
    let type_engine = engines.te();
    let module_path = type_info_prefix.unwrap_or(mod_path);
    let type_id = match (*type_engine.get(type_id)).clone() {
        TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        } => {
            let type_decl_opt = resolve_qualified_call_path(
                handler,
                engines,
                namespace,
                module_path,
                &qualified_call_path,
                self_type,
                subst_ctx,
            )
            .ok();
            type_decl_opt_to_type_id(
                handler,
                engines,
                namespace,
                type_decl_opt,
                &qualified_call_path.call_path,
                span,
                enforce_type_arguments,
                mod_path,
                type_arguments.clone(),
                self_type,
                subst_ctx,
            )?
        }
        TypeInfo::Array(mut elem_ty, n) => {
            elem_ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id,
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
            )
            .unwrap_or_else(|err| {
                engines
                    .te()
                    .insert(engines, TypeInfo::ErrorRecovery(err), None)
            });

            engines.te().insert(
                engines,
                TypeInfo::Array(elem_ty.clone(), n.clone()),
                elem_ty.span.source_id(),
            )
        }
        TypeInfo::Slice(mut elem_ty) => {
            elem_ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id,
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
            )
            .unwrap_or_else(|err| {
                engines
                    .te()
                    .insert(engines, TypeInfo::ErrorRecovery(err), None)
            });

            engines.te().insert(
                engines,
                TypeInfo::Slice(elem_ty.clone()),
                elem_ty.span.source_id(),
            )
        }
        TypeInfo::Tuple(mut type_arguments) => {
            for type_argument in type_arguments.iter_mut() {
                type_argument.type_id = resolve_type(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    type_argument.type_id,
                    span,
                    enforce_type_arguments,
                    None,
                    self_type,
                    subst_ctx,
                )
                .unwrap_or_else(|err| {
                    engines
                        .te()
                        .insert(engines, TypeInfo::ErrorRecovery(err), None)
                });
            }

            engines
                .te()
                .insert(engines, TypeInfo::Tuple(type_arguments), span.source_id())
        }
        TypeInfo::TraitType {
            name,
            trait_type_id,
        } => {
            let trait_item_ref = namespace
                .root
                .module
                .current_items()
                .implemented_traits
                .get_trait_item_for_type(handler, engines, &name, trait_type_id, None)?;

            if let ResolvedTraitImplItem::Typed(TyTraitItem::Type(type_ref)) = trait_item_ref {
                let type_decl = engines.de().get_type(type_ref.id());
                if let Some(ty) = &type_decl.ty {
                    ty.type_id
                } else {
                    type_id
                }
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Expecting associated type",
                    trait_item_ref.span(engines),
                )));
            }
        }
        TypeInfo::Ref {
            referenced_type: mut ty,
            to_mutable_value,
        } => {
            ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                ty.type_id,
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
            )
            .unwrap_or_else(|err| {
                engines
                    .te()
                    .insert(engines, TypeInfo::ErrorRecovery(err), None)
            });

            engines.te().insert(
                engines,
                TypeInfo::Ref {
                    to_mutable_value,
                    referenced_type: ty.clone(),
                },
                None,
            )
        }
        _ => type_id,
    };

    let mut type_id = type_id;
    type_id.subst(subst_ctx);

    Ok(type_id)
}

pub fn resolve_qualified_call_path(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    qualified_call_path: &QualifiedCallPath,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let type_engine = engines.te();
    if let Some(qualified_path_root) = qualified_call_path.clone().qualified_path_root {
        let root_type_id = match &&*type_engine.get(qualified_path_root.ty.type_id) {
            TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
                ..
            } => {
                let type_decl = resolve_call_path(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    &qualified_call_path.clone().to_call_path(handler)?,
                    self_type,
                )?;
                type_decl_opt_to_type_id(
                    handler,
                    engines,
                    namespace,
                    Some(type_decl),
                    &qualified_call_path.call_path,
                    &qualified_path_root.ty.span(),
                    EnforceTypeArguments::No,
                    mod_path,
                    type_arguments.clone(),
                    self_type,
                    subst_ctx,
                )?
            }
            _ => qualified_path_root.ty.type_id,
        };

        let as_trait_opt = match &&*type_engine.get(qualified_path_root.as_trait) {
            TypeInfo::Custom {
                qualified_call_path: call_path,
                ..
            } => Some(
                call_path
                    .clone()
                    .to_call_path(handler)?
                    .to_fullpath(engines, namespace),
            ),
            _ => None,
        };

        resolve_call_path_and_root_type_id(
            handler,
            engines,
            &namespace.root.module,
            root_type_id,
            as_trait_opt,
            &qualified_call_path.call_path,
            self_type,
        )
    } else {
        resolve_call_path(
            handler,
            engines,
            namespace,
            mod_path,
            &qualified_call_path.call_path,
            self_type,
        )
    }
}

/// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
///
/// This will concatenate the `mod_path` with the `call_path`'s prefixes and
/// then calling `resolve_symbol` with the resulting path and call_path's suffix.
///
/// The `mod_path` is significant here as we assume the resolution is done within the
/// context of the module pointed to by `mod_path` and will only check the call path prefixes
/// and the symbol's own visibility.
pub fn resolve_call_path(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    call_path: &CallPath,
    self_type: Option<TypeId>,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let (decl, mod_path) = namespace
        .root
        .resolve_call_path_and_mod_path(handler, engines, mod_path, call_path, self_type)?;

    // In case there is no mod path we don't need to check visibility
    if mod_path.is_empty() {
        return Ok(decl);
    }

    // In case there are no prefixes we don't need to check visibility
    if call_path.prefixes.is_empty() {
        return Ok(decl);
    }

    // check the visibility of the call path elements
    // we don't check the first prefix because direct children are always accessible
    for prefix in iter_prefixes(&call_path.prefixes).skip(1) {
        let module = namespace.lookup_submodule_from_absolute_path(handler, engines, prefix)?;
        if module.visibility().is_private() {
            let prefix_last = prefix[prefix.len() - 1].clone();
            handler.emit_err(CompileError::ImportPrivateModule {
                span: prefix_last.span(),
                name: prefix_last,
            });
        }
    }

    // check the visibility of the symbol itself
    if !decl.visibility(engines).is_public() {
        handler.emit_err(CompileError::ImportPrivateSymbol {
            name: call_path.suffix.clone(),
            span: call_path.suffix.span(),
        });
    }

    Ok(decl)
}

pub fn decl_to_type_info(
    handler: &Handler,
    engines: &Engines,
    symbol: &Ident,
    decl: ResolvedDeclaration,
) -> Result<TypeInfo, ErrorEmitted> {
    match decl {
        ResolvedDeclaration::Parsed(_decl) => todo!(),
        ResolvedDeclaration::Typed(decl) => Ok(match decl.clone() {
            ty::TyDecl::StructDecl(struct_ty_decl) => TypeInfo::Struct(struct_ty_decl.decl_id),
            ty::TyDecl::EnumDecl(enum_ty_decl) => TypeInfo::Enum(enum_ty_decl.decl_id),
            ty::TyDecl::TraitTypeDecl(type_decl) => {
                let type_decl = engines.de().get_type(&type_decl.decl_id);
                if type_decl.ty.is_none() {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Trait type declaration has no type",
                        symbol.span(),
                    )));
                }
                (*engines.te().get(type_decl.ty.clone().unwrap().type_id)).clone()
            }
            ty::TyDecl::GenericTypeForFunctionScope(decl) => {
                (*engines.te().get(decl.type_id)).clone()
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
pub fn resolve_associated_item_from_type_id(
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
            TyTraitItem::Constant(const_ref) => Ok(ResolvedDeclaration::Typed(const_ref.into())),
            TyTraitItem::Type(type_ref) => Ok(ResolvedDeclaration::Typed(type_ref.into())),
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_associated_type(
    handler: &Handler,
    engines: &Engines,
    module: &Module,
    symbol: &Ident,
    decl: ResolvedDeclaration,
    as_trait: Option<CallPath>,
    self_type: Option<TypeId>,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let type_info = decl_to_type_info(handler, engines, symbol, decl)?;
    let type_id = engines
        .te()
        .insert(engines, type_info, symbol.span().source_id());

    resolve_associated_item_from_type_id(
        handler, engines, module, symbol, type_id, as_trait, self_type,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_associated_item(
    handler: &Handler,
    engines: &Engines,
    module: &Module,
    symbol: &Ident,
    decl: ResolvedDeclaration,
    as_trait: Option<CallPath>,
    self_type: Option<TypeId>,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let type_info = decl_to_type_info(handler, engines, symbol, decl)?;
    let type_id = engines
        .te()
        .insert(engines, type_info, symbol.span().source_id());

    resolve_associated_item_from_type_id(
        handler, engines, module, symbol, type_id, as_trait, self_type,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_call_path_and_root_type_id(
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
            decl_opt = Some(resolve_associated_item_from_type_id(
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
            decl_opt = Some(resolve_associated_type(
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
        let decl = resolve_associated_item_from_type_id(
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
        let decl = resolve_associated_item(
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
