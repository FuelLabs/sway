use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::*;
use crate::semantic_analysis::{ast_node::*, TCOpts, TypeCheckArguments};
use crate::type_engine::TypeId;
use std::cmp::Ordering;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    typed_function_decl: TypedFunctionDeclaration,
    name: CallPath,
    arguments: Vec<Expression>,
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let TypedFunctionDeclaration {
        parameters,
        return_type,
        body,
        span,
        purity,
        ..
    } = typed_function_decl;

    if opts.purity != purity {
        errors.push(CompileError::PureCalledImpure { span: name.span() });
    }

    match arguments.len().cmp(&parameters.len()) {
        Ordering::Greater => {
            let arguments_span = arguments.iter().fold(
                arguments
                    .get(0)
                    .map(|x| x.span())
                    .unwrap_or_else(|| name.span()),
                |acc, arg| join_spans(acc, arg.span()),
            );
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: arguments_span,
                method_name: name.suffix.clone(),
                expected: parameters.len(),
                received: arguments.len(),
            });
        }
        Ordering::Less => {
            let arguments_span = arguments.iter().fold(
                arguments
                    .get(0)
                    .map(|x| x.span())
                    .unwrap_or_else(|| name.span()),
                |acc, arg| join_spans(acc, arg.span()),
            );
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: arguments_span,
                method_name: name.suffix.clone(),
                expected: parameters.len(),
                received: arguments.len(),
            });
        }
        Ordering::Equal => {}
    }
    // type check arguments in function application vs arguments in function
    // declaration. Use parameter type annotations as annotations for the
    // arguments
    //
    let typed_call_arguments = arguments
        .into_iter()
        .zip(parameters.iter())
        .map(|(arg, param)| {
            (
                param.name.clone(),
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: arg.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: param.r#type,
                    help_text: "The argument that has been provided to this function's type does \
                        not match the declared type of the parameter in the function \
                        declaration.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                })
                .unwrap_or_else(&mut warnings, &mut errors, || {
                    error_recovery_expr(arg.span())
                }),
            )
        })
        .collect();

    ok(
        TypedExpression {
            return_type,
            // now check the function call return type
            // FEATURE this IsConstant can be true if the function itself is
            // constant-able const functions would be an
            // advanced feature and are not supported right
            // now
            is_constant: IsConstant::No,
            expression: TypedExpressionVariant::FunctionApplication {
                arguments: typed_call_arguments,
                name,
                function_body: body,
                selector: None, // regular functions cannot be in a contract call; only methods
            },
            span,
        },
        warnings,
        errors,
    )
}
