use crate::{
    decl_engine::{DeclEngineInsert, DeclRefFunction, ReplaceDecls, UpdateConstantExpression},
    language::{parsed::*, ty, *},
    namespace::TryInsertingTraitImplOnFailure,
    semantic_analysis::*,
    type_system::*,
};
use ast_node::typed_expression::check_function_arguments_arity;
use std::collections::{HashMap, VecDeque};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{constants, integer_bits::IntegerBits};
use sway_types::{constants::CONTRACT_CALL_COINS_PARAMETER_NAME, Spanned};
use sway_types::{Ident, Span};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    mut method_name_binding: TypeBinding<MethodName>,
    contract_call_params: Vec<StructExpressionField>,
    arguments: Vec<Expression>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    // type check the function arguments
    let mut args_buf = VecDeque::new();
    for arg in &arguments {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
        args_buf.push_back(
            ty::TyExpression::type_check(handler, ctx, arg.clone())
                .unwrap_or_else(|err| ty::TyExpression::error(err, span.clone(), engines)),
        );
    }

    // resolve the method name to a typed function declaration and type_check
    let (decl_ref, call_path_typeid) = resolve_method_name(
        handler,
        ctx.by_ref(),
        &mut method_name_binding,
        args_buf.clone(),
    )?;
    let mut method = decl_engine.get_function(&decl_ref);

    // check the method visibility
    if span.source_id() != method.span.source_id() && method.visibility.is_private() {
        return Err(handler.emit_err(CompileError::CallingPrivateLibraryMethod {
            name: method.name.as_str().to_string(),
            span,
        }));
    }

    // check the function storage purity
    if !method.is_contract_call {
        // 'method.purity' is that of the callee, 'opts.purity' of the caller.
        if !ctx.purity().can_call(method.purity) {
            handler.emit_err(CompileError::StorageAccessMismatch {
                attrs: promote_purity(ctx.purity(), method.purity).to_attribute_syntax(),
                span: method_name_binding.inner.easy_name().span(),
            });
        }
        if !contract_call_params.is_empty() {
            handler.emit_err(CompileError::CallParamForNonContractCallMethod {
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
                handler.emit_err(CompileError::ContractCallParamRepeated {
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
                        engines,
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
                        ty::TyExpression::type_check(handler, ctx, param.value).unwrap_or_else(
                            |err| ty::TyExpression::error(err, span.clone(), engines),
                        ),
                    );
                }
                _ => {
                    handler.emit_err(CompileError::UnrecognizedContractParam {
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
                return Err(
                    handler.emit_err(CompileError::CoinsPassedToNonPayableMethod {
                        fn_name: method.name,
                        span,
                    }),
                );
            }
        }
    }

    // If this function is being called with method call syntax, a.b(c),
    // then make sure the first parameter is self, else issue an error.
    let mut is_method_call_syntax_used = false;
    if !method.is_contract_call {
        if let MethodName::FromModule { ref method_name } = method_name_binding.inner {
            if let Some(first_arg) = args_buf.get(0) {
                // check if the user calls an ABI supertrait's method (those are private)
                // as a contract method
                if let TypeInfo::ContractCaller { .. } = type_engine.get(first_arg.return_type) {
                    return Err(handler.emit_err(
                        CompileError::AbiSupertraitMethodCallAsContractCall {
                            fn_name: method_name.clone(),
                            span,
                        },
                    ));
                }
            }
            is_method_call_syntax_used = true;
            let is_first_param_self = method
                .parameters
                .get(0)
                .map(|f| f.is_self())
                .unwrap_or_default();
            if !is_first_param_self {
                return Err(
                    handler.emit_err(CompileError::AssociatedFunctionCalledAsMethod {
                        fn_name: method_name.clone(),
                        span,
                    }),
                );
            }
        }
    }

    // Validate mutability of self. Check that the variable that the method is called on is mutable
    // _if_ the method requires mutable self.
    fn mutability_check(
        handler: &Handler,
        ctx: &TypeCheckContext,
        method_name_binding: &TypeBinding<MethodName>,
        span: &Span,
        exp: &ty::TyExpressionVariant,
    ) -> Result<(), ErrorEmitted> {
        match exp {
            ty::TyExpressionVariant::VariableExpression { name, .. } => {
                let unknown_decl = ctx
                    .namespace
                    .resolve_symbol(&Handler::default(), name)
                    .cloned()?;

                let is_decl_mutable = match unknown_decl {
                    ty::TyDecl::ConstantDecl { .. } => false,
                    _ => {
                        let variable_decl = unknown_decl.expect_variable(handler).cloned()?;
                        variable_decl.mutability.is_mutable()
                    }
                };

                if !is_decl_mutable {
                    return Err(handler.emit_err(CompileError::MethodRequiresMutableSelf {
                        method_name: method_name_binding.inner.easy_name(),
                        variable_name: name.clone(),
                        span: span.clone(),
                    }));
                }

                Ok(())
            }
            ty::TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                mutability_check(handler, ctx, method_name_binding, span, &prefix.expression)
            }
            _ => Ok(()),
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
            mutability_check(handler, &ctx, &method_name_binding, &span, exp)?;
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
        MethodName::FromQualifiedPathRoot { method_name, .. } => CallPath {
            prefixes: vec![],
            suffix: method_name,
            is_absolute: false,
        },
    };

    // build the function selector
    let selector = if method.is_contract_call {
        let contract_caller = args_buf.pop_front();
        let contract_address = match contract_caller
            .clone()
            .map(|x| type_engine.get(x.return_type))
        {
            Some(TypeInfo::ContractCaller { address, .. }) => match address {
                Some(address) => address,
                None => {
                    return Err(handler.emit_err(CompileError::ContractAddressMustBeKnown {
                        span: call_path.span(),
                    }));
                }
            },
            None => {
                return Err(handler.emit_err(CompileError::ContractCallsItsOwnMethod { span }));
            }
            _ => {
                return Err(handler.emit_err(CompileError::Internal(
                    "Attempted to find contract address of non-contract-call.",
                    span,
                )));
            }
        };
        let func_selector = method
            .to_fn_selector_value(handler, engines)
            .unwrap_or([0; 4]);
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

    check_function_arguments_arity(
        handler,
        args_buf.len(),
        &method,
        &call_path,
        is_method_call_syntax_used,
    )?;

    // unify the types of the arguments with the types of the parameters from the function declaration
    let typed_arguments_with_names =
        unify_arguments_and_parameters(handler, ctx.by_ref(), args_buf, &method.parameters)?;

    // Retrieve the implemented traits for the type of the return type and
    // insert them in the broader namespace.
    ctx.namespace
        .insert_trait_implementation_for_type(engines, method.return_type.type_id);

    // Handle the trait constraints. This includes checking to see if the trait
    // constraints are satisfied and replacing old decl ids based on the
    // constraint with new decl ids based on the new type.
    let decl_mapping = TypeParameter::gather_decl_mapping_from_trait_constraints(
        handler,
        ctx.by_ref(),
        &method.type_parameters,
        &call_path.span(),
    )?;
    method.replace_decls(&decl_mapping, handler, &mut ctx)?;
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

    Ok(exp)
}

/// Unifies the types of the arguments with the types of the parameters. Returns
/// a list of the arguments with the names of the corresponding parameters.
fn unify_arguments_and_parameters(
    handler: &Handler,
    ctx: TypeCheckContext,
    arguments: VecDeque<ty::TyExpression>,
    parameters: &[ty::TyFunctionParameter],
) -> Result<Vec<(Ident, ty::TyExpression)>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();
    let mut typed_arguments_and_names = vec![];

    handler.scope(|handler| {
        for (arg, param) in arguments.into_iter().zip(parameters.iter()) {
            // unify the type of the argument with the type of the param
            let unify_res = handler.scope(|handler| {
                type_engine.unify(
                    handler,
                    engines,
                    arg.return_type,
                    param.type_argument.type_id,
                    &arg.span,
                    "This argument's type is not castable to the declared parameter type.",
                    Some(CompileError::ArgumentParameterTypeMismatch {
                        span: arg.span.clone(),
                        provided: engines.help_out(arg.return_type).to_string(),
                        should_be: engines.help_out(param.type_argument.type_id).to_string(),
                    }),
                );
                Ok(())
            });
            if unify_res.is_err() {
                continue;
            }

            typed_arguments_and_names.push((param.name.clone(), arg));
        }
        Ok(typed_arguments_and_names)
    })
}

pub(crate) fn resolve_method_name(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    method_name: &mut TypeBinding<MethodName>,
    arguments: VecDeque<ty::TyExpression>,
) -> Result<(DeclRefFunction, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    // retrieve the function declaration using the components of the method name
    let (decl_ref, type_id) = match &method_name.inner {
        MethodName::FromType {
            call_path_binding,
            method_name,
        } => {
            // type check the call path
            let type_id = call_path_binding
                .type_check_with_type_info(handler, &mut ctx)
                .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err)));

            // find the module that the symbol is in
            let type_info_prefix = ctx
                .namespace
                .find_module_path(&call_path_binding.inner.prefixes);
            ctx.namespace
                .root()
                .check_submodule(handler, &type_info_prefix)?;

            // find the method
            let decl_ref = ctx.namespace.find_method_for_type(
                handler,
                type_id,
                &type_info_prefix,
                method_name,
                ctx.type_annotation(),
                &arguments,
                None,
                engines,
                TryInsertingTraitImplOnFailure::Yes,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromTrait { call_path } => {
            // find the module that the symbol is in
            let module_path = if !call_path.is_absolute {
                ctx.namespace.find_module_path(&call_path.prefixes)
            } else {
                let mut module_path = call_path.prefixes.clone();
                if let (Some(root_mod), Some(root_name)) = (
                    module_path.get(0).cloned(),
                    ctx.namespace.root().name.clone(),
                ) {
                    if root_mod.as_str() == root_name.as_str() {
                        module_path.remove(0);
                    }
                }
                module_path
            };

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| type_engine.insert(engines, TypeInfo::Unknown));

            // find the method
            let decl_ref = ctx.namespace.find_method_for_type(
                handler,
                type_id,
                &module_path,
                &call_path.suffix,
                ctx.type_annotation(),
                &arguments,
                None,
                engines,
                TryInsertingTraitImplOnFailure::Yes,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromModule { method_name } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace.find_module_path(vec![]);

            // find the type of the first argument
            let type_id = arguments
                .get(0)
                .map(|x| x.return_type)
                .unwrap_or_else(|| type_engine.insert(engines, TypeInfo::Unknown));

            // find the method
            let decl_ref = ctx.namespace.find_method_for_type(
                handler,
                type_id,
                &module_path,
                method_name,
                ctx.type_annotation(),
                &arguments,
                None,
                engines,
                TryInsertingTraitImplOnFailure::Yes,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromQualifiedPathRoot {
            ty,
            as_trait,
            method_name,
        } => {
            // type check the call path
            let type_id = ty.type_id;
            let type_info_prefix = vec![];

            // find the method
            let decl_ref = ctx.namespace.find_method_for_type(
                handler,
                type_id,
                &type_info_prefix,
                method_name,
                ctx.type_annotation(),
                &arguments,
                Some(as_trait.clone()),
                engines,
                TryInsertingTraitImplOnFailure::Yes,
            )?;

            (decl_ref, type_id)
        }
    };

    let mut func_decl = decl_engine.get_function(&decl_ref);

    // monomorphize the function declaration
    let method_name_span = method_name.span();
    ctx.monomorphize(
        handler,
        &mut func_decl,
        method_name.type_arguments.to_vec_mut(),
        EnforceTypeArguments::No,
        &method_name_span,
    )?;

    if let Some(implementing_type) = &func_decl.implementing_type {
        func_decl
            .body
            .update_constant_expression(engines, implementing_type);
    }

    let decl_ref = ctx
        .engines
        .de()
        .insert(func_decl)
        .with_parent(ctx.engines.de(), (*decl_ref.id()).into());

    Ok((decl_ref, type_id))
}
