use crate::{
    declaration_engine::ReplaceDecls,
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
    mut function_decl: ty::TyFunctionDeclaration,
    call_path: CallPath,
    arguments: Vec<Expression>,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let declaration_engine = ctx.declaration_engine;
    let engines = ctx.engines();

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

    let typed_arguments_with_names = check!(
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
    function_decl.replace_decls(&decl_mapping, engines);
    let return_type = function_decl.return_type;
    let span = function_decl.span.clone();
    let new_decl_id = declaration_engine.insert_function(function_decl);

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params: HashMap::new(),
            arguments: typed_arguments_with_names,
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
    let declaration_engine = ctx.declaration_engine;
    let engines = ctx.engines();

    let typed_arguments = arguments
        .into_iter()
        .map(|arg| {
            let ctx = ctx.by_ref().with_help_text("").with_type_annotation(
                type_engine.insert_type(declaration_engine, TypeInfo::Unknown),
            );
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
    parameters: &[ty::TyFunctionParameter],
) -> CompileResult<Vec<(Ident, ty::TyExpression)>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let declaration_engine = ctx.declaration_engine;
    let mut typed_arguments_and_names = vec![];

    for (arg, param) in typed_arguments.into_iter().zip(parameters.iter()) {
        // unify the type of the argument with the type of the param
        check!(
            CompileResult::from(type_engine.unify(
                declaration_engine,
                arg.return_type,
                param.type_id,
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
        let param_mutability =
            ty::VariableMutability::new_from_ref_mut(param.is_reference, param.is_mutable);
        if arg.gather_mutability().is_immutable() && param_mutability.is_mutable() {
            errors.push(CompileError::ImmutableArgumentToMutableParameter {
                span: arg.span.clone(),
            });
        }

        typed_arguments_and_names.push((param.name.clone(), arg));
    }

    if errors.is_empty() {
        ok(typed_arguments_and_names, warnings, errors)
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
