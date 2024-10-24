use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::{engine::DeclEngineGetParsedDeclId, DeclEngineInsert},
    language::{
        ty::{self},
        CallPath,
    },
    namespace::{ModulePath, ResolvedDeclaration},
    semantic_analysis::type_resolve::resolve_type,
    type_system::ast_elements::create_type_id::CreateTypeId,
    EnforceTypeArguments, Engines, Namespace, SubstTypes, SubstTypesContext, TypeArgument, TypeId,
    TypeInfo, TypeParameter, TypeSubstMap,
};

pub(crate) trait MonomorphizeHelper {
    fn name(&self) -> &Ident;
    fn type_parameters(&self) -> &[TypeParameter];
    fn has_self_type_param(&self) -> bool;
}

/// Given a `value` of type `T` that is able to be monomorphized and a set
/// of `type_arguments`, prepare a `TypeSubstMap` that can be used as an
/// input for monomorphization.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_type_subst_map_for_monomorphize<T>(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    value: &T,
    type_arguments: &mut [TypeArgument],
    enforce_type_arguments: EnforceTypeArguments,
    call_site_span: &Span,
    mod_path: &ModulePath,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<TypeSubstMap, ErrorEmitted>
where
    T: MonomorphizeHelper + SubstTypes,
{
    fn make_type_arity_mismatch_error(
        name: Ident,
        span: Span,
        given: usize,
        expected: usize,
    ) -> CompileError {
        match (expected, given) {
            (0, 0) => unreachable!(),
            (_, 0) => CompileError::NeedsTypeArguments { name, span },
            (0, _) => CompileError::DoesNotTakeTypeArguments { name, span },
            (_, _) => CompileError::IncorrectNumberOfTypeArguments {
                name,
                given,
                expected,
                span,
            },
        }
    }

    match (value.type_parameters().len(), type_arguments.len()) {
        (0, 0) => Ok(TypeSubstMap::default()),
        (num_type_params, 0) => {
            if let EnforceTypeArguments::Yes = enforce_type_arguments {
                return Err(handler.emit_err(make_type_arity_mismatch_error(
                    value.name().clone(),
                    call_site_span.clone(),
                    0,
                    num_type_params,
                )));
            }
            let type_mapping = TypeSubstMap::from_type_parameters(engines, value.type_parameters());
            Ok(type_mapping)
        }
        (0, num_type_args) => {
            let type_arguments_span = type_arguments
                .iter()
                .map(|x| x.span.clone())
                .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                .unwrap_or_else(|| value.name().span());
            Err(handler.emit_err(make_type_arity_mismatch_error(
                value.name().clone(),
                type_arguments_span.clone(),
                num_type_args,
                0,
            )))
        }
        (_, num_type_args) => {
            // a trait decl is passed the self type parameter and the corresponding argument
            // but it would be confusing for the user if the error reporting mechanism
            // reported the number of arguments including the implicit self, hence
            // we adjust it below
            let adjust_for_trait_decl = value.has_self_type_param() as usize;
            let non_parent_type_params = value
                .type_parameters()
                .iter()
                .filter(|x| !x.is_from_parent)
                .count()
                - adjust_for_trait_decl;

            let num_type_args = num_type_args - adjust_for_trait_decl;
            if non_parent_type_params != num_type_args {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                    .unwrap_or_else(|| value.name().span());

                return Err(handler.emit_err(make_type_arity_mismatch_error(
                    value.name().clone(),
                    type_arguments_span,
                    num_type_args,
                    non_parent_type_params,
                )));
            }

            for type_argument in type_arguments.iter_mut() {
                type_argument.type_id = resolve_type(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    type_argument.type_id,
                    &type_argument.span,
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
            let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
                value
                    .type_parameters()
                    .iter()
                    .map(|type_param| type_param.type_id)
                    .collect(),
                type_arguments
                    .iter()
                    .map(|type_arg| type_arg.type_id)
                    .collect(),
            );
            Ok(type_mapping)
        }
    }
}

/// Given a `value` of type `T` that is able to be monomorphized and a set
/// of `type_arguments`, monomorphize `value` with the `type_arguments`.
///
/// When this function is called, it is passed a `T` that is a copy of some
/// original declaration for `T` (let's denote the original with `[T]`).
/// Because monomorphization happens at application time (e.g. function
/// application), we want to be able to modify `value` such that type
/// checking the application of `value` affects only `T` and not `[T]`.
///
/// So, at a high level, this function does two things. It 1) performs the
/// necessary work to refresh the relevant generic types in `T` so that they
/// are distinct from the generics of the same name in `[T]`. And it 2)
/// applies `type_arguments` (if any are provided) to the type parameters
/// of `value`, unifying the types.
///
/// There are 4 cases that are handled in this function:
///
/// 1. `value` does not have type parameters + `type_arguments` is empty:
///     1a. return ok
/// 2. `value` has type parameters + `type_arguments` is empty:
///     2a. if the [EnforceTypeArguments::Yes] variant is provided, then
///         error
///     2b. refresh the generic types with a [TypeSubstMapping]
/// 3. `value` does have type parameters + `type_arguments` is nonempty:
///     3a. error
/// 4. `value` has type parameters + `type_arguments` is nonempty:
///     4a. check to see that the type parameters and `type_arguments` have
///         the same length
///     4b. for each type argument in `type_arguments`, resolve the type
///     4c. refresh the generic types with a [TypeSubstMapping]
#[allow(clippy::too_many_arguments)]
pub(crate) fn monomorphize_with_modpath<T>(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    value: &mut T,
    type_arguments: &mut [TypeArgument],
    enforce_type_arguments: EnforceTypeArguments,
    call_site_span: &Span,
    mod_path: &ModulePath,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<(), ErrorEmitted>
where
    T: MonomorphizeHelper + SubstTypes,
{
    let type_mapping = prepare_type_subst_map_for_monomorphize(
        handler,
        engines,
        namespace,
        value,
        type_arguments,
        enforce_type_arguments,
        call_site_span,
        mod_path,
        self_type,
        subst_ctx,
    )?;
    value.subst(&SubstTypesContext::new(engines, &type_mapping, true));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_decl_opt_to_type_id(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    type_decl_opt: Option<ResolvedDeclaration>,
    call_path: &CallPath,
    span: &Span,
    enforce_type_arguments: EnforceTypeArguments,
    mod_path: &ModulePath,
    type_arguments: Option<Vec<TypeArgument>>,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<TypeId, ErrorEmitted> {
    let decl_engine = engines.de();
    let type_engine = engines.te();
    Ok(match type_decl_opt {
        Some(ResolvedDeclaration::Typed(ty::TyDecl::StructDecl(ty::StructDecl {
            decl_id: original_id,
            ..
        }))) => {
            // get the copy from the declaration engine
            let mut new_copy = (*decl_engine.get_struct(&original_id)).clone();

            // monomorphize the copy, in place
            monomorphize_with_modpath(
                handler,
                engines,
                namespace,
                &mut new_copy,
                &mut type_arguments.unwrap_or_default(),
                enforce_type_arguments,
                span,
                mod_path,
                self_type,
                subst_ctx,
            )?;

            // insert the new copy in the decl engine
            let new_decl_ref = decl_engine.insert(
                new_copy,
                decl_engine.get_parsed_decl_id(&original_id).as_ref(),
            );

            // create the type id from the copy
            type_engine.insert(
                engines,
                TypeInfo::Struct(*new_decl_ref.id()),
                new_decl_ref.span().source_id(),
            )
        }
        Some(ResolvedDeclaration::Typed(ty::TyDecl::EnumDecl(ty::EnumDecl {
            decl_id: original_id,
            ..
        }))) => {
            // get the copy from the declaration engine
            let mut new_copy = (*decl_engine.get_enum(&original_id)).clone();

            // monomorphize the copy, in place
            monomorphize_with_modpath(
                handler,
                engines,
                namespace,
                &mut new_copy,
                &mut type_arguments.unwrap_or_default(),
                enforce_type_arguments,
                span,
                mod_path,
                self_type,
                subst_ctx,
            )?;

            // insert the new copy in the decl engine
            let new_decl_ref = decl_engine.insert(
                new_copy,
                decl_engine.get_parsed_decl_id(&original_id).as_ref(),
            );

            // create the type id from the copy
            type_engine.insert(
                engines,
                TypeInfo::Enum(*new_decl_ref.id()),
                new_decl_ref.span().source_id(),
            )
        }
        Some(ResolvedDeclaration::Typed(ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl {
            decl_id: original_id,
            ..
        }))) => {
            let new_copy = decl_engine.get_type_alias(&original_id);

            // TODO: monomorphize the copy, in place, when generic type aliases are
            // supported

            new_copy.create_type_id(engines)
        }
        Some(ResolvedDeclaration::Typed(ty::TyDecl::GenericTypeForFunctionScope(
            ty::GenericTypeForFunctionScope { type_id, .. },
        ))) => type_id,
        Some(ResolvedDeclaration::Typed(ty::TyDecl::TraitTypeDecl(ty::TraitTypeDecl {
            decl_id,
        }))) => {
            let decl_type = decl_engine.get_type(&decl_id);

            if let Some(ty) = &decl_type.ty {
                ty.type_id
            } else if let Some(implementing_type) = self_type {
                type_engine.insert(
                    engines,
                    TypeInfo::TraitType {
                        name: decl_type.name.clone(),
                        trait_type_id: implementing_type,
                    },
                    decl_type.name.span().source_id(),
                )
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Self type not provided.",
                    span.clone(),
                )));
            }
        }
        _ => {
            let err = handler.emit_err(CompileError::UnknownTypeName {
                name: call_path.to_string(),
                span: call_path.span(),
            });
            type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
        }
    })
}
