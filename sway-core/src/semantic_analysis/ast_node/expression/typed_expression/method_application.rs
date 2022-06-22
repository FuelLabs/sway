use crate::constants;
use crate::Expression::StorageAccess;
use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{TypedExpressionVariant::VariableExpression, *},
    type_engine::*,
};
use std::collections::{HashMap, VecDeque};
use sway_types::Spanned;
use sway_types::{state::StateIndex, Span};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    method_name_binding: TypeBinding<MethodName>,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
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
        TypedFunctionDeclaration::find_from_method_name(
            method_name_binding.clone(),
            args_buf.clone(),
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

    if !method.is_contract_call {
        // 'method.purity' is that of the callee, 'opts.purity' of the caller.
        if !opts.purity.can_call(method.purity) {
            errors.push(CompileError::StorageAccessMismatch {
                attrs: promote_purity(opts.purity, method.purity).to_attribute_syntax(),
                span: method_name_binding.inner.easy_name().span(),
            });
        }

        if !contract_call_params.is_empty() {
            errors.push(CompileError::CallParamForNonContractCallMethod {
                span: contract_call_params[0].name.span(),
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
            param.type_id,
            self_type,
            &arg.span,
            "This argument's type is not castable to the declared parameter type.",
        );
        warnings.append(&mut new_warnings);
        if !new_errors.is_empty() {
            errors.push(CompileError::ArgumentParameterTypeMismatch {
                span: arg.span.clone(),
                provided: arg.return_type.to_string(),
                should_be: param.type_id.to_string(),
            });
        }
        // The annotation may result in a cast, which is handled in the type engine.
    }

    // Validate mutability of self. Check that the variable that the method is called on is mutable
    // _if_ the method requires mutable self.
    if let (
        Some(TypedExpression {
            expression: VariableExpression { name, .. },
            ..
        }),
        Some(TypedFunctionParameter { is_mutable, .. }),
    ) = (args_buf.get(0), method.parameters.get(0))
    {
        let unknown_decl = check!(
            namespace.resolve_symbol(name).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let variable_decl = check!(
            unknown_decl.expect_variable().cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        if !variable_decl.is_mutable.is_mutable() && *is_mutable {
            errors.push(CompileError::MethodRequiresMutableSelf {
                method_name: method_name_binding.inner.easy_name(),
                variable_name: name.clone(),
                span,
            });
            return err(warnings, errors);
        }
    }

    let call_path = match method_name_binding.inner {
        MethodName::FromModule { method_name } => CallPath {
            prefixes: vec![],
            suffix: method_name,
            is_absolute: false,
        },
        MethodName::FromType {
            call_path_binding, ..
        } => call_path_binding.inner,
        MethodName::FromTrait { call_path } => call_path,
    };

    let selector = if method.is_contract_call {
        let contract_address =
            match contract_caller.map(|x| crate::type_engine::look_up_type_id(x.return_type)) {
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
