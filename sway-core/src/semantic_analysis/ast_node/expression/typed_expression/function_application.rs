use crate::{
    decl_engine::{DeclEngineInsert, DeclRefFunction, ReplaceDecls},
    language::{ty, *},
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use std::collections::HashMap;
use sway_error::error::CompileError;
use sway_types::Spanned;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    function_decl_ref: DeclRefFunction,
    call_path_binding: TypeBinding<CallPath>,
    arguments: Option<Vec<Expression>>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    let mut function_decl = decl_engine.get_function(&function_decl_ref);

    if arguments.is_none() {
        return Err(
            handler.emit_err(CompileError::MissingParenthesesForFunction {
                method_name: call_path_binding.inner.suffix.clone(),
                span: call_path_binding.inner.span(),
            }),
        );
    }
    let arguments = arguments.unwrap_or_default();

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !ctx.purity().can_call(function_decl.purity) {
        handler.emit_err(CompileError::StorageAccessMismatch {
            attrs: promote_purity(ctx.purity(), function_decl.purity).to_attribute_syntax(),
            span: call_path_binding.span(),
        });
    }

    // check that the number of parameters and the number of the arguments is the same
    check_function_arguments_arity(
        handler,
        arguments.len(),
        &function_decl,
        &call_path_binding.inner,
        false,
    )?;

    let typed_arguments = type_check_arguments(handler, ctx.by_ref(), arguments)?;

    let typed_arguments_with_names = unify_arguments_and_parameters(
        handler,
        ctx.by_ref(),
        typed_arguments,
        &function_decl.parameters,
    )?;

    // Retrieve the implemented traits for the type of the return type and
    // insert them in the broader namespace.
    ctx.namespace
        .insert_trait_implementation_for_type(engines, function_decl.return_type.type_id);

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = TypeParameter::gather_decl_mapping_from_trait_constraints(
        handler,
        ctx.by_ref(),
        &function_decl.type_parameters,
        &call_path_binding.span(),
    )?;
    function_decl.replace_decls(&decl_mapping, engines);
    let return_type = function_decl.return_type.clone();
    let new_decl_ref = decl_engine
        .insert(function_decl)
        .with_parent(decl_engine, (*function_decl_ref.id()).into());

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path: call_path_binding.inner.clone(),
            contract_call_params: HashMap::new(),
            arguments: typed_arguments_with_names,
            fn_ref: new_decl_ref,
            selector: None,
            type_binding: Some(call_path_binding.strip_inner()),
            call_path_typeid: None,
        },
        return_type: return_type.type_id,
        span,
    };

    Ok(exp)
}

/// Type checks the arguments.
fn type_check_arguments(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    arguments: Vec<parsed::Expression>,
) -> Result<Vec<ty::TyExpression>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut error_emitted = None;

    let typed_arguments = arguments
        .into_iter()
        .map(|arg| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, arg.clone()).unwrap_or_else(|err| {
                error_emitted = Some(err);
                ty::TyExpression::error(arg.span(), engines)
            })
        })
        .collect();

    if let Some(err) = error_emitted {
        Err(err)
    } else {
        Ok(typed_arguments)
    }
}

/// Unifies the types of the arguments with the types of the parameters. Returns
/// a list of the arguments with the names of the corresponding parameters.
fn unify_arguments_and_parameters(
    handler: &Handler,
    ctx: TypeCheckContext,
    typed_arguments: Vec<ty::TyExpression>,
    parameters: &[ty::TyFunctionParameter],
) -> Result<Vec<(Ident, ty::TyExpression)>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();
    let mut typed_arguments_and_names = vec![];

    let mut error_emitted = None;

    for (arg, param) in typed_arguments.into_iter().zip(parameters.iter()) {
        // unify the type of the argument with the type of the param

        let (warnings, errors) = type_engine.unify(
            engines,
            arg.return_type,
            param.type_argument.type_id,
            &arg.span,
            "The argument that has been provided to this function's type does \
            not match the declared type of the parameter in the function \
            declaration.",
            None,
        );
        for warn in warnings {
            handler.emit_warn(warn);
        }
        for err in errors.clone() {
            error_emitted = Some(handler.emit_err(err));
        }
        if !errors.is_empty() {
            continue;
        }

        // check for matching mutability
        let param_mutability =
            ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
        if arg.gather_mutability().is_immutable() && param_mutability.is_mutable() {
            error_emitted = Some(handler.emit_err(
                CompileError::ImmutableArgumentToMutableParameter {
                    span: arg.span.clone(),
                },
            ));
        }

        typed_arguments_and_names.push((param.name.clone(), arg));
    }

    if let Some(err) = error_emitted {
        Err(err)
    } else {
        Ok(typed_arguments_and_names)
    }
}

pub(crate) fn check_function_arguments_arity(
    handler: &Handler,
    arguments_len: usize,
    function_decl: &ty::TyFunctionDecl,
    call_path: &CallPath,
    is_method_call_syntax_used: bool,
) -> Result<(), ErrorEmitted> {
    // if is_method_call_syntax_used then we have the guarantee
    // that at least the self argument is passed
    let (expected, received) = if is_method_call_syntax_used {
        (function_decl.parameters.len() - 1, arguments_len - 1)
    } else {
        (function_decl.parameters.len(), arguments_len)
    };
    match expected.cmp(&received) {
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Less => {
            Err(handler.emit_err(CompileError::TooFewArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            }))
        }
        std::cmp::Ordering::Greater => {
            Err(handler.emit_err(CompileError::TooManyArgumentsForFunction {
                span: call_path.span(),
                method_name: function_decl.name.clone(),
                dot_syntax_used: is_method_call_syntax_used,
                expected,
                received,
            }))
        }
    }
}
