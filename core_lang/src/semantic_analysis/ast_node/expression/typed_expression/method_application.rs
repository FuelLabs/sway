use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::types::{MaybeResolvedType, ResolvedType};
use std::collections::VecDeque;

pub(crate) fn type_check_method_application<'sc>(
    method_name: MethodName<'sc>,
    arguments: Vec<Expression<'sc>>,
    span: Span<'sc>,
    namespace: &mut Namespace<'sc>,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> CompileResult<'sc, TypedExpression<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut args_buf = VecDeque::new();
    for arg in arguments {
        args_buf.push_back(check!(
            TypedExpression::type_check(
                arg,
                namespace,
                None,
                "",
                self_type,
                build_config,
                dead_code_graph
            ),
            error_recovery_expr(span.clone()),
            warnings,
            errors
        ));
    }

    let method = check!(
        namespace
            .find_method_for_type(args_buf[0].return_type, &method_name, self_type, &args_buf,),
        return err(warnings, errors),
        warnings,
        errors
    );
    let contract_caller = if method.is_contract_call {
        args_buf.pop_front()
    } else {
        None
    };

    // type check all of the arguments against the parameters in the method declaration
    for (arg, param) in args_buf.iter().zip(method.parameters.iter()) {
        let arg_ret_type = namespace.look_up_type_id(arg.return_type);
        let param_type = namespace.look_up_type_id(param.r#type);
        if arg_ret_type != param_type && arg_ret_type != ResolvedType::ErrorRecovery {
            errors.push(CompileError::ArgumentParameterTypeMismatch {
                span: arg.span.clone(),
                provided: arg_ret_type.friendly_type_str(),
                should_be: param_type.friendly_type_str(),
            });
        }
    }
    let exp = match method_name {
        // something like a.b(c)
        MethodName::FromModule { method_name } => {
            if args_buf.len() > method.parameters.len() {
                errors.push(CompileError::TooManyArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.primary_name,
                    expected: method.parameters.len(),
                    received: args_buf.len(),
                });
            }

            if args_buf.len() < method.parameters.len() {
                errors.push(CompileError::TooFewArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.primary_name,
                    expected: method.parameters.len(),
                    received: args_buf.len(),
                });
            }

            let args_and_names = method
                .parameters
                .iter()
                .zip(args_buf.into_iter())
                .map(|(param, arg)| (param.name.clone(), arg))
                .collect::<Vec<(_, _)>>();

            TypedExpression {
                expression: TypedExpressionVariant::FunctionApplication {
                    name: CallPath {
                        prefixes: vec![],
                        suffix: method_name,
                    },
                    arguments: args_and_names,
                    function_body: method.body.clone(),
                    selector: if method.is_contract_call {
                        let contract_address = match contract_caller
                            .map(|x| namespace.look_up_type_id(x.return_type))
                        {
                            Some(ResolvedType::ContractCaller { address, .. }) => address,
                            _ => {
                                errors.push(CompileError::Internal(
                                    "Attempted to find contract address of non-contract-call.",
                                    span.clone(),
                                ));
                                Box::new(error_recovery_expr(span.clone()))
                            }
                        };
                        let func_selector =
                            check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                        Some(ContractCallMetadata {
                            func_selector,
                            contract_address,
                        })
                    } else {
                        None
                    },
                },
                return_type: method.return_type,
                is_constant: IsConstant::No,
                span,
            }
        }
        // something like blah::blah::~Type::foo()
        MethodName::FromType { ref call_path, .. } => {
            if args_buf.len() > method.parameters.len() {
                errors.push(CompileError::TooManyArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.easy_name(),
                    expected: method.parameters.len(),
                    received: args_buf.len(),
                });
            }

            if args_buf.len() < method.parameters.len() {
                errors.push(CompileError::TooFewArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.easy_name(),
                    expected: method.parameters.len(),
                    received: args_buf.len(),
                });
            }

            let args_and_names = method
                .parameters
                .iter()
                .zip(args_buf.into_iter())
                .map(|(param, arg)| (param.name.clone(), arg))
                .collect::<Vec<(_, _)>>();
            TypedExpression {
                expression: TypedExpressionVariant::FunctionApplication {
                    name: call_path.clone(),
                    arguments: args_and_names,
                    function_body: method.body.clone(),
                    selector: if method.is_contract_call {
                        let contract_address = match contract_caller
                            .map(|x| namespace.look_up_type_id(x.return_type))
                        {
                            Some(ResolvedType::ContractCaller { address, .. }) => address,
                            _ => {
                                errors.push(CompileError::Internal(
                                    "Attempted to find contract address of non-contract-call.",
                                    span.clone(),
                                ));
                                Box::new(error_recovery_expr(span.clone()))
                            }
                        };
                        let func_selector =
                            check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                        Some(ContractCallMetadata {
                            func_selector,
                            contract_address,
                        })
                    } else {
                        None
                    },
                },
                return_type: method.return_type,
                is_constant: IsConstant::No,
                span,
            }
        }
    };
    ok(exp, warnings, errors)
}
