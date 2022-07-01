use crate::{
    error::*,
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::{hash_map::RandomState, HashMap, VecDeque};
use sway_types::{state::StateIndex, Spanned};

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    mut ctx: TypeCheckContext,
    mut function_decl: TypedFunctionDeclaration,
    call_path: CallPath<Ident>,
    type_arguments: Vec<TypeArgument>,
    arguments: Vec<Expression>,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // monomorphize the function declaration
    check!(
        ctx.monomorphize(
            &mut function_decl,
            type_arguments,
            EnforceTypeArguments::No,
            &call_path.span()
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(function_decl.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), function_decl.purity).to_attribute_syntax(),
            span: call_path.span(),
        });
    }

    check!(
        check_function_arguments_arity(arguments.len(), &function_decl, &call_path),
        return err(warnings, errors),
        warnings,
        errors
    );

    // type check arguments in function application vs arguments in function
    // declaration. Use parameter type annotations as annotations for the
    // arguments
    let typed_call_arguments = arguments
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
                .with_type_annotation(param.type_id);
            let exp = check!(
                TypedExpression::type_check(ctx, arg.clone()),
                error_recovery_expr(arg.span()),
                warnings,
                errors
            );
            (param.name.clone(), exp)
        })
        .collect();

    let span = function_decl.span.clone();
    let exp = check!(
        instantiate_function_application_inner(
            call_path,
            HashMap::new(),
            typed_call_arguments,
            function_decl,
            None,
            IsConstant::No,
            None,
            span,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(exp, warnings, errors)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application_simple(
    call_path: CallPath<Ident>,
    contract_call_params: HashMap<String, TypedExpression, RandomState>,
    arguments: VecDeque<TypedExpression>,
    function_decl: TypedFunctionDeclaration,
    selector: Option<ContractCallMetadata>,
    is_constant: IsConstant,
    self_state_idx: Option<StateIndex>,
    span: Span,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

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

    let exp = check!(
        instantiate_function_application_inner(
            call_path,
            contract_call_params,
            args_and_names,
            function_decl,
            selector,
            is_constant,
            self_state_idx,
            span,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(exp, warnings, errors)
}

fn check_function_arguments_arity(
    arguments_len: usize,
    function_decl: &TypedFunctionDeclaration,
    call_path: &CallPath<Ident>,
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
#[allow(clippy::comparison_chain)]
fn instantiate_function_application_inner(
    call_path: CallPath<Ident>,
    contract_call_params: HashMap<String, TypedExpression, RandomState>,
    arguments: Vec<(Ident, TypedExpression)>,
    function_decl: TypedFunctionDeclaration,
    selector: Option<ContractCallMetadata>,
    is_constant: IsConstant,
    self_state_idx: Option<StateIndex>,
    span: Span,
) -> CompileResult<TypedExpression> {
    let warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != function_decl.parameters.len() {
        errors.push(CompileError::Internal(
            "expected same number of function parameters and arguments",
            span,
        ));
        return err(warnings, errors);
    }

    ok(
        TypedExpression {
            expression: TypedExpressionVariant::FunctionApplication {
                call_path,
                contract_call_params,
                arguments,
                function_body: function_decl.body.clone(),
                function_body_name_span: function_decl.name.span(),
                function_body_purity: function_decl.purity,
                self_state_idx,
                selector,
            },
            return_type: function_decl.return_type,
            is_constant,
            span,
        },
        warnings,
        errors,
    )
}
