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
    mut ctx: TypeCheckContext,
    method_name: MethodName,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // type check the function arguments
    let mut args_buf = VecDeque::new();
    for arg in &arguments {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        args_buf.push_back(check!(
            TypedExpression::type_check(ctx, arg.clone()),
            error_recovery_expr(span.clone()),
            warnings,
            errors
        ));
    }

    // resolve the method name to a typed function declaration
    let method = check!(
        resolve_method_name(ctx.by_ref(), &method_name, args_buf.clone(), type_arguments,),
        return err(warnings, errors),
        warnings,
        errors
    );

    // check the function storage purity
    if !method.is_contract_call {
        // 'method.purity' is that of the callee, 'opts.purity' of the caller.
        if !ctx.purity().can_call(method.purity) {
            errors.push(CompileError::StorageAccessMismatch {
                attrs: promote_purity(ctx.purity(), method.purity).to_attribute_syntax(),
                span: method_name.easy_name().span(),
            });
        }
        if !contract_call_params.is_empty() {
            errors.push(CompileError::CallParamForNonContractCallMethod {
                span: contract_call_params[0].name.span(),
            });
        }
    }

    // generate the map of the contract call params
    let mut contract_call_params_map = HashMap::new();
    if method.is_contract_call {
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
            let type_annotation = match param.name.span().as_str() {
                constants::CONTRACT_CALL_GAS_PARAMETER_NAME
                | constants::CONTRACT_CALL_COINS_PARAMETER_NAME => {
                    insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour))
                }
                constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME => insert_type(TypeInfo::B256),
                _ => unreachable!(),
            };
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_annotation);
            match param.name.span().as_str() {
                constants::CONTRACT_CALL_GAS_PARAMETER_NAME
                | constants::CONTRACT_CALL_COINS_PARAMETER_NAME
                | constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME => {
                    contract_call_params_map.insert(
                        param.name.to_string(),
                        check!(
                            TypedExpression::type_check(ctx, param.value),
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
    if ctx.namespace.has_storage_declared() {
        let storage_fields = check!(
            ctx.namespace.get_storage_field_descriptors(),
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

    // If this function is being called with method call syntax, a.b(c),
    // then make sure the first parameter is self, else issue an error.
    if !method.is_contract_call {
        if let MethodName::FromModule { ref method_name } = method_name {
            let is_first_param_self = method
                .parameters
                .get(0)
                .map(|f| f.is_self())
                .unwrap_or_default();
            if !is_first_param_self {
                errors.push(CompileError::AssociatedFunctionCalledAsMethod {
                    fn_name: method_name.clone(),
                    span,
                });
                return err(warnings, errors);
            }
        }
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
            ctx.namespace.resolve_symbol(name).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        let is_decl_mutable = match unknown_decl {
            TypedDeclaration::ConstantDeclaration(_) => false,
            _ => {
                let variable_decl = check!(
                    unknown_decl.expect_variable().cloned(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                variable_decl.is_mutable.is_mutable()
            }
        };

        if !is_decl_mutable && *is_mutable {
            errors.push(CompileError::MethodRequiresMutableSelf {
                method_name: method_name.easy_name(),
                variable_name: name.clone(),
                span,
            });
            return err(warnings, errors);
        }
    }

    // retrieve the function call path
    let call_path = match method_name {
        MethodName::FromType {
            call_path,
            method_name,
        } => CallPath {
            prefixes: call_path.prefixes,
            suffix: method_name,
            is_absolute: call_path.is_absolute,
        },
        MethodName::FromModule { method_name } => CallPath {
            prefixes: vec![],
            suffix: method_name,
            is_absolute: false,
        },
        MethodName::FromTrait { call_path } => call_path,
    };

    // build the function selector
    let selector = if method.is_contract_call {
        let contract_caller = args_buf.pop_front();
        let contract_address = match contract_caller.map(|x| look_up_type_id(x.return_type)) {
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

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(args_buf.len(), &method, &call_path),
        return err(warnings, errors),
        warnings,
        errors
    );

    // unify the types of the arguments with the types of the parameters from the function declaration
    for (arg, param) in args_buf.iter().zip(method.parameters.iter()) {
        let (mut new_warnings, new_errors) = ctx
            .by_ref()
            .with_help_text("This argument's type is not castable to the declared parameter type.")
            .with_type_annotation(param.type_id)
            .unify_with_self(arg.return_type, &arg.span);
        warnings.append(&mut new_warnings);
        if !new_errors.is_empty() {
            errors.push(CompileError::ArgumentParameterTypeMismatch {
                span: arg.span.clone(),
                provided: arg.return_type.to_string(),
                should_be: param.type_id.to_string(),
            });
        }
    }

    // build the function application
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

pub(crate) fn resolve_method_name(
    mut ctx: TypeCheckContext,
    method_name: &MethodName,
    arguments: VecDeque<TypedExpression>,
    type_arguments: Vec<TypeArgument>,
) -> CompileResult<TypedFunctionDeclaration> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let func_decl = match method_name {
        MethodName::FromType {
            call_path,
            method_name,
        } => {
            let (type_info, type_info_span) = call_path.suffix.clone();

            // find the module that the symbol is in
            let type_info_prefix = ctx.namespace.find_module_path(&call_path.prefixes);
            check!(
                ctx.namespace.root().check_submodule(&type_info_prefix),
                return err(warnings, errors),
                warnings,
                errors
            );

            // create the type info object
            let type_info = check!(
                type_info.apply_type_arguments(type_arguments, &type_info_span),
                return err(warnings, errors),
                warnings,
                errors
            );

            // resolve the type of the type info object
            let type_id = check!(
                ctx.resolve_type_with_self(
                    insert_type(type_info),
                    &type_info_span,
                    EnforceTypeArguments::No,
                    Some(&type_info_prefix)
                ),
                insert_type(TypeInfo::ErrorRecovery),
                warnings,
                errors
            );

            // find the method
            check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &type_info_prefix,
                    method_name,
                    ctx.self_type(),
                    &arguments
                ),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
        MethodName::FromTrait { call_path } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace.find_module_path(&call_path.prefixes);

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| insert_type(TypeInfo::Unknown));

            // find the method
            check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &module_path,
                    &call_path.suffix,
                    ctx.self_type(),
                    &arguments
                ),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
        MethodName::FromModule { method_name } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace.find_module_path(vec![]);

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| insert_type(TypeInfo::Unknown));

            // find the method
            check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &module_path,
                    method_name,
                    ctx.self_type(),
                    &arguments
                ),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
    };
    ok(func_decl, warnings, errors)
}
