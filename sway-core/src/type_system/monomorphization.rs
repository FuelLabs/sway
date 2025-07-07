use crate::{
    decl_engine::{engine::DeclEngineGetParsedDeclId, DeclEngineInsert, MaterializeConstGenerics},
    language::{
        ty::{self, TyExpression},
        CallPath,
    },
    namespace::{ModulePath, ResolvedDeclaration},
    semantic_analysis::type_resolve::{resolve_type, VisibilityCheck},
    type_system::ast_elements::create_type_id::CreateTypeId,
    EnforceTypeArguments, Engines, GenericArgument, Namespace, SubstTypes, SubstTypesContext,
    TypeId, TypeParameter, TypeSubstMap,
};
use std::collections::BTreeMap;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

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
    type_arguments: &mut [GenericArgument],
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
                .map(|x| x.span().clone())
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
                .filter(|x| !x.is_from_parent())
                .count()
                - adjust_for_trait_decl;

            let num_type_args = num_type_args - adjust_for_trait_decl;
            if non_parent_type_params != num_type_args {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span())
                    .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                    .unwrap_or_else(|| value.name().span());

                return Err(handler.emit_err(make_type_arity_mismatch_error(
                    value.name().clone(),
                    type_arguments_span,
                    num_type_args,
                    non_parent_type_params,
                )));
            }

            let params = value.type_parameters();
            let args = type_arguments.iter_mut();

            for (param, arg) in params.iter().zip(args) {
                match (param, arg) {
                    (TypeParameter::Type(_), GenericArgument::Type(arg)) => {
                        arg.type_id = resolve_type(
                            handler,
                            engines,
                            namespace,
                            mod_path,
                            arg.type_id,
                            &arg.span,
                            enforce_type_arguments,
                            None,
                            self_type,
                            subst_ctx,
                            VisibilityCheck::Yes,
                        )
                        .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));
                    }
                    (TypeParameter::Const(_), GenericArgument::Type(arg)) => {
                        let suffix = arg
                            .call_path_tree
                            .as_ref()
                            .unwrap()
                            .qualified_call_path
                            .call_path
                            .suffix
                            .clone();
                        let _ = crate::semantic_analysis::type_resolve::resolve_call_path(
                            handler,
                            engines,
                            namespace,
                            mod_path,
                            &CallPath {
                                prefixes: vec![],
                                suffix,
                                callpath_type: crate::language::CallPathType::Ambiguous,
                            },
                            self_type,
                            VisibilityCheck::No,
                        )
                        .map(|d| d.expect_typed())?;
                    }
                    (_, GenericArgument::Const(arg)) => {
                        match &arg.expr {
                            crate::ast_elements::type_parameter::ConstGenericExpr::Literal { ..} => {},
                            crate::ast_elements::type_parameter::ConstGenericExpr::AmbiguousVariableExpression { ident } => {
                                let _ = crate::semantic_analysis::type_resolve::resolve_call_path(
                                    handler,
                                    engines,
                                    namespace,
                                    mod_path,
                                    &CallPath {
                                        prefixes: vec![],
                                        suffix: ident.clone(),
                                        callpath_type: crate::language::CallPathType::Ambiguous,
                                    },
                                    self_type,
                                    VisibilityCheck::No,
                                )
                                .map(|d| d.expect_typed())?;
                            },
                        }
                    }
                }
            }

            let mut params = vec![];
            let mut args = vec![];
            let mut consts = BTreeMap::new();
            for (p, a) in value.type_parameters().iter().zip(type_arguments.iter()) {
                match (p, a) {
                    (TypeParameter::Type(p), GenericArgument::Type(a)) => {
                        params.push(p.type_id);
                        args.push(a.type_id);
                    }
                    (TypeParameter::Const(p), GenericArgument::Const(a)) => {
                        consts.insert(
                            p.name.as_str().to_string(),
                            a.expr.to_ty_expression(engines),
                        );
                    }
                    // TODO const generic was not materialized yet
                    (TypeParameter::Const(_), GenericArgument::Type(_)) => {}
                    x => todo!("{x:?}"),
                }
            }

            Ok(
                TypeSubstMap::from_type_parameters_and_type_arguments_and_const_generics(
                    params, args, consts,
                ),
            )
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
///    1a. return ok
/// 2. `value` has type parameters + `type_arguments` is empty:
///    2a. if the [EnforceTypeArguments::Yes] variant is provided, then
///    error
///    2b. refresh the generic types with a [TypeSubstMapping]
/// 3. `value` does have type parameters + `type_arguments` is nonempty:
///    3a. error
/// 4. `value` has type parameters + `type_arguments` is nonempty:
///    4a. check to see that the type parameters and `type_arguments` have
///    the same length
///    4b. for each type argument in `type_arguments`, resolve the type
///    4c. refresh the generic types with a [TypeSubstMapping]
#[allow(clippy::too_many_arguments)]
pub(crate) fn monomorphize_with_modpath<T>(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    value: &mut T,
    type_arguments: &mut [GenericArgument],
    const_generics: BTreeMap<String, TyExpression>,
    enforce_type_arguments: EnforceTypeArguments,
    call_site_span: &Span,
    mod_path: &ModulePath,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
) -> Result<(), ErrorEmitted>
where
    T: MonomorphizeHelper + SubstTypes + MaterializeConstGenerics,
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

    for (name, expr) in const_generics.iter() {
        let _ = value.materialize_const_generics(engines, handler, name, expr);
    }

    for (name, expr) in type_mapping.const_generics_materialization.iter() {
        let _ = value.materialize_const_generics(engines, handler, name, expr);
    }

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
    type_arguments: Option<Vec<GenericArgument>>,
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
                BTreeMap::new(),
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
            type_engine.insert_struct(engines, *new_decl_ref.id())
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
                BTreeMap::new(),
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
            type_engine.insert_enum(engines, *new_decl_ref.id())
        }
        Some(ResolvedDeclaration::Typed(ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl {
            decl_id: original_id,
            ..
        }))) => {
            let new_copy = decl_engine.get_type_alias(&original_id);

            // TODO: (GENERIC-TYPE-ALIASES) Monomorphize the copy, in place, once generic type aliases are
            //       supported.

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
                ty.type_id()
            } else if let Some(implementing_type) = self_type {
                type_engine.insert_trait_type(engines, decl_type.name.clone(), implementing_type)
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Self type not provided.",
                    span.clone(),
                )));
            }
        }
        Some(ResolvedDeclaration::Typed(ty::TyDecl::ConstGenericDecl(_))) => {
            return Ok(engines.te().id_of_u64())
        }
        _ => {
            let err = handler.emit_err(CompileError::UnknownTypeName {
                name: call_path.to_string(),
                span: call_path.span(),
            });
            type_engine.id_of_error_recovery(err)
        }
    })
}
