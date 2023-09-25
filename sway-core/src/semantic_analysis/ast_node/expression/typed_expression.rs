mod constant_expression;
mod enum_instantiation;
mod function_application;
mod if_expression;
mod lazy_operator;
mod method_application;
mod struct_field_access;
mod struct_instantiation;
mod tuple_index_access;
mod unsafe_downcast;

use self::constant_expression::instantiate_constant_expression;

pub(crate) use self::{
    enum_instantiation::*, function_application::*, if_expression::*, lazy_operator::*,
    method_application::*, struct_field_access::*, struct_instantiation::*, tuple_index_access::*,
    unsafe_downcast::*,
};

use crate::{
    asm_lang::{virtual_ops::VirtualOp, virtual_register::VirtualRegister},
    decl_engine::*,
    language::{
        parsed::*,
        ty::{self, TyImplItem},
        *,
    },
    namespace::{IsExtendingExistingImpl, IsImplSelf},
    semantic_analysis::{expression::ReachableReport, type_check_context::EnforceTypeArguments, *},
    transform::to_parsed_lang::type_name_to_type_info_opt,
    type_system::*,
    Engines,
};

use ast_node::declaration::{insert_supertraits_into_namespace, SupertraitOf};
use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{integer_bits::IntegerBits, u256::U256, Ident, Named, Span, Spanned};

use rustc_hash::FxHashSet;

use either::Either;

use std::collections::{HashMap, VecDeque};

