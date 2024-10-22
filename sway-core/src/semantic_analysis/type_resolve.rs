use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Span, Spanned};
use sway_utils::iter_prefixes;

use crate::{
    language::{
        ty::{self, TyTraitItem},
        CallPath, QualifiedCallPath,
    },
    monomorphization::type_decl_opt_to_type_id,
    namespace::{ModulePath, ResolvedTraitImplItem},
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
            root_type_id,
        } => {
            let type_decl_opt = if let Some(root_type_id) = root_type_id {
                namespace
                    .root()
                    .resolve_call_path_and_root_type_id(
                        handler,
                        engines,
                        namespace.module(engines),
                        root_type_id,
                        None,
                        &qualified_call_path.clone().to_call_path(handler)?,
                        self_type,
                    )
                    .map(|decl| decl.expect_typed())
                    .ok()
            } else {
                resolve_qualified_call_path(
                    handler,
                    engines,
                    namespace,
                    module_path,
                    &qualified_call_path,
                    self_type,
                    subst_ctx,
                )
                .ok()
            };
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
        TypeInfo::Array(mut elem_ty, length) => {
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
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines
                .te()
                .insert_array(engines, elem_ty, length)
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
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines.te().insert_slice(engines, elem_ty)
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
                .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));
            }

            engines.te().insert_tuple(engines, type_arguments)
        }
        TypeInfo::TraitType {
            name,
            trait_type_id,
        } => {
            let item_ref = namespace.get_root_trait_item_for_type(
                handler,
                engines,
                &name,
                trait_type_id,
                None,
            )?;
            if let ResolvedTraitImplItem::Typed(TyTraitItem::Type(type_ref)) = item_ref {
                let type_decl = engines.de().get_type(type_ref.id());
                if let Some(ty) = &type_decl.ty {
                    ty.type_id
                } else {
                    type_id
                }
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Expecting associated type",
                    item_ref.span(engines),
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
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines
                .te()
                .insert_ref(engines, to_mutable_value, ty)
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
) -> Result<ty::TyDecl, ErrorEmitted> {
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

        namespace
            .root
            .resolve_call_path_and_root_type_id(
                handler,
                engines,
                &namespace.root.module,
                root_type_id,
                as_trait_opt,
                &qualified_call_path.call_path,
                self_type,
            )
            .map(|decl| decl.expect_typed())
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
) -> Result<ty::TyDecl, ErrorEmitted> {
    let (decl, mod_path) = namespace
        .root
        .resolve_call_path_and_mod_path(handler, engines, mod_path, call_path, self_type)?;
    let decl = decl.expect_typed();

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
    if !decl.visibility(engines.de()).is_public() {
        handler.emit_err(CompileError::ImportPrivateSymbol {
            name: call_path.suffix.clone(),
            span: call_path.suffix.span(),
        });
    }

    Ok(decl)
}
