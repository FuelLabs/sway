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
    let typed_arguments: Vec<(Ident, ty::TyExpression)> = arguments
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

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = check!(
        handle_trait_constraints(
            ctx.by_ref(),
            &function_decl.type_parameters,
            &call_path.span()
        ),
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
    mut ctx: TypeCheckContext,
    type_parameters: &[TypeParameter],
    access_span: &Span,
) -> CompileResult<DeclMapping> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let mut original_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
    let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

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
                .check_if_trait_constraints_are_satisfied_for_type(
                    *type_id,
                    trait_constraints,
                    access_span
                ),
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
                    let trait_decl = check!(
                        CompileResult::from(de_get_trait(decl_id.clone(), &trait_name.span())),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    let (trait_original_method_ids, trait_method_ids, trait_impld_method_ids) = check!(
                        trait_decl.retrieve_interface_surface_and_methods_and_implemented_methods_for_type(ctx.by_ref(), *type_id, trait_name, trait_type_arguments),
                        continue,
                        warnings,
                        errors
                    );
                    original_method_ids.extend(trait_original_method_ids);
                    original_method_ids.extend(trait_method_ids);
                    impld_method_ids.extend(trait_impld_method_ids);

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
            DeclMapping::from_original_and_new_decl_ids(original_method_ids, impld_method_ids);
        ok(decl_mapping, warnings, errors)
    } else {
        err(warnings, errors)
    }
}
