use crate::{
    decl_engine::*,
    error::*,
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::HashMap;
use sway_error::error::CompileError;
use sway_types::Spanned;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    mut ctx: TypeCheckContext,
    function_decl_ref: DeclRefFunction,
    call_path_binding: TypeBinding<CallPath>,
    arguments: Option<Vec<Expression>>,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let engines = ctx.engines();

    let fn_decl = function_decl_ref.relative_copy(engines);
    let fn_params = fn_decl.as_ref().map(|s| &s.parameters);

    if arguments.is_none() {
        errors.push(CompileError::MissingParenthesesForFunction {
            method_name: call_path_binding.inner.suffix.clone(),
            span: call_path_binding.inner.span(),
        });
        return err(warnings, errors);
    }
    let arguments = arguments.unwrap_or_default();

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(fn_decl.inner().purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), fn_decl.inner().purity).to_attribute_syntax(),
            span: call_path_binding.span(),
        });
    }

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(arguments.len(), &fn_decl, &call_path_binding.inner, false),
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

    let typed_arguments_with_names = check!(
        unify_arguments_and_parameters(ctx.by_ref(), typed_arguments, fn_params),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Retrieve the implemented traits for the type of the return type and
    // insert them in the broader namespace.
    ctx.namespace
        .insert_trait_implementation_for_type(engines, fn_decl.inner().return_type.type_id);

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path: call_path_binding.inner.clone(),
            contract_call_params: HashMap::new(),
            arguments: typed_arguments_with_names,
            fn_ref: function_decl_ref,
            self_state_idx: None,
            selector: None,
            type_binding: Some(call_path_binding.strip_inner()),
        },
        return_type: fn_decl.inner().return_type.type_id,
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
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    let typed_arguments = arguments
        .into_iter()
        .map(|arg| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, arg.clone()),
                ty::TyExpression::error(arg.span(), engines),
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
    parameters: Substituted<&Vec<ty::TyFunctionParameter>>,
) -> CompileResult<Vec<(Ident, ty::TyExpression)>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let mut typed_arguments_and_names = vec![];

    for (arg, param) in typed_arguments.into_iter().zip(parameters.iter()) {
        // unify the type of the argument with the type of the param
        check!(
            CompileResult::from(type_engine.unify(
                decl_engine,
                arg.return_type.apply_subst(&ctx),
                param.map(|p| p.type_argument.type_id),
                &arg.span,
                "The argument that has been provided to this function's type does \
            not match the declared type of the parameter in the function \
            declaration.",
                None
            )),
            continue,
            warnings,
            errors
        );

        // check for matching mutability
        let param_mutability = ty::VariableMutability::new_from_ref_mut(
            param.inner().is_reference,
            param.inner().is_mutable,
        );
        if arg.gather_mutability().is_immutable() && param_mutability.is_mutable() {
            errors.push(CompileError::ImmutableArgumentToMutableParameter {
                span: arg.span.clone(),
            });
        }

        typed_arguments_and_names.push((param.inner().name.clone(), arg));
    }

    if errors.is_empty() {
        ok(typed_arguments_and_names, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

pub(crate) fn check_function_arguments_arity(
    arguments_len: usize,
    fn_decl: &Substituted<ty::TyFunctionDecl>,
    call_path: &CallPath,
    is_method_call_syntax_used: bool,
) -> CompileResult<()> {
    let warnings = vec![];
    let mut errors = vec![];

    // if is_method_call_syntax_used then we have the guarantee
    // that at least the self argument is passed
    let (expected, received) = if is_method_call_syntax_used {
        (fn_decl.inner().parameters.len() - 1, arguments_len - 1)
    } else {
        (fn_decl.inner().parameters.len(), arguments_len)
    };
    match expected.cmp(&received) {
        std::cmp::Ordering::Equal => ok((), warnings, errors),
        std::cmp::Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: call_path.span(),
                method_name: fn_decl.inner().name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            });
            err(warnings, errors)
        }
        std::cmp::Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: call_path.span(),
                method_name: fn_decl.inner().name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            });
            err(warnings, errors)
        }
    }
}
