use crate::{
    declaration_engine::ReplaceDecls,
    error::*,
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::HashMap;
use sway_error::{error::CompileError, type_error::TypeError};
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

    let type_engine = ctx.type_engine;

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(function_decl.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), function_decl.purity).to_attribute_syntax(),
            span: call_path.span(),
        });
    }

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(arguments.len(), &function_decl, &call_path, false),
        return err(warnings, errors),
        warnings,
        errors
    );

    let typed_arguments = check!(
        type_check_arguments(ctx.by_ref(), arguments),
        return err(warnings, errors),
        warnings,
        errors
    );

    let typed_arguments = check!(
        unify_arguments_and_parameters(ctx.by_ref(), typed_arguments, &function_decl.parameters),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = check!(
        TypeParameter::gather_decl_mapping_from_trait_constraints(
            ctx.by_ref(),
            &function_decl.type_parameters,
            &call_path.span()
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    function_decl.replace_decls(&decl_mapping, type_engine);
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

/// Type checks the arguments.
fn type_check_arguments(
    mut ctx: TypeCheckContext,
    arguments: Vec<parsed::Expression>,
) -> CompileResult<Vec<ty::TyExpression>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;

    let typed_arguments = arguments
        .into_iter()
        .map(|arg| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, arg.clone()),
                ty::TyExpression::error(arg.span(), type_engine),
                warnings,
                errors
            )
        })
        .collect();

    if errors.is_empty() {
        ok(typed_arguments, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

/// Unifies the types of the arguments with the types of the parameters. Returns
/// a list of the arguments with the names of the corresponding parameters.
fn unify_arguments_and_parameters(
    ctx: TypeCheckContext,
    typed_arguments: Vec<ty::TyExpression>,
    parameters: &[ty::TyFunctionParameter],
) -> CompileResult<Vec<(Ident, ty::TyExpression)>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;

    // Type check the arguments from the function application and unify them with
    // the arguments from the function application.
    let typed_arguments: Vec<(Ident, ty::TyExpression)> = typed_arguments
        .into_iter()
        .zip(parameters.iter())
        .map(|(arg, param)| {
            let (mut new_warnings, new_errors) =
                type_engine.unify_right(arg.return_type, param.type_id, &arg.span, "");
            warnings.append(&mut new_warnings);
            if !new_errors.is_empty() {
                errors.push(CompileError::TypeError(TypeError::MismatchedType {
                    expected: type_engine.help_out(param.type_id).to_string(),
                    received: type_engine.help_out(arg.return_type).to_string(),
                    help_text: "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration."
                        .to_string(),
                    span: arg.span.clone(),
                }));
            }
            // check for matching mutability
            let param_mutability =
                ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
            if arg.gather_mutability().is_immutable() && param_mutability.is_mutable() {
                errors.push(CompileError::ImmutableArgumentToMutableParameter {
                    span: arg.span.clone(),
                });
            }
            (param.name.clone(), arg)
        })
        .collect();

    if errors.is_empty() {
        ok(typed_arguments, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

pub(crate) fn check_function_arguments_arity(
    arguments_len: usize,
    function_decl: &ty::TyFunctionDeclaration,
    call_path: &CallPath,
    is_method_call_syntax_used: bool,
) -> CompileResult<()> {
    let warnings = vec![];
    let mut errors = vec![];
    // if is_method_call_syntax_used then we have the guarantee
    // that at least the self argument is passed
    let (expected, received) = if is_method_call_syntax_used {
        (function_decl.parameters.len() - 1, arguments_len - 1)
    } else {
        (function_decl.parameters.len(), arguments_len)
    };
    match expected.cmp(&received) {
        std::cmp::Ordering::Equal => ok((), warnings, errors),
        std::cmp::Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            });
            err(warnings, errors)
        }
        std::cmp::Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            });
            err(warnings, errors)
        }
    }
}
