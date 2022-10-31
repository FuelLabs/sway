use crate::{
    declaration_engine::{DeclMapping, DeclarationId, ReplaceDecls},
    error::*,
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::{BTreeMap, HashMap};
use sway_error::error::CompileError;
use sway_types::Spanned;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    mut ctx: TypeCheckContext,
    mut function_decl: ty::TyFunctionDeclaration,
    call_path: CallPath,
    arguments: Vec<Expression>,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(function_decl.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), function_decl.purity).to_attribute_syntax(),
            span: call_path.span(),
        });
    }

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(arguments.len(), &function_decl, &call_path),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Type check the arguments from the function application and unify them with
    // the arguments from the function application.
    let typed_arguments = arguments
        .into_iter()
        .zip(function_decl.parameters.iter())
        .map(|(arg, param)| {
            let ctx = ctx
                .by_ref()
                .with_help_text(
                    "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration.",
                )
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            let exp = check!(
                ty::TyExpression::type_check(ctx, arg.clone()),
                ty::TyExpression::error(arg.span()),
                warnings,
                errors
            );
            append!(
                unify_right(
                    exp.return_type,
                    param.type_id,
                    &exp.span,
                    "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration."
                ),
                warnings,
                errors
            );

            // check for matching mutability
            let param_mutability =
                ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
            if exp.gather_mutability().is_immutable() && param_mutability.is_mutable() {
                errors.push(CompileError::ImmutableArgumentToMutableParameter { span: arg.span() });
            }

            (param.name.clone(), exp)
        })
        .collect();

    let decl_mapping = check!(
        handle_trait_constraints(ctx.by_ref(), &function_decl.type_parameters),
        return err(warnings, errors),
        warnings,
        errors
    );
    function_decl.replace_decls(&decl_mapping);
    let return_type = function_decl.return_type;
    let span = function_decl.span.clone();
    let new_decl_id = de_insert_function(function_decl);

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params: HashMap::new(),
            arguments: typed_arguments,
            function_decl_id: new_decl_id,
            self_state_idx: None,
            selector: None,
        },
        return_type,
        span,
    };

    ok(exp, warnings, errors)
}

pub(crate) fn check_function_arguments_arity(
    arguments_len: usize,
    function_decl: &ty::TyFunctionDeclaration,
    call_path: &CallPath,
) -> CompileResult<()> {
    let warnings = vec![];
    let mut errors = vec![];
    match arguments_len.cmp(&function_decl.parameters.len()) {
        std::cmp::Ordering::Equal => ok((), warnings, errors),
        std::cmp::Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                expected: function_decl.parameters.len(),
                received: arguments_len,
            });
            err(warnings, errors)
        }
        std::cmp::Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                expected: function_decl.parameters.len(),
                received: arguments_len,
            });
            err(warnings, errors)
        }
    }
}

fn handle_trait_constraints(
    ctx: TypeCheckContext,
    type_parameters: &[TypeParameter],
) -> CompileResult<DeclMapping> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let mut original_decl_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
    let mut new_decl_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

    for type_param in type_parameters.iter() {
        let TypeParameter {
            type_id,
            trait_constraints,
            ..
        } = type_param;

        // Check to see if the trait constraints are satisfied.
        check!(
            ctx.namespace
                .implemented_traits
                .check_if_trait_constraints_are_satisfied_for_type(*type_id, trait_constraints),
            continue,
            warnings,
            errors
        );

        for trait_constraint in trait_constraints.iter() {
            let TraitConstraint {
                trait_name,
                type_arguments: trait_type_arguments,
            } = trait_constraint;

            match ctx
                .namespace
                .resolve_call_path(trait_name)
                .ok(&mut warnings, &mut errors)
                .cloned()
            {
                Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                    let ty::TyTraitDeclaration {
                        interface_surface: trait_interface_surface,
                        type_parameters: trait_type_parameters,
                        methods: trait_methods,
                        ..
                    } = check!(
                        CompileResult::from(de_get_trait(decl_id.clone(), &trait_name.span())),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
                        trait_type_parameters
                            .iter()
                            .map(|type_param| type_param.type_id)
                            .collect(),
                        trait_type_arguments
                            .iter()
                            .map(|type_arg| type_arg.type_id)
                            .collect(),
                    );

                    // Retrieve the interface surface for this trait.
                    for decl_id in trait_interface_surface.into_iter() {
                        let method = check!(
                            CompileResult::from(de_get_trait_fn(
                                decl_id.clone(),
                                &trait_name.span()
                            )),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        original_decl_ids.insert(method.name, decl_id);
                    }

                    // Retrieve the trait methods for this trait.
                    for decl_id in trait_methods.into_iter() {
                        let method = check!(
                            CompileResult::from(de_get_function(
                                decl_id.clone(),
                                &trait_name.span()
                            )),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        original_decl_ids.insert(method.name, decl_id);
                    }

                    // Retrieve the implemented methods for this trait and this
                    // type. This includes the interface surface methods and the
                    // trait methods.
                    for decl_id in ctx
                        .namespace
                        .get_methods_for_type_and_trait_name(*type_id, trait_name)
                        .into_iter()
                    {
                        let mut method = check!(
                            CompileResult::from(de_get_function(
                                decl_id.clone(),
                                &trait_name.span()
                            )),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        method.copy_types(&type_mapping);
                        new_decl_ids.insert(
                            method.name.clone(),
                            de_insert_function(method).with_parent(decl_id),
                        );
                    }

                    // TODO: handle supertraits

                    // TODO: handle the recursive case
                    //
                    // let next_decl_mapping = check!(
                    //     handle_trait_constraints(ctx.by_ref(), &trait_type_parameters),
                    //     return err(warnings, errors),
                    //     warnings,
                    //     errors
                    // );
                }
                _ => errors.push(CompileError::TraitNotFound {
                    name: trait_name.to_string(),
                    span: trait_name.span(),
                }),
            }
        }
    }

    if errors.is_empty() {
        let decl_mapping =
            DeclMapping::from_original_and_new_decl_ids(original_decl_ids, new_decl_ids);
        ok(decl_mapping, warnings, errors)
    } else {
        err(warnings, errors)
    }
}
