use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parse_tree::MethodName;
use crate::parser::{Rule, SwayParser};
use crate::semantic_analysis::TCOpts;
use pest::Parser;
use std::collections::VecDeque;

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    method_name: MethodName,
    arguments: Vec<Expression>,
    span: Span,
    namespace: NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut args_buf = VecDeque::new();
    for arg in arguments {
        args_buf.push_back(check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: arg,
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(span.clone()),
            warnings,
            errors
        ));
    }

    let method = match method_name {
        MethodName::FromType {
            ref type_name,
            ref call_path,
        } => {
            let ty = match type_name {
                Some(name) => {
                    if *name == TypeInfo::SelfType {
                        self_type
                    } else {
                        insert_type(name.clone())
                    }
                }
                None => args_buf
                    .get(0)
                    .map(|x| x.return_type)
                    .unwrap_or_else(|| insert_type(TypeInfo::Unknown)),
            };
            let from_module = if call_path.is_absolute {
                Some(crate_namespace)
            } else {
                None
            };
            check!(
                namespace.find_method_for_type(
                    ty,
                    &call_path.suffix,
                    &call_path.prefixes[..],
                    from_module,
                    self_type,
                    &args_buf,
                ),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
        MethodName::FromModule { ref method_name } => {
            let ty = args_buf
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| insert_type(TypeInfo::Unknown));
            check!(
                namespace.find_method_for_type(ty, method_name, &[], None, self_type, &args_buf),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
    };
    let contract_caller = if method.is_contract_call {
        args_buf.pop_front()
    } else {
        None
    };

    // type check all of the arguments against the parameters in the method declaration
    for (arg, param) in args_buf.iter().zip(method.parameters.iter()) {
        // if the return type cannot be cast into the annotation type then it is a type error
        match crate::type_engine::unify_with_self(
            arg.return_type,
            param.r#type,
            self_type,
            &arg.span,
        ) {
            Ok(mut ws) => {
                warnings.append(&mut ws);
            }
            Err(_e) => {
                errors.push(CompileError::ArgumentParameterTypeMismatch {
                    span: arg.span.clone(),
                    provided: arg.return_type.friendly_type_str(),
                    should_be: param.r#type.friendly_type_str(),
                });
            }
        }
        // The annotation may result in a cast, which is handled in the type engine.
    }
    let exp = match method_name {
        // something like a.b(c)
        MethodName::FromModule { method_name } => {
            if args_buf.len() > method.parameters.len() {
                errors.push(CompileError::TooManyArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.clone(),
                    expected: method.parameters.len(),
                    received: args_buf.len(),
                });
            }

            if args_buf.len() < method.parameters.len() {
                errors.push(CompileError::TooFewArgumentsForFunction {
                    span: span.clone(),
                    method_name: method_name.clone(),
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
                        is_absolute: false,
                    },
                    arguments: args_and_names,
                    function_body: method.body.clone(),
                    selector: if method.is_contract_call {
                        let contract_address = match contract_caller
                            .map(|x| crate::type_engine::look_up_type_id(x.return_type))
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
                                contract_address.into(),
                                build_config,
                                namespace,
                                crate_namespace,
                                self_type,
                                dead_code_graph,
                                opts,
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
                            .map(|x| crate::type_engine::look_up_type_id(x.return_type))
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
                                contract_address.into(),
                                build_config,
                                namespace,
                                crate_namespace,
                                self_type,
                                dead_code_graph,
                                opts,
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
#[allow(clippy::too_many_arguments)]
fn re_parse_expression(
    contract_string: Arc<str>,
    build_config: &BuildConfig,
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let span = sway_types::span::Span {
        span: pest::Span::new(
            "TODO(static span): use Idents instead of Strings".into(),
            0,
            0,
        )
        .unwrap(),
        path: None,
    };

    let mut contract_pairs = match SwayParser::parse(Rule::expr, contract_string) {
        Ok(o) => o,
        Err(_e) => {
            errors.push(CompileError::Internal(
                "Internal error handling contract call address parsing.",
                span,
            ));
            return err(warnings, errors);
        }
    };
    let contract_pair = match contract_pairs.next() {
        Some(o) => o,
        None => {
            errors.push(CompileError::Internal(
                "Internal error handling contract call address parsing. No address.",
                span,
            ));
            return err(warnings, errors);
        }
    };

    let contract_address = check!(
        Expression::parse_from_pair(contract_pair, Some(build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );
    let contract_address = check!(
        TypedExpression::type_check(TypeCheckArguments {
            checkee: contract_address,
            namespace,
            crate_namespace,
            return_type_annotation: insert_type(TypeInfo::Unknown),
            help_text: Default::default(),
            self_type,
            build_config,
            dead_code_graph,
            mode: Mode::NonAbi,
            opts,
        }),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(contract_address, warnings, errors)
}
