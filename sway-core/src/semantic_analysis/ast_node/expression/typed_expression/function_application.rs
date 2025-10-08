use crate::{
    decl_engine::{
        engine::DeclEngineGetParsedDeclId, DeclEngineInsert, DeclRefFunction, ReplaceDecls,
    },
    language::{
        ty::{self, TyFunctionDecl, TyFunctionSig},
        *,
    },
    semantic_analysis::{ast_node::*, TypeCheckContext},
};
use ast_elements::type_parameter::GenericTypeParameter;
use indexmap::IndexMap;
use sway_error::error::CompileError;
use sway_types::{IdentUnique, Spanned};

const UNIFY_ARGS_HELP_TEXT: &str =
    "The argument that has been provided to this function's type does \
not match the declared type of the parameter in the function \
declaration.";

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    function_decl_ref: DeclRefFunction,
    call_path_binding: TypeBinding<CallPath>,
    arguments: Option<&[Expression]>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let engines = ctx.engines();
    let decl_engine = engines.de();

    if arguments.is_none() {
        return Err(
            handler.emit_err(CompileError::MissingParenthesesForFunction {
                method_name: call_path_binding.inner.suffix.clone(),
                span: call_path_binding.inner.span(),
            }),
        );
    }

    let function_decl = decl_engine.get_function(&function_decl_ref);
    let arguments = arguments.unwrap_or_default();

    // check that the number of parameters and the number of the arguments is the same
    check_function_arguments_arity(
        handler,
        arguments.len(),
        &function_decl,
        &call_path_binding.inner,
        false,
    )?;

    let typed_arguments =
        type_check_arguments(handler, ctx.by_ref(), arguments, &function_decl.parameters)?;
    let typed_arguments_with_names = unify_arguments_and_parameters(
        handler,
        ctx.by_ref(),
        typed_arguments,
        &function_decl.parameters,
    )?;

    // unify function return type with current ctx.type_annotation().
    engines.te().unify_with_generic(
        handler,
        engines,
        function_decl.return_type.type_id,
        ctx.type_annotation(),
        &call_path_binding.span(),
        "Function return type does not match up with local type annotation.",
        || None,
    );

    let mut function_return_type_id = function_decl.return_type.type_id;

    let function_ident: IdentUnique = function_decl.name.clone().into();
    let function_sig = TyFunctionSig::from_fn_decl(&function_decl);

    let new_decl_ref = if let Some(cached_fn_ref) =
        ctx.engines()
            .qe()
            .get_function(engines, &function_ident, function_sig.clone())
    {
        cached_fn_ref
    } else {
        let mut function_decl = TyFunctionDecl::clone(&*function_decl);

        if !ctx.code_block_first_pass() {
            // Handle the trait constraints. This includes checking to see if the trait
            // constraints are satisfied and replacing old decl ids based on the
            // constraint with new decl ids based on the new type.
            let decl_mapping = GenericTypeParameter::gather_decl_mapping_from_trait_constraints(
                handler,
                ctx.by_ref(),
                &function_decl.type_parameters,
                function_decl.name.as_str(),
                &call_path_binding.span(),
            )?;
            function_decl.replace_decls(&decl_mapping, handler, &mut ctx)?;
        }

        let method_sig = TyFunctionSig::from_fn_decl(&function_decl);

        function_return_type_id = function_decl.return_type.type_id;
        let function_is_type_check_finalized = function_decl.is_type_check_finalized;
        let function_is_trait_method_dummy = function_decl.is_trait_method_dummy;
        let new_decl_ref = decl_engine
            .insert(
                function_decl,
                decl_engine
                    .get_parsed_decl_id(function_decl_ref.id())
                    .as_ref(),
            )
            .with_parent(decl_engine, (*function_decl_ref.id()).into());

        if !ctx.code_block_first_pass()
            && method_sig.is_concrete(engines)
            && function_is_type_check_finalized
            && !function_is_trait_method_dummy
        {
            ctx.engines().qe().insert_function(
                engines,
                function_ident,
                method_sig,
                new_decl_ref.clone(),
            );
        }

        new_decl_ref
    };

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path: call_path_binding.inner.clone(),
            arguments: typed_arguments_with_names,
            fn_ref: new_decl_ref,
            selector: None,
            type_binding: Some(call_path_binding.strip_inner()),
            method_target: None,
            contract_call_params: IndexMap::new(),
            contract_caller: None,
        },
        return_type: function_return_type_id,
        span,
    };

    Ok(exp)
}

/// Type checks the arguments.
fn type_check_arguments(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    arguments: &[parsed::Expression],
    parameters: &[ty::TyFunctionParameter],
) -> Result<Vec<ty::TyExpression>, ErrorEmitted> {
    let engines = ctx.engines();

    // Sanity check before zipping arguments and parameters
    if arguments.len() != parameters.len() {
        return Err(handler.emit_err(CompileError::Internal(
            "Arguments and parameters length are not equal.",
            Span::dummy(),
        )));
    }

    handler.scope(|handler| {
        let typed_arguments = arguments
            .iter()
            .zip(parameters)
            .map(|(arg, param)| {
                let ctx = ctx
                    .by_ref()
                    .with_help_text(UNIFY_ARGS_HELP_TEXT)
                    .with_type_annotation(param.type_argument.type_id);
                ty::TyExpression::type_check(handler, ctx, arg)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, arg.span(), engines))
            })
            .collect();

        Ok(typed_arguments)
    })
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

    handler.scope(|handler| {
        for (arg, param) in typed_arguments.into_iter().zip(parameters.iter()) {
            // unify the type of the argument with the type of the param

            let unify_res = handler.scope(|unify_handler| {
                type_engine.unify(
                    unify_handler,
                    engines,
                    arg.return_type,
                    param.type_argument.type_id,
                    &arg.span,
                    UNIFY_ARGS_HELP_TEXT,
                    || None,
                );
                Ok(())
            });
            if unify_res.is_err() {
                continue;
            }

            // check for matching mutability
            let param_mutability =
                ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
            if arg.gather_mutability().is_immutable() && param_mutability.is_mutable() {
                handler.emit_err(CompileError::ImmutableArgumentToMutableParameter {
                    span: arg.span.clone(),
                });
            }

            typed_arguments_and_names.push((param.name.clone(), arg));
        }

        Ok(typed_arguments_and_names)
    })
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
