use crate::{
    error::*,
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::{hash_map::RandomState, HashMap, VecDeque};
use sway_error::error::CompileError;
use sway_types::{state::StateIndex, Spanned};

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    mut ctx: TypeCheckContext,
    function_decl: ty::TyFunctionDeclaration,
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

    // type check arguments in function application vs arguments in function
    // declaration. Use parameter type annotations as annotations for the
    // arguments
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

    let span = function_decl.span.clone();
    let exp = instantiate_function_application_inner(
        call_path,
        HashMap::new(),
        typed_arguments,
        function_decl,
        None,
        None,
        span,
    );

    ok(exp, warnings, errors)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application_simple(
    call_path: CallPath,
    contract_call_params: HashMap<String, ty::TyExpression, RandomState>,
    arguments: VecDeque<ty::TyExpression>,
    function_decl: ty::TyFunctionDeclaration,
    selector: Option<ty::ContractCallParams>,
    self_state_idx: Option<StateIndex>,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(arguments.len(), &function_decl, &call_path),
        return err(warnings, errors),
        warnings,
        errors
    );

    let args_and_names = function_decl
        .parameters
        .iter()
        .zip(arguments.into_iter())
        .map(|(param, arg)| (param.name.clone(), arg))
        .collect::<Vec<(_, _)>>();

    let exp = instantiate_function_application_inner(
        call_path,
        contract_call_params,
        args_and_names,
        function_decl,
        selector,
        self_state_idx,
        span,
    );
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

#[allow(clippy::too_many_arguments)]
fn instantiate_function_application_inner(
    call_path: CallPath,
    contract_call_params: HashMap<String, ty::TyExpression, RandomState>,
    arguments: Vec<(Ident, ty::TyExpression)>,
    function_decl: ty::TyFunctionDeclaration,
    selector: Option<ty::ContractCallParams>,
    self_state_idx: Option<StateIndex>,
    span: Span,
) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params,
            arguments,
            function_decl_id: de_insert_function(function_decl.clone()),
            self_state_idx,
            selector,
        },
        return_type: function_decl.return_type,
        span,
    }
}
