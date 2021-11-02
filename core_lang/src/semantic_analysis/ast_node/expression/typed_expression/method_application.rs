use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parser::{HllParser, Rule};
use crate::type_engine::{look_up_type_id, look_up_type_with_self};
use crate::types::ResolvedType;
use pest::Parser;
use std::collections::{HashMap, VecDeque};

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
        let arg_ret_type = look_up_type_id(
            namespace.resolve_type_with_self(look_up_type_id(arg.return_type), self_type),
        );
        let param_type = look_up_type_id(
            namespace.resolve_type_with_self(look_up_type_id(param.r#type), self_type),
        );
        if arg_ret_type != param_type && arg_ret_type != TypeInfo::ErrorRecovery {
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
                            .map(|x| TYPE_ENGINE.lock().unwrap().look_up_type_id(x.return_type))
                        {
                            Some(TypeInfo::ContractCaller { address, .. }) => address,
                            _ => {
                                errors.push(CompileError::Internal(
                                    "Attempted to find contract address of non-contract-call.",
                                    span.clone(),
                                ));
                                String::new()
                            }
                        };
                        // TODO(static span): this can be a normal address expression,
                        // so we don't need to re-parse and re-compile
                        let contract_address = check!(
                            re_parse_expression(
                                contract_address,
                                build_config,
                                &mut Default::default(),
                                namespace,
                                self_type,
                                dead_code_graph
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let func_selector =
                            check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                        Some(ContractCallMetadata {
                            func_selector,
                            contract_address: Box::new(contract_address),
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
                            .map(|x| TYPE_ENGINE.lock().unwrap().look_up_type_id(x.return_type))
                        {
                            Some(TypeInfo::ContractCaller { address, .. }) => address,
                            _ => {
                                errors.push(CompileError::Internal(
                                    "Attempted to find contract address of non-contract-call.",
                                    span.clone(),
                                ));
                                String::new()
                            }
                        };
                        let contract_address = check!(
                            re_parse_expression(
                                contract_address,
                                build_config,
                                &mut Default::default(),
                                namespace,
                                self_type,
                                dead_code_graph
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let func_selector =
                            check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                        Some(ContractCallMetadata {
                            func_selector,
                            contract_address: Box::new(contract_address),
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

// TODO(static span): this whole method can go away and the address can go back in the contract
// caller type.
fn re_parse_expression<'a>(
    contract_string: String,
    build_config: &BuildConfig,
    docstrings: &mut HashMap<String, String>,
    namespace: &mut Namespace<'a>,
    self_type: TypeId,
    dead_code_graph: &mut ControlFlowGraph<'a>,
) -> CompileResult<'a, TypedExpression<'a>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let span = crate::Span {
        span: pest::Span::new("TODO(static span): use Idents instead of Strings", 0, 0).unwrap(),
        path: None,
    };

    let leaked_contract_string = Box::leak(contract_string.into_boxed_str());
    let mut contract_pairs = match HllParser::parse(Rule::expr, leaked_contract_string) {
        Ok(o) => o,
        Err(_e) => {
            errors.push(CompileError::Internal(
                "Internal error handling contract call address parsing.",
                span.clone(),
            ));
            return err(warnings, errors);
        }
    };
    let contract_pair = match contract_pairs.next() {
        Some(o) => o,
        None => {
            errors.push(CompileError::Internal(
                "Internal error handling contract call address parsing. No address.",
                span.clone(),
            ));
            return err(warnings, errors);
        }
    };

    let contract_address = check!(
        Expression::parse_from_pair(contract_pair, Some(build_config), docstrings),
        return err(warnings, errors),
        warnings,
        errors
    );
    let contract_address = check!(
        TypedExpression::type_check(
            contract_address,
            namespace,
            None,
            "",
            self_type,
            build_config,
            dead_code_graph,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(contract_address, warnings, errors)
}