#[allow(clippy::too_many_arguments)]
impl ty::TyExpression {
    pub(crate) fn core_ops_eq(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        arguments: Vec<ty::TyExpression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let decl_engine = ctx.engines.de();

        let call_path = CallPath {
            prefixes: vec![
                Ident::new_with_override("core".into(), span.clone()),
                Ident::new_with_override("ops".into(), span.clone()),
            ],
            suffix: Op {
                op_variant: OpVariant::Equals,
                span: span.clone(),
            }
            .to_var_name(),
            is_absolute: true,
        };
        let mut method_name_binding = TypeBinding {
            inner: MethodName::FromTrait {
                call_path: call_path.clone(),
            },
            type_arguments: TypeArgs::Regular(vec![]),
            span: call_path.span(),
        };
        let arguments = VecDeque::from(arguments);
        let (mut decl_ref, _) = resolve_method_name(
            handler,
            ctx.by_ref(),
            &method_name_binding,
            arguments.clone(),
        )?;
        decl_ref = monomorphize_method(
            handler,
            ctx,
            decl_ref.clone(),
            method_name_binding.type_arguments.to_vec_mut(),
        )?;
        let method = decl_engine.get_function(&decl_ref);
        // check that the number of parameters and the number of the arguments is the same
        check_function_arguments_arity(handler, arguments.len(), &method, &call_path, false)?;
        let return_type = method.return_type;
        let args_and_names = method
            .parameters
            .into_iter()
            .zip(arguments)
            .map(|(param, arg)| (param.name, arg))
            .collect::<Vec<(_, _)>>();
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::FunctionApplication {
                call_path,
                contract_call_params: HashMap::new(),
                arguments: args_and_names,
                fn_ref: decl_ref,
                selector: None,
                type_binding: None,
                call_path_typeid: None,
                deferred_monomorphization: false,
            },
            return_type: return_type.type_id,
            span,
        };
        Ok(exp)
    }

    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        expr: Expression,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        let expr_span = expr.span();
        let span = expr_span.clone();
        let res = match expr.kind {
            // We've already emitted an error for the `::Error` case.
            ExpressionKind::Error(_, err) => Ok(ty::TyExpression::error(err, span, engines)),
            ExpressionKind::Literal(lit) => Ok(Self::type_check_literal(engines, lit, span)),
            ExpressionKind::AmbiguousVariableExpression(name) => {
                let call_path = CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                };
                if matches!(
                    ctx.namespace
                        .resolve_call_path(&Handler::default(), engines, &call_path)
                        .ok(),
                    Some(ty::TyDecl::EnumVariantDecl { .. })
                ) {
                    Self::type_check_delineated_path(
                        handler,
                        ctx.by_ref(),
                        TypeBinding {
                            span: call_path.span(),
                            inner: call_path,
                            type_arguments: TypeArgs::Regular(vec![]),
                        },
                        span,
                        None,
                    )
                } else {
                    Self::type_check_variable_expression(handler, ctx.by_ref(), name, span)
                }
            }
            ExpressionKind::Variable(name) => {
                Self::type_check_variable_expression(handler, ctx.by_ref(), name, span)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    arguments,
                } = *function_application_expression;
                Self::type_check_function_application(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    arguments,
                    span,
                )
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { op, lhs, rhs }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Boolean));
                Self::type_check_lazy_operator(handler, ctx, op, *lhs, *rhs, span)
            }
            ExpressionKind::CodeBlock(contents) => {
                Self::type_check_code_block(handler, ctx.by_ref(), contents, span)
            }
            // TODO if _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
            }) => Self::type_check_if_expression(
                handler,
                ctx.by_ref().with_help_text(""),
                *condition,
                *then,
                r#else.map(|e| *e),
                span,
            ),
            ExpressionKind::Match(MatchExpression { value, branches }) => {
                Self::type_check_match_expression(
                    handler,
                    ctx.by_ref().with_help_text(""),
                    *value,
                    branches,
                    span,
                )
            }
            ExpressionKind::Asm(asm) => {
                Self::type_check_asm_expression(handler, ctx.by_ref(), *asm, span)
            }
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    fields,
                } = *struct_expression;
                struct_instantiation(handler, ctx.by_ref(), call_path_binding, fields, span)
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
            }) => Self::type_check_subfield_expression(
                handler,
                ctx.by_ref(),
                *prefix,
                span,
                field_to_access,
            ),
            ExpressionKind::MethodApplication(method_application_expression) => {
                let MethodApplicationExpression {
                    method_name_binding,
                    contract_call_params,
                    arguments,
                } = *method_application_expression;
                type_check_method_application(
                    handler,
                    ctx.by_ref(),
                    method_name_binding,
                    contract_call_params,
                    arguments,
                    span,
                )
            }
            ExpressionKind::Tuple(fields) => {
                Self::type_check_tuple(handler, ctx.by_ref(), fields, span)
            }
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix,
                index,
                index_span,
            }) => Self::type_check_tuple_index(
                handler,
                ctx.by_ref(),
                *prefix,
                index,
                index_span,
                span,
            ),
            ExpressionKind::AmbiguousPathExpression(e) => {
                let AmbiguousPathExpression {
                    call_path_binding,
                    args,
                    qualified_path_root,
                } = *e;
                Self::type_check_ambiguous_path(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    span,
                    args,
                    qualified_path_root,
                )
            }
            ExpressionKind::DelineatedPath(delineated_path_expression) => {
                let DelineatedPathExpression {
                    call_path_binding,
                    args,
                } = *delineated_path_expression;
                Self::type_check_delineated_path(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    span,
                    args,
                )
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                let AbiCastExpression { abi_name, address } = *abi_cast_expression;
                Self::type_check_abi_cast(handler, ctx.by_ref(), abi_name, *address, span)
            }
            ExpressionKind::Array(array_expression) => {
                Self::type_check_array(handler, ctx.by_ref(), array_expression.contents, span)
            }
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_array_index(handler, ctx, *prefix, *index, span)
            }
            ExpressionKind::StorageAccess(StorageAccessExpression {
                field_names,
                storage_keyword_span,
            }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_storage_access(
                    handler,
                    ctx,
                    field_names,
                    storage_keyword_span,
                    &span,
                )
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                kind_binding,
                arguments,
                ..
            }) => Self::type_check_intrinsic_function(
                handler,
                ctx.by_ref(),
                kind_binding,
                arguments,
                span,
            ),
            ExpressionKind::WhileLoop(WhileLoopExpression { condition, body }) => {
                Self::type_check_while_loop(handler, ctx.by_ref(), *condition, body, span)
            }
            ExpressionKind::Break => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Break,
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    span,
                };
                Ok(expr)
            }
            ExpressionKind::Continue => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Continue,
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    span,
                };
                Ok(expr)
            }
            ExpressionKind::Reassignment(ReassignmentExpression { lhs, rhs }) => {
                Self::type_check_reassignment(handler, ctx.by_ref(), lhs, *rhs, span)
            }
            ExpressionKind::Return(expr) => {
                let ctx = ctx
                    // we use "unknown" here because return statements do not
                    // necessarily follow the type annotation of their immediate
                    // surrounding context. Because a return statement is control flow
                    // that breaks out to the nearest function, we need to type check
                    // it against the surrounding function.
                    // That is impossible here, as we don't have that information. It
                    // is the responsibility of the function declaration to type check
                    // all return statements contained within it.
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                    .with_help_text(
                        "Returned value must match up with the function return type \
                        annotation.",
                    );
                let expr_span = expr.span();
                let expr = ty::TyExpression::type_check(handler, ctx, *expr)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, expr_span, engines));
                let typed_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Return(Box::new(expr)),
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    // FIXME: This should be Yes?
                    span,
                };
                Ok(typed_expr)
            }
        };
        let mut typed_expression = match res {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        // if the return type cannot be cast into the annotation type then it is a type error
        ctx.unify_with_self(handler, typed_expression.return_type, &expr_span);

        // The annotation may result in a cast, which is handled in the type engine.
        typed_expression.return_type = ctx
            .resolve_type_with_self(
                handler,
                typed_expression.return_type,
                ctx.self_type(),
                &expr_span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err)));

        // Literals of type Numeric can now be resolved if typed_expression.return_type is
        // an UnsignedInteger or a Numeric
        if let ty::TyExpressionVariant::Literal(lit) = typed_expression.clone().expression {
            if let Literal::Numeric(_) = lit {
                match type_engine.get(typed_expression.return_type) {
                    TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => {
                        typed_expression = Self::resolve_numeric_literal(
                            handler,
                            ctx,
                            lit,
                            expr_span,
                            typed_expression.return_type,
                        )?
                    }
                    _ => {}
                }
            }
        }

        typed_expression.check_deprecated(engines, handler);

        Ok(typed_expression)
    }

    fn type_check_literal(engines: &Engines, lit: Literal, span: Span) -> ty::TyExpression {
        let type_engine = engines.te();
        let return_type = match &lit {
            Literal::String(_) => TypeInfo::StringSlice,
            Literal::Numeric(_) => TypeInfo::Numeric,
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::U256(_) => TypeInfo::UnsignedInteger(IntegerBits::V256),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::B256(_) => TypeInfo::B256,
        };
        let id = type_engine.insert(engines, return_type);
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(lit),
            return_type: id,
            span,
        }
    }

    pub(crate) fn type_check_variable_expression(
        handler: &Handler,
        ctx: TypeCheckContext,
        name: Ident,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let exp = match ctx
            .namespace
            .resolve_symbol(&Handler::default(), engines, &name)
            .ok()
        {
            Some(ty::TyDecl::VariableDecl(decl)) => {
                let ty::TyVariableDecl {
                    name: decl_name,
                    mutability,
                    return_type,
                    ..
                } = *decl;
                ty::TyExpression {
                    return_type,
                    expression: ty::TyExpressionVariant::VariableExpression {
                        name: decl_name.clone(),
                        span: name.span(),
                        mutability,
                        call_path: Some(
                            CallPath::from(decl_name.clone()).to_fullpath(ctx.namespace),
                        ),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. })) => {
                let const_decl = decl_engine.get_constant(&decl_id);
                let decl_name = const_decl.name().clone();
                ty::TyExpression {
                    return_type: const_decl.return_type,
                    expression: ty::TyExpressionVariant::ConstantExpression {
                        const_decl: Box::new(const_decl),
                        span: name.span(),
                        call_path: Some(CallPath::from(decl_name).to_fullpath(ctx.namespace)),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. })) => {
                let decl = decl_engine.get_abi(&decl_id);
                ty::TyExpression {
                    return_type: decl.create_type_id(engines),
                    expression: ty::TyExpressionVariant::AbiName(AbiName::Known(decl.name.into())),
                    span,
                }
            }
            Some(a) => {
                let err = handler.emit_err(CompileError::NotAVariable {
                    name: name.clone(),
                    what_it_is: a.friendly_type_name(),
                    span,
                });
                ty::TyExpression::error(err, name.span(), engines)
            }
            None => {
                let err = handler.emit_err(CompileError::UnknownVariable {
                    var_name: name.clone(),
                    span,
                });
                ty::TyExpression::error(err, name.span(), engines)
            }
        };
        Ok(exp)
    }

    fn type_check_function_application(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        mut call_path_binding: TypeBinding<CallPath>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        // Grab the fn declaration.
        let (fn_ref, _, _): (DeclRefFunction, _, _) =
            TypeBinding::type_check(&mut call_path_binding, handler, ctx.by_ref())?;

        instantiate_function_application(
            handler,
            ctx,
            fn_ref,
            call_path_binding,
            Some(arguments),
            span,
        )
    }

    fn type_check_lazy_operator(
        handler: &Handler,
        ctx: TypeCheckContext,
        op: LazyOp,
        lhs: Expression,
        rhs: Expression,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let mut ctx = ctx.with_help_text("");
        let engines = ctx.engines();
        let typed_lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs.clone())
            .unwrap_or_else(|err| ty::TyExpression::error(err, lhs.span(), engines));

        let typed_rhs = ty::TyExpression::type_check(handler, ctx.by_ref(), rhs.clone())
            .unwrap_or_else(|err| ty::TyExpression::error(err, rhs.span(), engines));

        let type_annotation = ctx.type_annotation();
        let exp = instantiate_lazy_operator(op, typed_lhs, typed_rhs, type_annotation, span);
        Ok(exp)
    }

    fn type_check_code_block(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        contents: CodeBlock,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let (typed_block, block_return_type) =
            ty::TyCodeBlock::type_check(handler, ctx.by_ref(), contents).unwrap_or_else(|_| {
                (
                    ty::TyCodeBlock { contents: vec![] },
                    type_engine.insert(engines, TypeInfo::Tuple(Vec::new())),
                )
            });

        ctx.unify_with_self(handler, block_return_type, &span);

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                contents: typed_block.contents,
            }),
            return_type: block_return_type,
            span,
        };
        Ok(exp)
    }

    #[allow(clippy::type_complexity)]
    fn type_check_if_expression(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        condition: Expression,
        then: Expression,
        r#else: Option<Expression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let condition = {
            let ctx = ctx
                .by_ref()
                .with_help_text("The condition of an if expression must be a boolean expression.")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Boolean));
            ty::TyExpression::type_check(handler, ctx, condition.clone())
                .unwrap_or_else(|err| ty::TyExpression::error(err, condition.span(), engines))
        };
        let then = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, then.clone())
                .unwrap_or_else(|err| ty::TyExpression::error(err, then.span(), engines))
        };
        let r#else = r#else.map(|expr| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, expr.clone())
                .unwrap_or_else(|err| ty::TyExpression::error(err, expr.span(), engines))
        });
        let exp = instantiate_if_expression(handler, ctx, condition, then, r#else, span)?;
        Ok(exp)
    }

    fn type_check_match_expression(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        value: Expression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // type check the value
        let typed_value = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, value.clone())
                .unwrap_or_else(|err| ty::TyExpression::error(err, value.span(), engines))
        };
        let type_id = typed_value.return_type;

        // check to make sure that the type of the value is something that can be matched upon
        type_engine
            .get(type_id)
            .expect_is_supported_in_match_expressions(handler, &typed_value.span)?;

        // type check the match expression and create a ty::TyMatchExpression object
        let (typed_match_expression, typed_scrutinees) = {
            let ctx = ctx.by_ref().with_help_text("");
            ty::TyMatchExpression::type_check(handler, ctx, typed_value, branches, span.clone())?
        };

        // check to see if the match expression is exhaustive and if all match arms are reachable
        let (witness_report, arms_reachability) = check_match_expression_usefulness(
            handler,
            engines,
            type_id,
            typed_scrutinees.clone(),
            span.clone(),
        )?;

        // if there is an interior catch-all arm
        if let Some(catch_all_arm_position) = interior_catch_all_arm_position(&arms_reachability) {
            // show the warning on the arms below it that it makes them unreachable...
            for reachable_report in arms_reachability[catch_all_arm_position + 1..].iter() {
                handler.emit_warn(CompileWarning {
                    span: reachable_report.scrutinee.span.clone(),
                    warning_content: Warning::MatchExpressionUnreachableArm {
                        match_value: value.span(),
                        match_type: engines.help_out(type_id).to_string(),
                        preceding_arms: Either::Right(
                            arms_reachability[catch_all_arm_position]
                                .scrutinee
                                .span
                                .clone(),
                        ),
                        unreachable_arm: reachable_report.scrutinee.span.clone(),
                        // In this case id doesn't matter if the concrete unreachable arm is
                        // the last arm or a catch-all arm itself.
                        // We want to point out the interior catch-all arm as problematic.
                        // So we simply put these two values both to false.
                        is_last_arm: false,
                        is_catch_all_arm: false,
                    },
                });
            }

            //...but still check the arms above it for reachability
            check_interior_non_catch_all_arms_for_reachability(
                handler,
                engines,
                type_id,
                &value,
                &arms_reachability[..catch_all_arm_position],
            );
        }
        // if there are no interior catch-all arms and there is more then one arm
        else if let Some((last_arm_report, other_arms_reachability)) =
            arms_reachability.split_last()
        {
            // check reachable report for all the arms except the last one
            check_interior_non_catch_all_arms_for_reachability(
                handler,
                engines,
                type_id,
                &value,
                other_arms_reachability,
            );

            // for the last one, give a different warning if it is an unreachable catch-all arm
            if !last_arm_report.reachable {
                handler.emit_warn(CompileWarning {
                    span: last_arm_report.scrutinee.span.clone(),
                    warning_content: Warning::MatchExpressionUnreachableArm {
                        match_value: value.span(),
                        match_type: engines.help_out(type_id).to_string(),
                        preceding_arms: Either::Left(
                            other_arms_reachability
                                .iter()
                                .map(|report| report.scrutinee.span.clone())
                                .collect(),
                        ),
                        unreachable_arm: last_arm_report.scrutinee.span.clone(),
                        is_last_arm: true,
                        is_catch_all_arm: last_arm_report.scrutinee.is_catch_all(),
                    },
                });
            }
        }

        // Emit errors for eventual multiple definitions of variables.
        // These errors can be carried on. The desugared version will treat
        // the duplicates as shadowing, which is fine for the rest of compilation.
        for scrutinee in typed_scrutinees.iter() {
            for duplicate in collect_duplicate_match_pattern_variables(scrutinee) {
                handler.emit_err(CompileError::MultipleDefinitionsOfMatchArmVariable {
                    match_value: value.span(),
                    match_type: engines.help_out(type_id).to_string(),
                    first_definition: duplicate.first_definition.1,
                    first_definition_is_struct_field: duplicate.first_definition.0,
                    duplicate: duplicate.duplicate.1,
                    duplicate_is_struct_field: duplicate.duplicate.0,
                });
            }
        }

        if witness_report.has_witnesses() {
            return Err(
                handler.emit_err(CompileError::MatchExpressionNonExhaustive {
                    missing_patterns: format!("{witness_report}"),
                    span,
                }),
            );
        }

        // desugar the typed match expression to a typed if expression
        let typed_if_exp = typed_match_expression.convert_to_typed_if_expression(handler, ctx)?;

        let match_exp = ty::TyExpression {
            span: typed_if_exp.span.clone(),
            return_type: typed_if_exp.return_type,
            expression: ty::TyExpressionVariant::MatchExp {
                desugared: Box::new(typed_if_exp),
                scrutinees: typed_scrutinees,
            },
        };

        return Ok(match_exp);

        /// Returns the position of the first match arm that is an "interior" arm, meaning:
        ///  - arm is a catch-all arm
        ///  - arm is not the last match arm
        /// or `None` if such arm does not exist.
        /// Note that the arm can be the first arm.
        fn interior_catch_all_arm_position(arms_reachability: &[ReachableReport]) -> Option<usize> {
            arms_reachability
                .split_last()?
                .1
                .iter()
                .position(|report| report.scrutinee.is_catch_all())
        }

        fn check_interior_non_catch_all_arms_for_reachability(
            handler: &Handler,
            engines: &Engines,
            type_id: TypeId,
            match_value: &Expression,
            arms_reachability: &[ReachableReport],
        ) {
            for (index, reachable_report) in arms_reachability.iter().enumerate() {
                if !reachable_report.reachable {
                    handler.emit_warn(CompileWarning {
                        span: reachable_report.scrutinee.span.clone(),
                        warning_content: Warning::MatchExpressionUnreachableArm {
                            match_value: match_value.span(),
                            match_type: engines.help_out(type_id).to_string(),
                            preceding_arms: Either::Left(
                                arms_reachability[..index]
                                    .iter()
                                    .map(|report| report.scrutinee.span.clone())
                                    .collect(),
                            ),
                            unreachable_arm: reachable_report.scrutinee.span.clone(),
                            is_last_arm: false,
                            is_catch_all_arm: false,
                        },
                    });
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        asm: AsmExpression,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Various checks that we can catch early to check that the assembly is valid. For now,
        // this includes two checks:
        // 1. Check that no control flow opcodes are used.
        // 2. Check that initialized registers are not reassigned in the `asm` block.
        check_asm_block_validity(handler, &asm)?;

        let asm_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());
        let return_type = ctx
            .resolve_type_with_self(
                handler,
                type_engine.insert(engines, asm.return_type.clone()),
                ctx.self_type(),
                &asm_span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err)));

        // type check the initializers
        let typed_registers =
            asm.registers
                .clone()
                .into_iter()
                .map(
                    |AsmRegisterDeclaration { name, initializer }| ty::TyAsmRegisterDeclaration {
                        name,
                        initializer: initializer.map(|initializer| {
                            let ctx = ctx.by_ref().with_help_text("").with_type_annotation(
                                type_engine.insert(engines, TypeInfo::Unknown),
                            );

                            ty::TyExpression::type_check(handler, ctx, initializer.clone())
                                .unwrap_or_else(|err| {
                                    ty::TyExpression::error(err, initializer.span(), engines)
                                })
                        }),
                    },
                )
                .collect();

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::AsmExpression {
                whole_block_span: asm.whole_block_span,
                body: asm.body,
                registers: typed_registers,
                returns: asm.returns,
            },
            return_type,
            span,
        };
        Ok(exp)
    }

    fn type_check_subfield_expression(
        handler: &Handler,
        ctx: TypeCheckContext,
        prefix: Expression,
        span: Span,
        field_to_access: Ident,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
        let parent = ty::TyExpression::type_check(handler, ctx, prefix)?;
        let exp = instantiate_struct_field_access(handler, engines, parent, field_to_access, span)?;
        Ok(exp)
    }

    fn type_check_tuple(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fields: Vec<Expression>,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let field_type_opt = match type_engine.get(ctx.type_annotation()) {
            TypeInfo::Tuple(field_type_ids) if field_type_ids.len() == fields.len() => {
                Some(field_type_ids)
            }
            _ => None,
        };
        let mut typed_field_types = Vec::with_capacity(fields.len());
        let mut typed_fields = Vec::with_capacity(fields.len());
        for (i, field) in fields.into_iter().enumerate() {
            let field_type = field_type_opt
                .as_ref()
                .map(|field_type_ids| field_type_ids[i].clone())
                .unwrap_or_else(|| {
                    let initial_type_id = type_engine.insert(engines, TypeInfo::Unknown);
                    TypeArgument {
                        type_id: initial_type_id,
                        initial_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    }
                });
            let field_span = field.span();
            let ctx = ctx
                .by_ref()
                .with_help_text("tuple field type does not match the expected type")
                .with_type_annotation(field_type.type_id);
            let typed_field = ty::TyExpression::type_check(handler, ctx, field)
                .unwrap_or_else(|err| ty::TyExpression::error(err, field_span, engines));
            typed_field_types.push(TypeArgument {
                type_id: typed_field.return_type,
                initial_type_id: field_type.type_id,
                span: typed_field.span.clone(),
                call_path_tree: None,
            });
            typed_fields.push(typed_field);
        }
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::Tuple {
                fields: typed_fields,
            },
            return_type: ctx
                .engines
                .te()
                .insert(engines, TypeInfo::Tuple(typed_field_types)),
            span,
        };
        Ok(exp)
    }

    /// Look up the current global storage state that has been created by storage declarations.
    /// If there isn't any storage, then this is an error. If there is storage, find the corresponding
    /// field that has been specified and return that value.
    fn type_check_storage_access(
        handler: &Handler,
        ctx: TypeCheckContext,
        checkee: Vec<Ident>,
        storage_keyword_span: Span,
        span: &Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        if !ctx.namespace.has_storage_declared() {
            return Err(handler.emit_err(CompileError::NoDeclaredStorage { span: span.clone() }));
        }

        let storage_fields = ctx
            .namespace
            .get_storage_field_descriptors(handler, decl_engine)?;

        // Do all namespace checking here!
        let (storage_access, mut access_type) = ctx.namespace.apply_storage_load(
            handler,
            ctx.engines,
            checkee,
            &storage_fields,
            storage_keyword_span,
        )?;

        // The type of a storage access is `core::storage::StorageKey`. This is
        // the path to it.
        let storage_key_mod_path = vec![
            Ident::new_with_override("core".into(), span.clone()),
            Ident::new_with_override("storage".into(), span.clone()),
        ];
        let storage_key_ident = Ident::new_with_override("StorageKey".into(), span.clone());

        // Search for the struct declaration with the call path above.
        let storage_key_decl_opt = ctx.namespace.root().resolve_symbol(
            handler,
            engines,
            &storage_key_mod_path,
            &storage_key_ident,
        )?;
        let storage_key_struct_decl_ref = storage_key_decl_opt.to_struct_ref(handler, engines)?;
        let mut storage_key_struct_decl = decl_engine.get_struct(&storage_key_struct_decl_ref);

        // Set the type arguments to `StorageKey` to the `access_type`, which is represents the
        // type of the data that the `StorageKey` "points" to.
        let mut type_arguments = vec![TypeArgument {
            initial_type_id: access_type,
            type_id: access_type,
            span: span.clone(),
            call_path_tree: None,
        }];

        // Monomorphize the generic `StorageKey` type given the type argument specified above
        let mut ctx = ctx;
        ctx.monomorphize(
            handler,
            &mut storage_key_struct_decl,
            &mut type_arguments,
            EnforceTypeArguments::Yes,
            span,
        )?;

        // Update `access_type` to be the type of the monomorphized struct after inserting it
        // into the type engine
        let storage_key_struct_decl_ref = ctx.engines().de().insert(storage_key_struct_decl);
        access_type = type_engine.insert(engines, TypeInfo::Struct(storage_key_struct_decl_ref));

        // take any trait items that apply to `StorageKey<T>` and copy them to the
        // monomorphized type
        ctx.insert_trait_implementation_for_type(access_type);

        Ok(ty::TyExpression {
            expression: ty::TyExpressionVariant::StorageAccess(storage_access),
            return_type: access_type,
            span: span.clone(),
        })
    }

    fn type_check_tuple_index(
        handler: &Handler,
        ctx: TypeCheckContext,
        prefix: Expression,
        index: usize,
        index_span: Span,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
        let parent = ty::TyExpression::type_check(handler, ctx, prefix)?;
        let exp =
            instantiate_tuple_index_access(handler, engines, parent, index, index_span, span)?;
        Ok(exp)
    }

    fn type_check_ambiguous_path(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        TypeBinding {
            inner:
                CallPath {
                    prefixes,
                    suffix: AmbiguousSuffix { before, suffix },
                    is_absolute,
                },
            type_arguments,
            span: path_span,
        }: TypeBinding<CallPath<AmbiguousSuffix>>,
        span: Span,
        args: Vec<Expression>,
        qualified_path_root: Option<QualifiedPathRootTypes>,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = ctx.engines;
        let decl_engine = engines.de();

        if let Some(QualifiedPathRootTypes { ty, as_trait, .. }) = qualified_path_root {
            if !prefixes.is_empty() || before.is_some() {
                return Err(handler.emit_err(
                    ConvertParseTreeError::UnexpectedCallPathPrefixAfterQualifiedRoot {
                        span: path_span,
                    }
                    .into(),
                ));
            }

            let method_name_binding = TypeBinding {
                inner: MethodName::FromQualifiedPathRoot {
                    ty,
                    as_trait,
                    method_name: suffix,
                },
                type_arguments,
                span: path_span,
            };

            return type_check_method_application(
                handler,
                ctx.by_ref(),
                method_name_binding,
                Vec::new(),
                args,
                span,
            );
        }

        // is it a singleton?
        let before = if let Some(b) = before {
            b
        } else {
            // if it's a singleton it's either an enum variant or a function
            let call_path_binding = TypeBinding {
                inner: CallPath {
                    prefixes,
                    suffix,
                    is_absolute,
                },
                type_arguments,
                span: path_span,
            };
            if matches!(
                ctx.namespace.resolve_call_path(
                    &Handler::default(),
                    engines,
                    &call_path_binding.inner
                ),
                Ok(ty::TyDecl::EnumVariantDecl { .. })
            ) {
                return Self::type_check_delineated_path(
                    handler,
                    ctx,
                    call_path_binding,
                    span,
                    Some(args),
                );
            } else {
                return Self::type_check_function_application(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    args,
                    span,
                );
            }
        };

        // Is `path = prefix ++ before` a module?
        let mut path = Vec::with_capacity(prefixes.len() + 1);
        path.extend(prefixes.iter().cloned());
        path.push(before.inner.clone());
        let not_module = {
            let h = Handler::default();
            ctx.namespace.check_submodule(&h, &path).is_err()
        };

        // Not a module? Not a `Enum::Variant` either?
        // Type check as an associated function call instead.
        let is_associated_call = not_module && {
            let probe_call_path = CallPath {
                prefixes: prefixes.clone(),
                suffix: before.inner.clone(),
                is_absolute,
            };
            ctx.namespace
                .resolve_call_path(&Handler::default(), engines, &probe_call_path)
                .and_then(|decl| decl.to_enum_ref(&Handler::default(), ctx.engines()))
                .map(|decl_ref| decl_engine.get_enum(&decl_ref))
                .and_then(|decl| {
                    decl.expect_variant_from_name(&Handler::default(), &suffix)
                        .map(drop)
                })
                .is_err()
        };

        if is_associated_call {
            let before_span = before.span();
            let type_name = before.inner;
            let type_info = type_name_to_type_info_opt(&type_name).unwrap_or(TypeInfo::Custom {
                call_path: type_name.clone().into(),
                type_arguments: None,
                root_type_id: None,
            });

            let method_name_binding = TypeBinding {
                inner: MethodName::FromType {
                    call_path_binding: TypeBinding {
                        span: before_span,
                        type_arguments: before.type_arguments,
                        inner: CallPath {
                            prefixes,
                            suffix: (type_info, type_name),
                            is_absolute,
                        },
                    },
                    method_name: suffix,
                },
                type_arguments,
                span: path_span,
            };
            type_check_method_application(
                handler,
                ctx.by_ref(),
                method_name_binding,
                Vec::new(),
                args,
                span,
            )
        } else {
            let mut type_arguments = type_arguments;
            if let TypeArgs::Regular(vec) = before.type_arguments {
                if !vec.is_empty() {
                    if !type_arguments.to_vec().is_empty() {
                        return Err(handler.emit_err(
                            ConvertParseTreeError::MultipleGenericsNotSupported { span }.into(),
                        ));
                    }
                    type_arguments = TypeArgs::Prefix(vec)
                }
            }

            let call_path_binding = TypeBinding {
                inner: CallPath {
                    prefixes: path,
                    suffix,
                    is_absolute,
                },
                type_arguments,
                span: path_span,
            };
            Self::type_check_delineated_path(handler, ctx, call_path_binding, span, Some(args))
        }
    }

    fn type_check_delineated_path(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        unknown_call_path_binding: TypeBinding<CallPath>,
        span: Span,
        args: Option<Vec<Expression>>,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        // The first step is to determine if the call path refers to a module,
        // enum, function or constant.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        // Check if this could be a module
        let module_probe_handler = Handler::default();
        let is_module = {
            let call_path_binding = unknown_call_path_binding.clone();
            ctx.namespace
                .check_submodule(
                    &module_probe_handler,
                    &[
                        call_path_binding.inner.prefixes,
                        vec![call_path_binding.inner.suffix],
                    ]
                    .concat(),
                )
                .ok()
                .is_some()
        };

        // Check if this could be a function
        let function_probe_handler = Handler::default();
        let maybe_function: Option<(DeclRefFunction, _)> = {
            let mut call_path_binding = unknown_call_path_binding.clone();
            TypeBinding::type_check(
                &mut call_path_binding,
                &function_probe_handler,
                ctx.by_ref(),
            )
            .ok()
            .map(|(fn_ref, _, _)| (fn_ref, call_path_binding))
        };

        // Check if this could be an enum
        let enum_probe_handler = Handler::default();
        let maybe_enum: Option<(DeclRefEnum, _, _, _)> = {
            let call_path_binding = unknown_call_path_binding.clone();
            let variant_name = call_path_binding.inner.suffix.clone();
            let enum_call_path = call_path_binding.inner.rshift();

            let mut call_path_binding = TypeBinding {
                inner: enum_call_path,
                type_arguments: call_path_binding.type_arguments,
                span: call_path_binding.span,
            };
            TypeBinding::type_check(&mut call_path_binding, &enum_probe_handler, ctx.by_ref())
                .ok()
                .map(|(enum_ref, _, ty_decl)| {
                    (
                        enum_ref,
                        variant_name,
                        call_path_binding,
                        ty_decl.expect("type_check for TyEnumDecl should always return TyDecl"),
                    )
                })
        };

        // Check if this could be a constant
        let const_probe_handler = Handler::default();
        let maybe_const =
            { Self::probe_const_decl(&unknown_call_path_binding, &mut ctx, &const_probe_handler) };

        // compare the results of the checks
        let exp = match (is_module, maybe_function, maybe_enum, maybe_const) {
            (
                false,
                None,
                Some((enum_ref, variant_name, call_path_binding, call_path_decl)),
                None,
            ) => {
                handler.append(enum_probe_handler);
                instantiate_enum(
                    handler,
                    ctx,
                    enum_ref,
                    variant_name,
                    args,
                    call_path_binding,
                    call_path_decl,
                )?
            }
            (false, Some((fn_ref, call_path_binding)), None, None) => {
                handler.append(function_probe_handler);
                // In case `foo::bar::<TyArgs>::baz(...)` throw an error.
                if let TypeArgs::Prefix(_) = call_path_binding.type_arguments {
                    handler.emit_err(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: call_path_binding.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                instantiate_function_application(
                    handler,
                    ctx,
                    fn_ref,
                    call_path_binding,
                    args,
                    span,
                )?
            }
            (true, None, None, None) => {
                handler.append(module_probe_handler);
                return Err(handler.emit_err(CompileError::Unimplemented(
                    "this case is not yet implemented",
                    span,
                )));
            }
            (false, None, None, Some((const_ref, call_path_binding))) => {
                handler.append(const_probe_handler);
                if !call_path_binding.type_arguments.to_vec().is_empty() {
                    // In case `foo::bar::CONST::<TyArgs>` throw an error.
                    // In case `foo::bar::<TyArgs>::CONST` throw an error.
                    handler.emit_err(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: unknown_call_path_binding.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                instantiate_constant_expression(ctx, const_ref, call_path_binding)
            }
            (false, None, None, None) => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: unknown_call_path_binding.inner.suffix.clone(),
                    span: unknown_call_path_binding.inner.suffix.span(),
                }));
            }
            _ => {
                return Err(handler.emit_err(CompileError::AmbiguousPath { span }));
            }
        };
        Ok(exp)
    }

    fn probe_const_decl(
        unknown_call_path_binding: &TypeBinding<CallPath>,
        ctx: &mut TypeCheckContext,
        const_probe_handler: &Handler,
    ) -> Option<(DeclRefConstant, TypeBinding<CallPath>)> {
        let mut call_path_binding = unknown_call_path_binding.clone();

        let type_info_opt = call_path_binding
            .clone()
            .inner
            .prefixes
            .last()
            .map(|type_name| {
                type_name_to_type_info_opt(type_name).unwrap_or(TypeInfo::Custom {
                    call_path: type_name.clone().into(),
                    type_arguments: None,
                    root_type_id: None,
                })
            });

        if let Some(TypeInfo::SelfType) = type_info_opt {
            call_path_binding.strip_prefixes();
        }

        let const_opt: Option<(DeclRefConstant, _)> =
            TypeBinding::type_check(&mut call_path_binding, &Handler::default(), ctx.by_ref())
                .ok()
                .map(|(const_ref, _, _)| (const_ref, call_path_binding.clone()));
        if const_opt.is_some() {
            return const_opt;
        }

        // If we didn't find a constant, check for the constant inside the impl.
        let suffix = call_path_binding.inner.suffix.clone();
        let const_call_path = call_path_binding.inner.rshift();

        let mut const_call_path_binding = TypeBinding {
            inner: const_call_path,
            type_arguments: call_path_binding.type_arguments.clone(),
            span: call_path_binding.span.clone(),
        };

        let (_, struct_type_id, _): (DeclRefStruct, _, _) = match TypeBinding::type_check(
            &mut const_call_path_binding,
            const_probe_handler,
            ctx.by_ref(),
        ) {
            Ok(val) => val,
            Err(_) => return None,
        };

        let const_decl_ref = match ctx.find_constant_for_type(
            const_probe_handler,
            struct_type_id.unwrap(),
            &suffix,
            ctx.self_type(),
        ) {
            Ok(Some(val)) => val,
            Ok(None) | Err(_) => return None,
        };

        Some((const_decl_ref, call_path_binding.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        abi_name: CallPath,
        address: Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // TODO use lib-std's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span();
        let address_expr = {
            let ctx = ctx
                .by_ref()
                .with_help_text("An address that is being ABI cast must be of type b256")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::B256));
            ty::TyExpression::type_check(handler, ctx, address)
                .unwrap_or_else(|err| ty::TyExpression::error(err, err_span, engines))
        };

        // look up the call path and get the declaration it references
        let abi = ctx
            .namespace
            .resolve_call_path(handler, engines, &abi_name)?;
        let abi_ref = match abi {
            ty::TyDecl::AbiDecl(ty::AbiDecl {
                name,
                decl_id,
                decl_span,
            }) => DeclRef::new(name, decl_id, decl_span),
            ty::TyDecl::VariableDecl(ref decl) => {
                let ty::TyVariableDecl { body: expr, .. } = &**decl;
                let ret_ty = type_engine.get(expr.return_type);
                let abi_name = match ret_ty {
                    TypeInfo::ContractCaller { abi_name, .. } => abi_name,
                    _ => {
                        return Err(handler.emit_err(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: abi.friendly_type_name(),
                        }));
                    }
                };
                match abi_name {
                    // look up the call path and get the declaration it references
                    AbiName::Known(abi_name) => {
                        let unknown_decl = ctx
                            .namespace
                            .resolve_call_path(handler, engines, &abi_name)?;
                        unknown_decl.to_abi_ref(handler)?
                    }
                    AbiName::Deferred => {
                        return Ok(ty::TyExpression {
                            return_type: type_engine.insert(
                                engines,
                                TypeInfo::ContractCaller {
                                    abi_name: AbiName::Deferred,
                                    address: None,
                                },
                            ),
                            expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                            span,
                        })
                    }
                }
            }
            a => {
                return Err(handler.emit_err(CompileError::NotAnAbi {
                    span: abi_name.span(),
                    actually_is: a.friendly_type_name(),
                }));
            }
        };
        let ty::TyAbiDecl {
            interface_surface,
            items,
            supertraits,
            span,
            ..
        } = decl_engine.get_abi(abi_ref.id());

        let return_type = type_engine.insert(
            engines,
            TypeInfo::ContractCaller {
                abi_name: AbiName::Known(abi_name.clone()),
                address: Some(Box::new(address_expr.clone())),
            },
        );

        // Retrieve the interface surface for this abi.
        let mut abi_items = vec![];

        for item in interface_surface.into_iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let method = decl_engine.get_trait_fn(&decl_ref);
                    abi_items.push(TyImplItem::Fn(
                        decl_engine
                            .insert(method.to_dummy_func(AbiMode::ImplAbiFn(
                                abi_name.suffix.clone(),
                                Some(*abi_ref.id()),
                            )))
                            .with_parent(decl_engine, (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(&decl_ref);
                    abi_items.push(TyImplItem::Constant(decl_engine.insert(const_decl)));
                }
                ty::TyTraitInterfaceItem::Type(decl_ref) => {
                    let type_decl = decl_engine.get_type(&decl_ref);
                    abi_items.push(TyImplItem::Type(decl_engine.insert(type_decl)));
                }
            }
        }

        // Retrieve the items for this abi.
        abi_items.append(&mut items.into_iter().collect::<Vec<_>>());

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this abi cast.
        insert_supertraits_into_namespace(
            handler,
            ctx.by_ref(),
            return_type,
            &supertraits,
            &SupertraitOf::Abi(span.clone()),
        )?;

        // Insert the abi methods into the namespace.
        ctx.insert_trait_implementation(
            handler,
            abi_name.clone(),
            vec![],
            return_type,
            &abi_items,
            &span,
            Some(span.clone()),
            IsImplSelf::No,
            IsExtendingExistingImpl::No,
        )?;

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::AbiCast {
                abi_name,
                address: Box::new(address_expr),
                span: span.clone(),
            },
            return_type,
            span,
        };
        Ok(exp)
    }

    fn type_check_array(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        contents: Vec<Expression>,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        if contents.is_empty() {
            let unknown_type = type_engine.insert(engines, TypeInfo::Unknown);
            return Ok(ty::TyExpression {
                expression: ty::TyExpressionVariant::Array {
                    elem_type: unknown_type,
                    contents: Vec::new(),
                },
                return_type: type_engine.insert(
                    engines,
                    TypeInfo::Array(
                        TypeArgument {
                            type_id: unknown_type,
                            span: Span::dummy(),
                            call_path_tree: None,
                            initial_type_id: unknown_type,
                        },
                        Length::new(0, Span::dummy()),
                    ),
                ),
                span,
            });
        };

        let typed_contents: Vec<ty::TyExpression> = contents
            .into_iter()
            .map(|expr| {
                let span = expr.span();
                let ctx = ctx
                    .by_ref()
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
                Self::type_check(handler, ctx, expr)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, span, engines))
            })
            .collect();

        let elem_type = typed_contents[0].return_type;
        for typed_elem in &typed_contents[1..] {
            let h = Handler::default();
            ctx.by_ref()
                .with_type_annotation(elem_type)
                .unify_with_self(&h, typed_elem.return_type, &typed_elem.span);
            let (new_errors, new_warnings) = h.consume();
            let no_warnings = new_warnings.is_empty();
            let no_errors = new_errors.is_empty();
            for warn in new_warnings {
                handler.emit_warn(warn);
            }
            for err in new_errors {
                handler.emit_err(err);
            }
            // In both cases, if there are warnings or errors then break here, since we don't
            // need to spam type errors for every element once we have one.
            if !no_warnings && !no_errors {
                break;
            }
        }

        let array_count = typed_contents.len();
        Ok(ty::TyExpression {
            expression: ty::TyExpressionVariant::Array {
                elem_type,
                contents: typed_contents,
            },
            return_type: type_engine.insert(
                engines,
                TypeInfo::Array(
                    TypeArgument {
                        type_id: elem_type,
                        span: Span::dummy(),
                        call_path_tree: None,
                        initial_type_id: elem_type,
                    },
                    Length::new(array_count, Span::dummy()),
                ),
            ), // Maybe?
            span,
        })
    }

    fn type_check_array_index(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        prefix: Expression,
        index: Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let prefix_te = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, prefix.clone())?
        };

        fn get_array_type(ty: TypeId, type_engine: &TypeEngine) -> Option<TypeInfo> {
            match &type_engine.get(ty) {
                TypeInfo::Array(..) => Some(type_engine.get(ty)),
                TypeInfo::Alias { ty, .. } => get_array_type(ty.type_id, type_engine),
                _ => None,
            }
        }

        // If the return type is a static array then create a `ty::TyExpressionVariant::ArrayIndex`.
        if let Some(TypeInfo::Array(elem_type, _)) =
            get_array_type(prefix_te.return_type, type_engine)
        {
            let type_info_u64 = TypeInfo::UnsignedInteger(IntegerBits::SixtyFour);
            let ctx = ctx
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, type_info_u64));
            let index_te = ty::TyExpression::type_check(handler, ctx, index)?;

            Ok(ty::TyExpression {
                expression: ty::TyExpressionVariant::ArrayIndex {
                    prefix: Box::new(prefix_te),
                    index: Box::new(index_te),
                },
                return_type: elem_type.type_id,
                span,
            })
        } else {
            // Otherwise convert into a method call 'index(self, index)' via the std::ops::Index trait.
            let method_name = TypeBinding {
                inner: MethodName::FromTrait {
                    call_path: CallPath {
                        prefixes: vec![
                            Ident::new_with_override("core".into(), span.clone()),
                            Ident::new_with_override("ops".into(), span.clone()),
                        ],
                        suffix: Ident::new_with_override("index".into(), span.clone()),
                        is_absolute: true,
                    },
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: span.clone(),
            };
            type_check_method_application(
                handler,
                ctx,
                method_name,
                vec![],
                vec![prefix, index],
                span,
            )
        }
    }

    fn type_check_intrinsic_function(
        handler: &Handler,
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let (intrinsic_function, return_type) = ty::TyIntrinsicFunctionKind::type_check(
            handler,
            ctx,
            kind_binding,
            arguments,
            span.clone(),
        )?;
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::IntrinsicFunction(intrinsic_function),
            return_type,
            span,
        };
        Ok(exp)
    }

    fn type_check_while_loop(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        condition: Expression,
        body: CodeBlock,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let typed_condition = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Boolean))
                .with_help_text("A while loop's loop condition must be a boolean expression.");
            ty::TyExpression::type_check(handler, ctx, condition)?
        };

        let unit_ty = type_engine.insert(engines, TypeInfo::Tuple(Vec::new()));
        let ctx = ctx.with_type_annotation(unit_ty).with_help_text(
            "A while loop's loop body cannot implicitly return a value. Try \
                 assigning it to a mutable variable declared outside of the loop \
                 instead.",
        );
        let (typed_body, _block_implicit_return) = ty::TyCodeBlock::type_check(handler, ctx, body)?;
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::WhileLoop {
                condition: Box::new(typed_condition),
                body: typed_body,
            },
            return_type: unit_ty,
            span,
        };
        Ok(exp)
    }

    fn type_check_reassignment(
        handler: &Handler,
        ctx: TypeCheckContext,
        lhs: ReassignmentTarget,
        rhs: Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let mut ctx = ctx
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
            .with_help_text("");
        // ensure that the lhs is a supported expression kind
        match lhs {
            ReassignmentTarget::VariableExpression(var) => {
                let mut expr = var;
                let mut names_vec = Vec::new();
                let (base_name, final_return_type) = loop {
                    match expr.kind {
                        ExpressionKind::Variable(name) => {
                            // check that the reassigned name exists
                            let unknown_decl =
                                ctx.namespace.resolve_symbol(handler, engines, &name)?;
                            let variable_decl = unknown_decl.expect_variable(handler).cloned()?;
                            if !variable_decl.mutability.is_mutable() {
                                return Err(handler.emit_err(
                                    CompileError::AssignmentToNonMutable { name, span },
                                ));
                            }
                            break (name, variable_decl.body.return_type);
                        }
                        ExpressionKind::Subfield(SubfieldExpression {
                            prefix,
                            field_to_access,
                            ..
                        }) => {
                            names_vec.push(ty::ProjectionKind::StructField {
                                name: field_to_access,
                            });
                            expr = prefix;
                        }
                        ExpressionKind::TupleIndex(TupleIndexExpression {
                            prefix,
                            index,
                            index_span,
                            ..
                        }) => {
                            names_vec.push(ty::ProjectionKind::TupleField { index, index_span });
                            expr = prefix;
                        }
                        ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                            let ctx = ctx.by_ref().with_help_text("");
                            let typed_index =
                                ty::TyExpression::type_check(handler, ctx, index.as_ref().clone())
                                    .unwrap_or_else(|err| {
                                        ty::TyExpression::error(err, span.clone(), engines)
                                    });
                            names_vec.push(ty::ProjectionKind::ArrayIndex {
                                index: Box::new(typed_index),
                                index_span: index.span(),
                            });
                            expr = prefix;
                        }
                        _ => {
                            return Err(
                                handler.emit_err(CompileError::InvalidExpressionOnLhs { span })
                            );
                        }
                    }
                };
                let names_vec = names_vec.into_iter().rev().collect::<Vec<_>>();
                let (ty_of_field, _ty_of_parent) = ctx.namespace.find_subfield_type(
                    handler,
                    ctx.engines(),
                    &base_name,
                    &names_vec,
                )?;
                // type check the reassignment
                let ctx = ctx.with_type_annotation(ty_of_field).with_help_text("");
                let rhs_span = rhs.span();
                let rhs = ty::TyExpression::type_check(handler, ctx, rhs)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, rhs_span, engines));

                Ok(ty::TyExpression {
                    expression: ty::TyExpressionVariant::Reassignment(Box::new(
                        ty::TyReassignment {
                            lhs_base_name: base_name,
                            lhs_type: final_return_type,
                            lhs_indices: names_vec,
                            rhs,
                        },
                    )),
                    return_type: type_engine.insert(engines, TypeInfo::Tuple(Vec::new())),
                    span,
                })
            }
        }
    }

    fn resolve_numeric_literal(
        handler: &Handler,
        ctx: TypeCheckContext,
        lit: Literal,
        span: Span,
        new_type: TypeId,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Parse and resolve a Numeric(span) based on new_type.
        let (val, new_integer_type) = match lit {
            Literal::Numeric(num) => match type_engine.get(new_type) {
                TypeInfo::UnsignedInteger(n) => match n {
                    IntegerBits::Eight => (
                        num.to_string().parse().map(Literal::U8).map_err(|e| {
                            Literal::handle_parse_int_error(
                                engines,
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Eight),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::Sixteen => (
                        num.to_string().parse().map(Literal::U16).map_err(|e| {
                            Literal::handle_parse_int_error(
                                engines,
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::ThirtyTwo => (
                        num.to_string().parse().map(Literal::U32).map_err(|e| {
                            Literal::handle_parse_int_error(
                                engines,
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::SixtyFour => (
                        num.to_string().parse().map(Literal::U64).map_err(|e| {
                            Literal::handle_parse_int_error(
                                engines,
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    // Numerics are limited to u64 for now
                    IntegerBits::V256 => (Ok(Literal::U256(U256::from(num))), new_type),
                },
                TypeInfo::Numeric => (
                    num.to_string().parse().map(Literal::Numeric).map_err(|e| {
                        Literal::handle_parse_int_error(engines, e, TypeInfo::Numeric, span.clone())
                    }),
                    type_engine.insert(engines, TypeInfo::Numeric),
                ),
                _ => unreachable!("Unexpected type for integer literals"),
            },
            _ => unreachable!("Unexpected non-integer literals"),
        };

        match val {
            Ok(v) => {
                let exp = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Literal(v),
                    return_type: new_integer_type,
                    span,
                };
                Ok(exp)
            }
            Err(e) => {
                let err = handler.emit_err(e);
                let exp = ty::TyExpression::error(err, span, engines);
                Ok(exp)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engines;
    use sway_error::type_error::TypeError;

    fn do_type_check(
        handler: &Handler,
        engines: &Engines,
        expr: Expression,
        type_annotation: TypeId,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let mut namespace = Namespace::init_root(namespace::Module::default());
        let ctx = TypeCheckContext::from_root(&mut namespace, engines)
            .with_type_annotation(type_annotation);
        ty::TyExpression::type_check(handler, ctx, expr)
    }

    fn do_type_check_for_boolx2(
        handler: &Handler,
        expr: Expression,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = Engines::default();
        do_type_check(
            handler,
            &engines,
            expr,
            engines.te().insert(
                &engines,
                TypeInfo::Array(
                    TypeArgument {
                        type_id: engines.te().insert(&engines, TypeInfo::Boolean),
                        span: Span::dummy(),
                        call_path_tree: None,
                        initial_type_id: engines.te().insert(&engines, TypeInfo::Boolean),
                    },
                    Length::new(2, Span::dummy()),
                ),
            ),
        )
    }

    #[test]
    fn test_array_type_check_non_homogeneous_0() {
        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression {
                contents: vec![
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(true)),
                        span: Span::dummy(),
                    },
                    Expression {
                        kind: ExpressionKind::Literal(Literal::U64(0)),
                        span: Span::dummy(),
                    },
                ],
                length_span: None,
            }),
            span: Span::dummy(),
        };

        let handler = Handler::default();
        let _comp_res = do_type_check_for_boolx2(&handler, expr);
        let (errors, _warnings) = handler.consume();
        assert!(errors.len() == 1);
        assert!(matches!(&errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected == "bool"
                                && received == "u64"));
    }

    #[test]
    fn test_array_type_check_non_homogeneous_1() {
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression {
                contents: vec![
                    Expression {
                        kind: ExpressionKind::Literal(Literal::U64(0)),
                        span: Span::dummy(),
                    },
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(true)),
                        span: Span::dummy(),
                    },
                ],
                length_span: None,
            }),
            span: Span::dummy(),
        };

        let handler = Handler::default();
        let _comp_res = do_type_check_for_boolx2(&handler, expr);
        let (errors, _warnings) = handler.consume();
        assert!(errors.len() == 2);
        assert!(matches!(&errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected == "u64"
                                && received == "bool"));
        assert!(matches!(&errors[1],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected == "[bool; 2]"
                                && received == "[u64; 2]"));
    }

    #[test]
    fn test_array_type_check_bad_count() {
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression {
                contents: vec![
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(true)),
                        span: Span::dummy(),
                    },
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(true)),
                        span: Span::dummy(),
                    },
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Boolean(true)),
                        span: Span::dummy(),
                    },
                ],
                length_span: None,
            }),
            span: Span::dummy(),
        };

        let handler = Handler::default();
        let _comp_res = do_type_check_for_boolx2(&handler, expr);
        let (errors, _warnings) = handler.consume();
        assert!(errors.len() == 1);
        assert!(matches!(&errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected == "[bool; 2]"
                                && received == "[bool; 3]"));
    }

    #[test]
    fn test_array_type_check_empty() {
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression {
                contents: Vec::new(),
                length_span: None,
            }),
            span: Span::dummy(),
        };

        let engines = Engines::default();
        let handler = Handler::default();
        let comp_res = do_type_check(
            &handler,
            &engines,
            expr,
            engines.te().insert(
                &engines,
                TypeInfo::Array(
                    TypeArgument {
                        type_id: engines.te().insert(&engines, TypeInfo::Boolean),
                        span: Span::dummy(),
                        call_path_tree: None,
                        initial_type_id: engines.te().insert(&engines, TypeInfo::Boolean),
                    },
                    Length::new(0, Span::dummy()),
                ),
            ),
        );
        let (errors, warnings) = handler.consume();
        assert!(comp_res.is_ok());
        assert!(warnings.is_empty() && errors.is_empty());
    }
}

