use super::*;
use crate::build_config::BuildConfig;
use crate::constants;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::parse_tree::{MethodName, StructExpressionField};
use crate::parser::{Rule, SwayParser};
use crate::semantic_analysis::TCOpts;
use pest::iterators::Pairs;
use pest::Parser;
use std::collections::{HashMap, VecDeque};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    method_name: MethodName,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
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
    let mut contract_call_params_map = HashMap::new();
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
            ref call_path,
            ref type_name,
            ref type_name_span,
        } => {
            let (ty, type_name_span): (TypeInfo, Span) = match (type_name, type_name_span) {
                (Some(type_name), Some(type_name_span)) => {
                    (type_name.clone(), type_name_span.clone())
                }
                _ => args_buf
                    .get(0)
                    .map(|x| (look_up_type_id(x.return_type), x.span.clone()))
                    .unwrap_or_else(|| (TypeInfo::Unknown, span.clone())),
            };
            let ty = match (ty, type_arguments.is_empty()) {
                (
                    TypeInfo::Custom {
                        name,
                        type_arguments: type_args,
                    },
                    false,
                ) => {
                    if type_args.is_empty() {
                        TypeInfo::Custom {
                            name,
                            type_arguments,
                        }
                    } else {
                        let type_args_span = type_args
                            .iter()
                            .map(|x| x.span.clone())
                            .fold(type_args[0].span.clone(), Span::join);
                        errors.push(CompileError::Internal(
                            "did not expect to find type arguments here",
                            type_args_span,
                        ));
                        return err(warnings, errors);
                    }
                }
                (_, false) => {
                    errors.push(CompileError::DoesNotTakeTypeArguments {
                        span: type_name_span,
                        name: call_path.suffix.clone(),
                    });
                    return err(warnings, errors);
                }
                (ty, true) => ty,
            };
            let from_module = if call_path.is_absolute {
                Some(crate_namespace)
            } else {
                None
            };
            check!(
                namespace.find_method_for_type(
                    insert_type(ty),
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

    if !method.is_contract_call {
        if !contract_call_params.is_empty() {
            errors.push(CompileError::CallParamForNonContractCallMethod {
                span: contract_call_params[0].name.span().clone(),
            });
        }
    } else {
        for param_name in &[
            constants::CONTRACT_CALL_GAS_PARAMETER_NAME,
            constants::CONTRACT_CALL_COINS_PARAMETER_NAME,
            constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME,
        ] {
            if contract_call_params
                .iter()
                .filter(|&param| param.name.span().as_str() == *param_name)
                .count()
                > 1
            {
                errors.push(CompileError::ContractCallParamRepeated {
                    param_name: param_name.to_string(),
                    span: span.clone(),
                });
            }
        }

        for param in contract_call_params {
            match param.name.span().as_str() {
                constants::CONTRACT_CALL_GAS_PARAMETER_NAME
                | constants::CONTRACT_CALL_COINS_PARAMETER_NAME
                | constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME => {
                    contract_call_params_map.insert(
                        param.name.to_string(),
                        check!(
                            TypedExpression::type_check(TypeCheckArguments {
                                checkee: param.value,
                                namespace,
                                crate_namespace,
                                return_type_annotation: match param.name.span().as_str() {
                                    constants::CONTRACT_CALL_GAS_PARAMETER_NAME
                                    | constants::CONTRACT_CALL_COINS_PARAMETER_NAME => insert_type(
                                        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                                    ),
                                    constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME =>
                                        insert_type(TypeInfo::B256),
                                    _ => unreachable!(),
                                },
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
                        ),
                    );
                }
                _ => {
                    errors.push(CompileError::UnrecognizedContractParam {
                        param_name: param.name.to_string(),
                        span: param.name.span().clone(),
                    });
                }
            };
        }
    }

    // type check all of the arguments against the parameters in the method declaration
    for (arg, param) in args_buf.iter().zip(method.parameters.iter()) {
        // if the return type cannot be cast into the annotation type then it is a type error
        let (mut new_warnings, new_errors) = unify_with_self(
            arg.return_type,
            param.r#type,
            self_type,
            &arg.span,
            "This argument's type is not castable to the declared parameter type.",
        );
        warnings.append(&mut new_warnings);
        if !new_errors.is_empty() {
            errors.push(CompileError::ArgumentParameterTypeMismatch {
                span: arg.span.clone(),
                provided: arg.return_type.friendly_type_str(),
                should_be: param.r#type.friendly_type_str(),
            });
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

            let selector = if method.is_contract_call {
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
                        span.clone(),
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let func_selector = check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                Some(ContractCallMetadata {
                    func_selector,
                    contract_address: Box::new(contract_address),
                })
            } else {
                None
            };

            let expression = TypedExpressionVariant::FunctionApplication {
                name: CallPath {
                    prefixes: vec![],
                    suffix: method_name,
                    is_absolute: false,
                },
                contract_call_params: contract_call_params_map,
                arguments: args_and_names,
                function_body: method.body.clone(),
                selector,
            };

            TypedExpression {
                expression,
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

            let selector = if method.is_contract_call {
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
                        span.clone(),
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let func_selector = check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                Some(ContractCallMetadata {
                    func_selector,
                    contract_address: Box::new(contract_address),
                })
            } else {
                None
            };

            let expression = TypedExpressionVariant::FunctionApplication {
                name: call_path.clone(),
                contract_call_params: contract_call_params_map,
                arguments: args_and_names,
                function_body: method.body.clone(),
                selector,
            };

            TypedExpression {
                expression,
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
    span: Span,
) -> CompileResult<TypedExpression> {
    if contract_string.is_empty() {
        return err(
            vec![],
            vec![CompileError::ContractAddressMustBeKnown { span }],
        );
    }
    let mut warnings = vec![];
    let mut errors = vec![];
    let span = sway_types::span::Span::new(
        "TODO(static span): use Idents instead of Strings".into(),
        0,
        0,
        None,
    )
    .unwrap();

    let mut contract_pairs: Pairs<Rule> = match SwayParser::parse(Rule::expr, contract_string) {
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

    // purposefully ignore var_decls as those have already been lifted during parsing
    let ParserLifter { value, .. } = check!(
        Expression::parse_from_pair(contract_pair, Some(build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );

    let contract_address = check!(
        TypedExpression::type_check(TypeCheckArguments {
            checkee: value,
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
