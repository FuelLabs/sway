use sway_types::Ident;
use sway_types::{state::StateIndex, Span};

use crate::constants;
use crate::Expression::StorageAccess;

use crate::{error::*, parse_tree::*, semantic_analysis::*, type_engine::*, types::*};

use std::collections::{HashMap, VecDeque};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    method_name: MethodName,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
    namespace: &mut Namespace,
    self_type: TypeId,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut args_buf = VecDeque::new();
    let mut contract_call_params_map = HashMap::new();
    for arg in &arguments {
        args_buf.push_back(check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: arg.clone(),
                namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(span.clone()),
            warnings,
            errors
        ));
    }

    let method = check!(
        resolve_method_name(
            &method_name,
            args_buf.clone(),
            type_arguments,
            span.clone(),
            namespace,
            self_type
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    let contract_caller = if method.is_contract_call {
        args_buf.pop_front()
    } else {
        None
    };

    // 'method.purity' is that of the callee, 'opts.purity' of the caller.
    if !opts.purity.can_call(method.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(opts.purity, method.purity).to_attribute_syntax(),
            span: method_name.easy_name().span().clone(),
        });
    }

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

    // If this method was called with self being a `StorageAccess` (e.g. storage.map.insert(..)),
    // then record the index of that storage variable and pass it on.
    let mut self_state_idx = None;
    if namespace.has_storage_declared() {
        let storage_fields = check!(
            namespace.get_storage_field_descriptors(),
            return err(warnings, errors),
            warnings,
            errors
        );

        self_state_idx = match arguments.first() {
            Some(StorageAccess { field_names, .. }) => {
                let first_field = field_names[0].clone();
                let self_state_idx = match storage_fields
                    .iter()
                    .enumerate()
                    .find(|(_, TypedStorageField { name, .. })| name == &first_field)
                {
                    Some((ix, _)) => StateIndex::new(ix),
                    None => {
                        errors.push(CompileError::StorageFieldDoesNotExist {
                            name: first_field.clone(),
                        });
                        return err(warnings, errors);
                    }
                };
                Some(self_state_idx)
            }
            _ => None,
        }
    };

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

    match method_name {
        // something like a.b(c)
        MethodName::FromModule { method_name } => {
            let selector = if method.is_contract_call {
                let contract_address = match contract_caller.map(|x| look_up_type_id(x.return_type))
                {
                    Some(TypeInfo::ContractCaller { address, .. }) => address,
                    _ => {
                        errors.push(CompileError::Internal(
                            "Attempted to find contract address of non-contract-call.",
                            span.clone(),
                        ));
                        None
                    }
                };
                let contract_address = if let Some(addr) = contract_address {
                    addr
                } else {
                    errors.push(CompileError::ContractAddressMustBeKnown {
                        span: method_name.span().clone(),
                    });
                    return err(warnings, errors);
                };
                let func_selector = check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                Some(ContractCallMetadata {
                    func_selector,
                    contract_address,
                })
            } else {
                None
            };

            let exp = check!(
                instantiate_function_application_simple(
                    CallPath {
                        prefixes: vec![],
                        suffix: method_name,
                        is_absolute: false,
                    },
                    contract_call_params_map,
                    args_buf,
                    method,
                    selector,
                    IsConstant::No,
                    self_state_idx,
                    span,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(exp, warnings, errors)
        }

        // something like blah::blah::~Type::foo()
        MethodName::FromType { call_path, .. } | MethodName::FromTrait { call_path } => {
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
                        None
                    }
                };
                let contract_address = if let Some(addr) = contract_address {
                    addr
                } else {
                    errors.push(CompileError::ContractAddressMustBeKnown {
                        span: call_path.span(),
                    });
                    return err(warnings, errors);
                };
                let func_selector = check!(method.to_fn_selector_value(), [0; 4], warnings, errors);
                Some(ContractCallMetadata {
                    func_selector,
                    contract_address,
                })
            } else {
                None
            };

            let exp = check!(
                instantiate_function_application_simple(
                    call_path,
                    contract_call_params_map,
                    args_buf,
                    method,
                    selector,
                    IsConstant::No,
                    self_state_idx,
                    span,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(exp, warnings, errors)
        }
    }
}

pub(crate) fn resolve_method_name(
    method_name: &MethodName,
    arguments: VecDeque<TypedExpression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
    namespace: &mut Namespace,
    self_type: TypeId,
) -> CompileResult<TypedFunctionDeclaration> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let func_decl = match method_name {
        MethodName::FromType {
            call_path,
            type_name,
            type_name_span,
        } => check!(
            find_method(
                type_name,
                type_name_span,
                type_arguments,
                namespace,
                &arguments,
                call_path,
                self_type
            ),
            return err(warnings, errors),
            warnings,
            errors
        ),
        MethodName::FromTrait { call_path } => {
            let (type_name, type_name_span) = arguments
                .get(0)
                .map(|x| (look_up_type_id(x.return_type), x.span.clone()))
                .unwrap_or_else(|| (TypeInfo::Unknown, span.clone()));
            check!(
                find_method(
                    &type_name,
                    &type_name_span,
                    type_arguments,
                    namespace,
                    &arguments,
                    call_path,
                    self_type
                ),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
        MethodName::FromModule { method_name } => {
            let ty = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| insert_type(TypeInfo::Unknown));
            let abs_path: Vec<_> = namespace.find_module_path(Some(method_name));
            check!(
                namespace.find_method_for_type(ty, &abs_path, self_type, &arguments),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
    };
    ok(func_decl, warnings, errors)
}

fn find_method(
    type_name: &TypeInfo,
    type_name_span: &Span,
    type_arguments: Vec<TypeArgument>,
    namespace: &mut Namespace,
    arguments: &VecDeque<TypedExpression>,
    call_path: &CallPath,
    self_type: TypeId,
) -> CompileResult<TypedFunctionDeclaration> {
    let warnings = vec![];
    let mut errors = vec![];
    let ty = match (type_name, type_arguments.is_empty()) {
        (
            TypeInfo::Custom {
                name,
                type_arguments: type_args,
            },
            false,
        ) => {
            if type_args.is_empty() {
                TypeInfo::Custom {
                    name: name.clone(),
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
                span: type_name_span.clone(),
                name: call_path.suffix.clone(),
            });
            return err(warnings, errors);
        }
        (ty, true) => ty.clone(),
    };
    let abs_path: Vec<Ident> = if call_path.is_absolute {
        call_path.full_path().cloned().collect()
    } else {
        namespace.find_module_path(call_path.full_path())
    };
    namespace.find_method_for_type(insert_type(ty), &abs_path, self_type, arguments)
}