fn check_asm_block_validity(handler: &Handler, asm: &AsmExpression) -> Result<(), ErrorEmitted> {
    // Collect all asm block instructions in the form of `VirtualOp`s
    let mut opcodes = vec![];
    for op in &asm.body {
        let registers = op
            .op_args
            .iter()
            .map(|reg_name| VirtualRegister::Virtual(reg_name.to_string()))
            .collect::<Vec<VirtualRegister>>();

        opcodes.push((
            crate::asm_lang::Op::parse_opcode(
                handler,
                &op.op_name,
                &registers,
                &op.immediate,
                op.span.clone(),
            )?,
            op.op_name.clone(),
            op.span.clone(),
        ));
    }

    // Check #1: Disallow control flow instructions
    //
    for err in opcodes
        .iter()
        .filter(|op| {
            matches!(
                op.0,
                VirtualOp::JMP(_)
                    | VirtualOp::JI(_)
                    | VirtualOp::JNE(..)
                    | VirtualOp::JNEI(..)
                    | VirtualOp::JNZI(..)
                    | VirtualOp::RET(_)
                    | VirtualOp::RETD(..)
                    | VirtualOp::RVRT(..)
            )
        })
        .map(|op| CompileError::DisallowedControlFlowInstruction {
            name: op.1.to_string(),
            span: op.2.clone(),
        })
    {
        handler.emit_err(err);
    }

    // Check #2: Disallow initialized registers from being reassigned in the asm block
    //
    // 1. Collect all registers that have initializers in the list of arguments
    let initialized_registers = asm
        .registers
        .iter()
        .filter(|reg| reg.initializer.is_some())
        .map(|reg| VirtualRegister::Virtual(reg.name.to_string()))
        .collect::<FxHashSet<_>>();

    // 2. From the list of `VirtualOp`s, figure out what registers are assigned
    let assigned_registers: FxHashSet<VirtualRegister> =
        opcodes.iter().fold(FxHashSet::default(), |mut acc, op| {
            for u in op.0.def_registers() {
                acc.insert(u.clone());
            }
            acc
        });

    // 3. Intersect the list of assigned registers with the list of initialized registers
    let initialized_and_assigned_registers = assigned_registers
        .intersection(&initialized_registers)
        .collect::<FxHashSet<_>>();

    // 4. Form all the compile errors given the violating registers above. Obtain span information
    //    from the original `asm.registers` vector.
    for err in asm
        .registers
        .iter()
        .filter(|reg| {
            initialized_and_assigned_registers
                .contains(&VirtualRegister::Virtual(reg.name.to_string()))
        })
        .map(|reg| CompileError::InitializedRegisterReassignment {
            name: reg.name.to_string(),
            span: reg.name.span(),
        })
    {
        handler.emit_err(err);
    }

    Ok(())
}
