use crate::{
    decl_engine::{DeclEngineInsert, DeclRefFunction, ReplaceDecls, UpdateConstantExpression},
    error::*,
    language::{parsed::*, ty, *},
    semantic_analysis::*,
    type_system::*,
};
use ast_node::typed_expression::check_function_arguments_arity;
use std::collections::{HashMap, VecDeque};
use sway_error::error::CompileError;
use sway_types::{constants, integer_bits::IntegerBits};
use sway_types::{constants::CONTRACT_CALL_COINS_PARAMETER_NAME, Spanned};
use sway_types::{Ident, Span};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    mut ctx: TypeCheckContext,
    mut method_name_binding: TypeBinding<MethodName>,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    // type check the function arguments
    let mut args_buf = VecDeque::new();
    for arg in &arguments {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));
        args_buf.push_back(check!(
            ty::TyExpression::type_check(ctx, arg.clone()),
            ty::TyExpression::error(span.clone(), engines),
            warnings,
            errors
        ));
    }

    // resolve the method name to a typed function declaration and type_check
    let (decl_ref, call_path_typeid) = check!(
        resolve_method_name(ctx.by_ref(), &mut method_name_binding, args_buf.clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut method = decl_engine.get_function(&decl_ref);

    // check the method visibility
    if span.path() != method.span.path() && method.visibility.is_private() {
        errors.push(CompileError::CallingPrivateLibraryMethod {
            name: method.name.as_str().to_string(),
            span,
        });
        return err(warnings, errors);
    }

    // check the function storage purity
    if !method.is_contract_call {
        // 'method.purity' is that of the callee, 'opts.purity' of the caller.
        if !ctx.purity().can_call(method.purity) {
            errors.push(CompileError::StorageAccessMismatch {
                attrs: promote_purity(ctx.purity(), method.purity).to_attribute_syntax(),
                span: method_name_binding.inner.easy_name().span(),
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
            match param.name.span().as_str() {
                constants::CONTRACT_CALL_GAS_PARAMETER_NAME
                | constants::CONTRACT_CALL_COINS_PARAMETER_NAME
                | constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME => {
                    let type_annotation = type_engine.insert(
                        decl_engine,
                        if param.name.span().as_str()
                            != constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME
                        {
                            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                        } else {
                            TypeInfo::B256
                        },
                    );
                    let ctx = ctx
                        .by_ref()
                        .with_help_text("")
                        .with_type_annotation(type_annotation);
                    contract_call_params_map.insert(
                        param.name.to_string(),
                        check!(
                            ty::TyExpression::type_check(ctx, param.value),
                            ty::TyExpression::error(span.clone(), engines),
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

        // check if method is non-payable but we do not know _statically_
        // the amount of coins sent in the contract call is zero
        // if the coins contract call parameter is not specified
        // it's considered to be zero and hence no error needs to be reported
        if let Some(coins_expr) = contract_call_params_map.get(CONTRACT_CALL_COINS_PARAMETER_NAME) {
            if coins_analysis::possibly_nonzero_u64_expression(
                ctx.namespace,
                decl_engine,
                coins_expr,
            ) && !method
                .attributes
                .contains_key(&crate::transform::AttributeKind::Payable)
            {
                errors.push(CompileError::CoinsPassedToNonPayableMethod {
                    fn_name: method.name,
                    span,
                });
                return err(warnings, errors);
            }
        }
    }

    // If this function is being called with method call syntax, a.b(c),
    // then make sure the first parameter is self, else issue an error.
    let mut is_method_call_syntax_used = false;
    if !method.is_contract_call {
        if let MethodName::FromModule { ref method_name } = method_name_binding.inner {
            is_method_call_syntax_used = true;
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
    fn mutability_check(
        ctx: &TypeCheckContext,
        method_name_binding: &TypeBinding<MethodName>,
        span: &Span,
        exp: &ty::TyExpressionVariant,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        match exp {
            ty::TyExpressionVariant::VariableExpression { name, .. } => {
                let unknown_decl = check!(
                    ctx.namespace.resolve_symbol(name).cloned(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let is_decl_mutable = match unknown_decl {
                    ty::TyDecl::ConstantDecl { .. } => false,
                    _ => {
                        let variable_decl = check!(
                            unknown_decl.expect_variable().cloned(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        variable_decl.mutability.is_mutable()
                    }
                };

                if !is_decl_mutable {
                    errors.push(CompileError::MethodRequiresMutableSelf {
                        method_name: method_name_binding.inner.easy_name(),
                        variable_name: name.clone(),
                        span: span.clone(),
                    });
                    return err(warnings, errors);
                }

                ok((), warnings, errors)
            }
            ty::TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                mutability_check(ctx, method_name_binding, span, &prefix.expression)
            }
            _ => ok((), warnings, errors),
        }
    }

    if let (
        Some(ty::TyExpression {
            expression: exp, ..
        }),
        Some(ty::TyFunctionParameter { is_mutable, .. }),
    ) = (args_buf.get(0), method.parameters.get(0))
    {
        if *is_mutable {
            check!(
                mutability_check(&ctx, &method_name_binding, &span, exp),
                return err(warnings, errors),
                warnings,
                errors
            );
        }
    }

    // retrieve the function call path
    let call_path = match method_name_binding.inner.clone() {
        MethodName::FromType {
            call_path_binding,
            method_name,
        } => {
            let mut prefixes = call_path_binding.inner.prefixes;
            prefixes.push(match &call_path_binding.inner.suffix {
                (TypeInfo::Custom { call_path, .. }, ..) => call_path.clone().suffix,
                (_, ident) => ident.clone(),
            });

            CallPath {
                prefixes,
                suffix: method_name,
                is_absolute: call_path_binding.inner.is_absolute,
            }
        }
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
        let contract_address = match contract_caller
            .clone()
            .map(|x| type_engine.get(x.return_type))
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
        let func_selector = check!(
            method.to_fn_selector_value(type_engine, decl_engine),
            [0; 4],
            warnings,
            errors
        );
        let contract_caller = contract_caller.unwrap();
        Some(ty::ContractCallParams {
            func_selector,
            contract_address,
            contract_caller: Box::new(contract_caller),
        })
    } else {
        None
    };

    // check that the number of parameters and the number of the arguments is the same
    check!(
        check_function_arguments_arity(
            args_buf.len(),
            &method,
            &call_path,
            is_method_call_syntax_used
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // unify the types of the arguments with the types of the parameters from the function declaration
    let typed_arguments_with_names = check!(
        unify_arguments_and_parameters(ctx.by_ref(), args_buf, &method.parameters),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Retrieve the implemented traits for the type of the return type and
    // insert them in the broader namespace.
    ctx.namespace
        .insert_trait_implementation_for_type(engines, method.return_type.type_id);

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = check!(
        TypeParameter::gather_decl_mapping_from_trait_constraints(
            ctx.by_ref(),
            &method.type_parameters,
            &call_path.span()
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    method.replace_decls(&decl_mapping, ctx.engines());
    let return_type = method.return_type.type_id;
    let new_decl_ref = decl_engine
        .insert(method)
        .with_parent(decl_engine, (*decl_ref.id()).into());

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params: contract_call_params_map,
            arguments: typed_arguments_with_names,
            fn_ref: new_decl_ref,
            selector,
            type_binding: Some(method_name_binding.strip_inner()),
            call_path_typeid: Some(call_path_typeid),
        },
        return_type,
        span,
    };

    ok(exp, warnings, errors)
}

/// Unifies the types of the arguments with the types of the parameters. Returns
/// a list of the arguments with the names of the corresponding parameters.
fn unify_arguments_and_parameters(
    ctx: TypeCheckContext,
    arguments: VecDeque<ty::TyExpression>,
    parameters: &[ty::TyFunctionParameter],
) -> CompileResult<Vec<(Ident, ty::TyExpression)>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();
    let mut typed_arguments_and_names = vec![];

    for (arg, param) in arguments.into_iter().zip(parameters.iter()) {
        // unify the type of the argument with the type of the param
        check!(
            CompileResult::from(type_engine.unify_with_self(
                decl_engine,
                arg.return_type,
                param.type_argument.type_id,
                ctx.self_type(),
                &arg.span,
                "This argument's type is not castable to the declared parameter type.",
                Some(CompileError::ArgumentParameterTypeMismatch {
                    span: arg.span.clone(),
                    provided: engines.help_out(arg.return_type).to_string(),
                    should_be: engines.help_out(param.type_argument.type_id).to_string(),
                })
            )),
            continue,
            warnings,
            errors
        );

        typed_arguments_and_names.push((param.name.clone(), arg));
    }

    if errors.is_empty() {
        ok(typed_arguments_and_names, warnings, errors)
    } else {
        err(warnings, errors)
    }
}

pub(crate) fn resolve_method_name(
    mut ctx: TypeCheckContext,
    method_name: &mut TypeBinding<MethodName>,
    arguments: VecDeque<ty::TyExpression>,
) -> CompileResult<(DeclRefFunction, TypeId)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    // retrieve the function declaration using the components of the method name
    let (decl_ref, type_id) = match &method_name.inner {
        MethodName::FromType {
            call_path_binding,
            method_name,
        } => {
            // type check the call path
            let type_id = check!(
                call_path_binding.type_check_with_type_info(&mut ctx),
                type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                warnings,
                errors
            );

            // find the module that the symbol is in
            let type_info_prefix = ctx
                .namespace
                .find_module_path(&call_path_binding.inner.prefixes);
            check!(
                ctx.namespace.root().check_submodule(&type_info_prefix),
                return err(warnings, errors),
                warnings,
                errors
            );

            // find the method
            let decl_ref = check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &type_info_prefix,
                    method_name,
                    ctx.self_type(),
                    &arguments,
                    engines,
                    ctx.experimental_private_modules_enabled()
                ),
                return err(warnings, errors),
                warnings,
                errors
            );

            (decl_ref, type_id)
        }
        MethodName::FromTrait { call_path } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace.find_module_path(&call_path.prefixes);

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| type_engine.insert(decl_engine, TypeInfo::Unknown));

            // find the method
            let decl_ref = check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &module_path,
                    &call_path.suffix,
                    ctx.self_type(),
                    &arguments,
                    engines,
                    ctx.experimental_private_modules_enabled(),
                ),
                return err(warnings, errors),
                warnings,
                errors
            );

            (decl_ref, type_id)
        }
        MethodName::FromModule { method_name } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace.find_module_path(vec![]);

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| type_engine.insert(decl_engine, TypeInfo::Unknown));

            // find the method
            let decl_ref = check!(
                ctx.namespace.find_method_for_type(
                    type_id,
                    &module_path,
                    method_name,
                    ctx.self_type(),
                    &arguments,
                    engines,
                    ctx.experimental_private_modules_enabled(),
                ),
                return err(warnings, errors),
                warnings,
                errors
            );

            (decl_ref, type_id)
        }
    };

    let mut func_decl = decl_engine.get_function(&decl_ref);

    // monomorphize the function declaration
    let method_name_span = method_name.span();
    check!(
        ctx.monomorphize(
            &mut func_decl,
            method_name.type_arguments.to_vec_mut(),
            EnforceTypeArguments::No,
            &method_name_span,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    if let Some(implementing_type) = &func_decl.implementing_type {
        func_decl
            .body
            .update_constant_expression(engines, implementing_type);
    }

    let decl_ref = ctx
        .decl_engine
        .insert(func_decl)
        .with_parent(ctx.decl_engine, (*decl_ref.id()).into());

    ok((decl_ref, type_id), warnings, errors)
}
