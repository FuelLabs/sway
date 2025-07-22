use crate::{
    decl_engine::{
        engine::{DeclEngineGet, DeclEngineGetParsedDeclId, DeclEngineReplace},
        DeclEngineInsert, DeclRefFunction, ReplaceDecls, UpdateConstantExpression,
    },
    language::{
        parsed::*,
        ty::{self, TyDecl, TyExpression, TyFunctionSig},
        *,
    },
    semantic_analysis::*,
    type_system::*,
};
use ast_elements::{
    type_argument::GenericTypeArgument,
    type_parameter::{ConstGenericExpr, GenericTypeParameter},
};
use ast_node::typed_expression::check_function_arguments_arity;
use indexmap::IndexMap;
use itertools::izip;
use std::collections::{BTreeMap, HashMap, VecDeque};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{constants, BaseIdent, IdentUnique, Named};
use sway_types::{constants::CONTRACT_CALL_COINS_PARAMETER_NAME, Spanned};
use sway_types::{Ident, Span};

#[allow(clippy::too_many_arguments)]
pub(crate) fn type_check_method_application(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    mut method_name_binding: TypeBinding<MethodName>,
    contract_call_params: Vec<StructExpressionField>,
    arguments: &[Expression],
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();
    let coercion_check = UnifyCheck::coercion(engines);

    // type check the function arguments (1st pass)
    // Some arguments may fail on this first pass because they may require the type_annotation to the parameter type.
    // If they fail the args_opt_buf will contain a None value.
    let mut args_opt_buf = VecDeque::new();
    for (index, arg) in arguments.iter().enumerate() {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.new_unknown());

        // Ignore errors in method parameters
        // On the second pass we will throw the errors if they persist.
        let arg_handler = Handler::default();
        let arg_opt = ty::TyExpression::type_check(&arg_handler, ctx, arg).ok();

        let has_errors = arg_handler.has_errors();
        let has_numerics = arg_opt
            .as_ref()
            .map(|x| {
                x.return_type
                    .extract_inner_types(engines, IncludeSelf::Yes)
                    .iter()
                    .any(|x| matches!(&*engines.te().get(*x), TypeInfo::Numeric))
            })
            .unwrap_or_default();
        let needs_second_pass = has_errors || has_numerics;

        if index == 0 {
            // We want to emit errors in the self parameter and ignore TraitConstraintNotSatisfied with Placeholder
            // which may be recoverable on the second pass.
            arg_handler.retain_err(|e| {
                if let CompileError::TraitConstraintNotSatisfied { type_id, .. } = e {
                    !matches!(
                        *type_engine.get(TypeId::from(*type_id)),
                        TypeInfo::Placeholder(_)
                    )
                } else {
                    true
                }
            });
            handler.append(arg_handler.clone());
        }

        args_opt_buf.push_back((arg_opt, arg_handler, needs_second_pass));
    }

    // resolve the method name to a typed function declaration and type_check
    let arguments_types = args_opt_buf
        .iter()
        .map(|(arg, _, _has_errors)| match arg {
            Some(arg) => arg.return_type,
            None => type_engine.new_unknown(),
        })
        .collect::<Vec<_>>();
    let method_result = resolve_method_name(
        handler,
        ctx.by_ref(),
        &method_name_binding,
        &arguments_types,
    );

    // In case resolve_method_name fails throw argument errors.
    let (original_decl_ref, call_path_typeid) = if let Err(e) = method_result {
        for (_, arg_handler, _) in args_opt_buf.iter() {
            handler.append(arg_handler.clone());
        }
        return Err(e);
    } else {
        method_result.unwrap()
    };

    // Prepare const generics materialization
    let mut const_generics = BTreeMap::new();

    let original_decl = engines.de().get(original_decl_ref.id());
    let has_const_generic_parameters = original_decl
        .type_parameters
        .iter()
        .any(|x| matches!(x, TypeParameter::Const(_)));
    if has_const_generic_parameters {
        let a = engines.te().get(
            engines.de().get(original_decl_ref.id()).parameters[0]
                .type_argument
                .type_id(),
        );
        let b = engines
            .te()
            .get(args_opt_buf[0].0.as_ref().unwrap().return_type);
        match (&*a, &*b) {
            (
                TypeInfo::Array(_, Length(ConstGenericExpr::AmbiguousVariableExpression { ident })),
                TypeInfo::Array(_, Length(ConstGenericExpr::Literal { val, .. })),
            ) => {
                const_generics.insert(
                    ident.as_str().to_string(),
                    TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::U64(*val as u64)),
                        return_type: engines.te().id_of_u64(),
                        span: Span::dummy(),
                    },
                );
            }
            (
                TypeInfo::StringArray(Length(ConstGenericExpr::AmbiguousVariableExpression {
                    ident,
                })),
                TypeInfo::StringArray(Length(ConstGenericExpr::Literal { val, .. })),
            ) => {
                const_generics.insert(
                    ident.as_str().to_string(),
                    TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::U64(*val as u64)),
                        return_type: engines.te().id_of_u64(),
                        span: Span::dummy(),
                    },
                );
            }
            _ => {}
        }
    }

    let mut fn_ref = monomorphize_method(
        handler,
        ctx.by_ref(),
        original_decl_ref.clone(),
        method_name_binding.type_arguments.to_vec_mut(),
        const_generics,
    )?;

    let mut method = (*decl_engine.get_function(&fn_ref)).clone();

    // unify method return type with current ctx.type_annotation().
    type_engine.unify_with_generic(
        handler,
        engines,
        method.return_type.type_id(),
        ctx.type_annotation(),
        &method_name_binding.span(),
        "Function return type does not match up with local type annotation.",
        || None,
    );

    // type check the function arguments (2nd pass)
    let mut args_buf = VecDeque::new();
    for (arg, index, arg_opt) in izip!(arguments.iter(), 0.., args_opt_buf.iter().cloned()) {
        let param_index = if method.is_contract_call {
            if index == 0 {
                if let (Some(arg), _, _) = arg_opt {
                    args_buf.push_back(arg);
                }
                continue;
            }
            index - 1 //contract call methods don't have self parameter.
        } else {
            index
        };

        if let (Some(arg), _, false) = arg_opt {
            if let Some(param) = method.parameters.get(param_index) {
                if coercion_check.check(arg.return_type, param.type_argument.type_id()) {
                    // If argument type coerces to resolved method parameter type skip second type_check.
                    args_buf.push_back(arg);
                    continue;
                }
            } else {
                args_buf.push_back(arg);
                continue;
            }
        }

        // We type check the argument expression again this time throwing out the error.
        let ctx = if let Some(param) = method.parameters.get(param_index) {
            // We now try to type check it again, this time with the type annotation.
            ctx.by_ref()
                .with_help_text(
                    "Function application argument type must match function parameter type.",
                )
                .with_type_annotation(param.type_argument.type_id())
        } else {
            ctx.by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.new_unknown())
        };

        args_buf.push_back(
            ty::TyExpression::type_check(handler, ctx, arg)
                .unwrap_or_else(|err| ty::TyExpression::error(err, span.clone(), engines)),
        );
    }

    // check the method visibility
    if span.source_id() != method.span.source_id() && method.visibility.is_private() {
        return Err(handler.emit_err(CompileError::CallingPrivateLibraryMethod {
            name: method.name.as_str().to_string(),
            span,
        }));
    }

    if !method.is_contract_call && !contract_call_params.is_empty() {
        handler.emit_err(CompileError::CallParamForNonContractCallMethod {
            span: contract_call_params[0].name.span(),
        });
    }

    // generate the map of the contract call params
    let mut untyped_contract_call_params_map = std::collections::HashMap::new();
    let mut contract_call_params_map = IndexMap::new();
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
                    untyped_contract_call_params_map
                        .insert(param.name.to_string(), param.value.clone());
                    let type_annotation = if param.name.span().as_str()
                        != constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME
                    {
                        type_engine.id_of_u64()
                    } else {
                        type_engine.id_of_b256()
                    };
                    let ctx = ctx
                        .by_ref()
                        .with_help_text("")
                        .with_type_annotation(type_annotation);
                    contract_call_params_map.insert(
                        param.name.to_string(),
                        ty::TyExpression::type_check(handler, ctx, &param.value).unwrap_or_else(
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
            if coins_analysis::possibly_nonzero_u64_expression(&ctx, coins_expr)
                && !method
                    .attributes
                    .has_any_of_kind(crate::transform::AttributeKind::Payable)
            {
                return Err(
                    handler.emit_err(CompileError::CoinsPassedToNonPayableMethod {
                        fn_name: method.name.clone(),
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
            if let Some(first_arg) = args_buf.front() {
                // check if the user calls an ABI supertrait's method (those are private)
                // as a contract method
                if let TypeInfo::ContractCaller { .. } = &*type_engine.get(first_arg.return_type) {
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
                .first()
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
                let unknown_decl = ctx.resolve_symbol(&Handler::default(), name)?;

                let is_decl_mutable = match unknown_decl {
                    ty::TyDecl::ConstantDecl { .. } => false,
                    _ => {
                        let variable_decl = unknown_decl
                            .expect_variable(handler, ctx.engines())
                            .cloned()?;
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
    ) = (args_buf.front(), method.parameters.first())
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
                (
                    TypeInfo::Custom {
                        qualified_call_path: call_path,
                        ..
                    },
                    ..,
                ) => call_path.call_path.clone().suffix,
                (_, ident) => ident.clone(),
            });

            CallPath {
                prefixes,
                suffix: method_name,
                callpath_type: call_path_binding.inner.callpath_type,
            }
        }
        MethodName::FromModule { method_name } => CallPath {
            prefixes: vec![],
            suffix: method_name,
            callpath_type: CallPathType::Ambiguous,
        },
        MethodName::FromTrait { call_path } => call_path,
        MethodName::FromQualifiedPathRoot { method_name, .. } => CallPath {
            prefixes: vec![],
            suffix: method_name,
            callpath_type: CallPathType::Ambiguous,
        },
    };

    // build the function selector
    let selector = if method.is_contract_call {
        let contract_caller = args_buf.pop_front();
        let contract_address = match contract_caller
            .clone()
            .map(|x| (*type_engine.get(x.return_type)).clone())
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
        let func_selector = if ctx.experimental.new_encoding {
            None
        } else {
            Some(
                method
                    .to_fn_selector_value(handler, engines)
                    .unwrap_or([0; 4]),
            )
        };
        Some(ty::ContractCallParams {
            func_selector,
            contract_address: contract_address.clone(),
            contract_caller: Box::new(contract_caller.unwrap()),
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

    let old_arguments = arguments;
    let arguments = method
        .parameters
        .iter()
        .map(|m| m.name.clone())
        .zip(args_buf)
        .collect::<Vec<_>>();

    // unify the types of the arguments with the types of the parameters from the function declaration
    let arguments =
        unify_arguments_and_parameters(handler, ctx.by_ref(), &arguments, &method.parameters)?;

    if ctx.experimental.new_encoding && method.is_contract_call {
        fn call_contract_call(
            ctx: &mut TypeCheckContext,
            original_span: Span,
            return_type: TypeId,
            method_name_expr: Expression,
            _caller: Expression,
            arguments: Vec<Expression>,
            typed_arguments: Vec<TypeId>,
            coins_expr: Expression,
            asset_id_expr: Expression,
            gas_expr: Expression,
        ) -> Expression {
            let tuple_args_type_id = ctx
                .engines
                .te()
                .insert_tuple_without_annotations(ctx.engines, typed_arguments);
            Expression {
                kind: ExpressionKind::FunctionApplication(Box::new(
                    FunctionApplicationExpression {
                        call_path_binding: TypeBinding {
                            inner: CallPath {
                                prefixes: vec![],
                                suffix: Ident::new_no_span("contract_call".into()),
                                callpath_type: CallPathType::Ambiguous,
                            },
                            type_arguments: TypeArgs::Regular(vec![
                                GenericArgument::Type(GenericTypeArgument {
                                    type_id: return_type,
                                    initial_type_id: return_type,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }),
                                GenericArgument::Type(GenericTypeArgument {
                                    type_id: tuple_args_type_id,
                                    initial_type_id: tuple_args_type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }),
                            ]),
                            span: Span::dummy(),
                        },
                        resolved_call_path_binding: None,
                        arguments: vec![
                            Expression {
                                kind: ExpressionKind::Literal(Literal::B256([0u8; 32])),
                                span: Span::dummy(),
                            },
                            method_name_expr,
                            as_tuple(arguments),
                            coins_expr,
                            asset_id_expr,
                            gas_expr,
                        ],
                    },
                )),
                span: original_span,
            }
        }

        fn string_slice_literal(ident: &BaseIdent) -> Expression {
            Expression {
                kind: ExpressionKind::Literal(Literal::String(ident.span())),
                span: ident.span(),
            }
        }

        fn as_tuple(elements: Vec<Expression>) -> Expression {
            Expression {
                kind: ExpressionKind::Tuple(elements),
                span: Span::dummy(),
            }
        }

        let gas_expr = untyped_contract_call_params_map
            .remove(constants::CONTRACT_CALL_GAS_PARAMETER_NAME)
            .unwrap_or_else(|| Expression {
                kind: ExpressionKind::Literal(Literal::U64(u64::MAX)),
                span: Span::dummy(),
            });
        let coins_expr = untyped_contract_call_params_map
            .remove(constants::CONTRACT_CALL_COINS_PARAMETER_NAME)
            .unwrap_or_else(|| Expression {
                kind: ExpressionKind::Literal(Literal::U64(0)),
                span: Span::dummy(),
            });
        let asset_id_expr = untyped_contract_call_params_map
            .remove(constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME)
            .unwrap_or_else(|| Expression {
                kind: ExpressionKind::Literal(Literal::B256([0u8; 32])),
                span: Span::dummy(),
            });

        // We need all impls of return type to be in scope, so that at call place we have access to its
        // AbiDecode impl.
        for type_id in method
            .return_type
            .type_id()
            .extract_inner_types(engines, IncludeSelf::Yes)
        {
            ctx.impls_import(engines, type_id);
        }

        let args = old_arguments.iter().skip(1).cloned().collect();
        let contract_call = call_contract_call(
            &mut ctx,
            span,
            method.return_type.type_id(),
            string_slice_literal(&method.name),
            old_arguments.first().cloned().unwrap(),
            args,
            arguments.iter().map(|x| x.1.return_type).collect(),
            coins_expr,
            asset_id_expr,
            gas_expr,
        );
        let mut expr = TyExpression::type_check(handler, ctx.by_ref(), &contract_call)?;

        // We need to "fix" contract_id here because it was created with zero
        // given that we only have it as TyExpression, therefore can only use it after we type_check
        // `expr``
        match &mut expr.expression {
            ty::TyExpressionVariant::FunctionApplication {
                arguments,
                contract_caller,
                ..
            } => {
                let selector = selector.unwrap();
                arguments[0].1 = (*selector.contract_address).clone();
                *contract_caller = Some(selector.contract_caller);
            }
            _ => unreachable!(),
        }

        return Ok(expr);
    }

    // Unify method type parameters with implementing type type parameters.
    if let Some(implementing_for_typeid) = method.implementing_for_typeid {
        if let Some(TyDecl::ImplSelfOrTrait(t)) = method.implementing_type.clone() {
            let t = &engines.de().get(&t.decl_id).implementing_for;
            if let TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } = &*type_engine.get(t.initial_type_id())
            {
                // Method type parameters that have is_from_parent set to true use the base ident as defined in
                // in the impl trait. The type parameter name may be different in the Struct or Enum.
                // Thus we use the index in the Struct's or Enum's type parameter the impl trait type parameter
                // was used on.
                let mut names_index = HashMap::<Ident, usize>::new();
                for (index, t_arg) in type_arguments.iter().enumerate() {
                    if let TypeInfo::Custom {
                        qualified_call_path,
                        ..
                    } = &*type_engine.get(t_arg.initial_type_id())
                    {
                        names_index.insert(qualified_call_path.call_path.suffix.clone(), index);
                    }
                }
                let implementing_type_parameters =
                    implementing_for_typeid.get_type_parameters(engines);
                if let Some(implementing_type_parameters) = implementing_type_parameters {
                    for p in method.type_parameters.clone() {
                        let Some(p) = p.as_type_parameter() else {
                            continue;
                        };

                        if p.is_from_parent {
                            if let Some(impl_type_param) =
                                names_index.get(&p.name).and_then(|type_param_index| {
                                    implementing_type_parameters.get(*type_param_index)
                                })
                            {
                                let impl_type_param = impl_type_param
                                    .as_type_parameter()
                                    .expect("only works with type parameters");
                                handler.scope(|handler| {
                                    type_engine.unify_with_generic(
                                        handler,
                                        engines,
                                        p.type_id,
                                        impl_type_param.type_id,
                                        &call_path.span(),
                                        "Function type parameter does not match up with implementing type type parameter.",
                                        || None,
                                    );
                                    Ok(())
                                })?;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut method_return_type_id = method.return_type.type_id();

    let method_ident: IdentUnique = method.name.clone().into();
    let method_sig = TyFunctionSig::from_fn_decl(&method);

    if let Some(cached_fn_ref) =
        ctx.engines()
            .qe()
            .get_function(engines, &method_ident, method_sig.clone())
    {
        fn_ref = cached_fn_ref;
    } else {
        if let Some(TyDecl::ImplSelfOrTrait(t)) = method.implementing_type.clone() {
            let t = &engines.de().get(&t.decl_id).implementing_for;
            if let TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
            } = &*type_engine.get(t.initial_type_id())
            {
                let mut subst_type_parameters = vec![];
                let mut subst_type_arguments = vec![];

                let mut names_type_ids = HashMap::<Ident, TypeId>::new();
                if let Some(type_arguments) = type_arguments {
                    for t_arg in type_arguments.iter() {
                        if let TypeInfo::Custom {
                            qualified_call_path,
                            ..
                        } = &*type_engine.get(t_arg.initial_type_id())
                        {
                            names_type_ids.insert(
                                qualified_call_path.call_path.suffix.clone(),
                                t_arg.type_id(),
                            );
                        }
                    }
                }

                // This handles the case of substituting the generic blanket type by call_path_typeid.
                for p in method.type_parameters.iter() {
                    if p.name().as_str() == qualified_call_path.call_path.suffix.as_str() {
                        subst_type_parameters.push(t.initial_type_id());
                        subst_type_arguments.push(call_path_typeid);
                        break;
                    }
                }

                // This will subst inner method_application placeholders with the already resolved
                // current method application type parameter
                for p in method
                    .type_parameters
                    .iter()
                    .filter(|x| x.as_type_parameter().is_some())
                {
                    if names_type_ids.contains_key(p.name()) {
                        let type_id = p
                            .as_type_parameter()
                            .expect("only works with type parameters")
                            .type_id;
                        subst_type_parameters.push(engines.te().new_placeholder(p.clone()));
                        subst_type_arguments.push(type_id);
                    }
                }

                let type_subst = TypeSubstMap::from_type_parameters_and_type_arguments(
                    subst_type_parameters,
                    subst_type_arguments,
                );

                method.subst(&SubstTypesContext::new(
                    engines,
                    &type_subst,
                    !ctx.code_block_first_pass(),
                ));
            }
        }

        if !ctx.code_block_first_pass() {
            // Handle the trait constraints. This includes checking to see if the trait
            // constraints are satisfied and replacing old decl ids based on the
            // constraint with new decl ids based on the new type.
            let decl_mapping = GenericTypeParameter::gather_decl_mapping_from_trait_constraints(
                handler,
                ctx.by_ref(),
                &method.type_parameters,
                method.name.as_str(),
                &call_path.span(),
            )
            .ok();

            if let Some(decl_mapping) = decl_mapping {
                method.replace_decls(&decl_mapping, handler, &mut ctx)?;
            }
        }

        let method_sig = TyFunctionSig::from_fn_decl(&method);

        method_return_type_id = method.return_type.type_id();
        decl_engine.replace(*fn_ref.id(), method.clone());

        if !ctx.code_block_first_pass()
            && method_sig.is_concrete(engines)
            && method.is_type_check_finalized
            && !method.is_trait_method_dummy
        {
            ctx.engines()
                .qe()
                .insert_function(engines, method_ident, method_sig, fn_ref.clone());
        }
    }

    let expression = ty::TyExpressionVariant::FunctionApplication {
        call_path,
        arguments,
        fn_ref,
        selector,
        type_binding: Some(method_name_binding.strip_inner()),
        call_path_typeid: Some(call_path_typeid),
        contract_call_params: contract_call_params_map,
        contract_caller: None,
    };

    let exp = ty::TyExpression {
        expression,
        return_type: method_return_type_id,
        span,
    };

    Ok(exp)
}

/// Unifies the types of the arguments with the types of the parameters. Returns
/// a list of the arguments with the names of the corresponding parameters.
fn unify_arguments_and_parameters(
    handler: &Handler,
    ctx: TypeCheckContext,
    arguments: &[(BaseIdent, ty::TyExpression)],
    parameters: &[ty::TyFunctionParameter],
) -> Result<Vec<(Ident, ty::TyExpression)>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();
    let mut typed_arguments_and_names = vec![];

    handler.scope(|handler| {
        for ((_, arg), param) in arguments.iter().zip(parameters.iter()) {
            // unify the type of the argument with the type of the param
            let unify_res = handler.scope(|handler| {
                type_engine.unify_with_generic(
                    handler,
                    engines,
                    arg.return_type,
                    param.type_argument.type_id(),
                    &arg.span,
                    "This argument's type is not castable to the declared parameter type.",
                    || {
                        Some(CompileError::ArgumentParameterTypeMismatch {
                            span: arg.span.clone(),
                            provided: engines.help_out(arg.return_type).to_string(),
                            should_be: engines.help_out(param.type_argument.type_id()).to_string(),
                        })
                    },
                );
                Ok(())
            });
            if unify_res.is_err() {
                continue;
            }

            typed_arguments_and_names.push((param.name.clone(), arg.clone()));
        }
        Ok(typed_arguments_and_names)
    })
}

pub(crate) fn resolve_method_name(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    method_name: &TypeBinding<MethodName>,
    arguments_types: &[TypeId],
) -> Result<(DeclRefFunction, TypeId), ErrorEmitted> {
    ctx.engines
        .obs()
        .raise_on_before_method_resolution(&ctx, method_name, arguments_types);

    let type_engine = ctx.engines.te();
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
                .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

            // find the module that the symbol is in
            let type_info_prefix = &call_path_binding
                .inner
                .to_fullpath(engines, ctx.namespace())
                .prefixes;
            ctx.namespace()
                .require_module_from_absolute_path(handler, type_info_prefix)?;

            // find the method
            let decl_ref = ctx.find_method_for_type(
                handler,
                type_id,
                type_info_prefix,
                method_name,
                ctx.type_annotation(),
                arguments_types,
                None,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromTrait { call_path } => {
            // find the module that the symbol is in
            let module_path = match call_path.callpath_type {
                CallPathType::RelativeToPackageRoot => {
                    let mut path = vec![ctx.namespace().current_package_name().clone()];
                    for ident in call_path.prefixes.iter() {
                        path.push(ident.clone())
                    }
                    path
                }
                CallPathType::Full => call_path.prefixes.clone(),
                CallPathType::Ambiguous => {
                    if ctx
                        .namespace()
                        .current_module()
                        .submodules()
                        .contains_key(call_path.prefixes.first().unwrap().as_str())
                    {
                        ctx.namespace().prepend_module_path(&call_path.prefixes)
                    } else {
                        call_path.prefixes.clone()
                    }
                }
            };

            // find the type of the first argument
            let type_id = arguments_types
                .first()
                .cloned()
                .unwrap_or_else(|| type_engine.new_unknown());

            // find the method
            let decl_ref = ctx.find_method_for_type(
                handler,
                type_id,
                &module_path,
                &call_path.suffix,
                ctx.type_annotation(),
                arguments_types,
                None,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromModule { method_name } => {
            // find the module that the symbol is in
            let module_path = ctx.namespace().current_mod_path();

            // find the type of the first argument
            let type_id = arguments_types
                .first()
                .cloned()
                .unwrap_or_else(|| type_engine.new_unknown());

            // find the method
            let decl_ref = ctx.find_method_for_type(
                handler,
                type_id,
                module_path.as_slice(),
                method_name,
                ctx.type_annotation(),
                arguments_types,
                None,
            )?;

            (decl_ref, type_id)
        }
        MethodName::FromQualifiedPathRoot {
            ty,
            as_trait,
            method_name,
        } => {
            // type check the call path
            let type_id = ty.type_id();

            // find the module that the symbol is in
            let module_path = ctx.namespace().current_mod_path();

            // find the method
            let decl_ref = ctx.find_method_for_type(
                handler,
                type_id,
                module_path,
                method_name,
                ctx.type_annotation(),
                arguments_types,
                Some(*as_trait),
            )?;

            (decl_ref, type_id)
        }
    };

    ctx.engines.obs().raise_on_after_method_resolution(
        &ctx,
        method_name,
        arguments_types,
        decl_ref.clone(),
        type_id,
    );

    Ok((decl_ref, type_id))
}

pub(crate) fn monomorphize_method(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    decl_ref: DeclRefFunction,
    type_arguments: &mut [GenericArgument],
    const_generics: BTreeMap<String, TyExpression>,
) -> Result<DeclRefFunction, ErrorEmitted> {
    let engines = ctx.engines();
    let decl_engine = engines.de();
    let mut func_decl = (*decl_engine.get_function(&decl_ref)).clone();

    // monomorphize the function declaration
    ctx.monomorphize(
        handler,
        &mut func_decl,
        type_arguments,
        const_generics,
        EnforceTypeArguments::No,
        &decl_ref.span(),
    )?;

    if let Some(implementing_type) = &func_decl.implementing_type {
        func_decl
            .body
            .update_constant_expression(engines, implementing_type);
    }

    let decl_ref = decl_engine
        .insert(
            func_decl,
            decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
        )
        .with_parent(decl_engine, (*decl_ref.id()).into());

    Ok(decl_ref)
}
