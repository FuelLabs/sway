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
    error::*,
    language::{
        parsed::*,
        ty::{self, TyImplItem},
        *,
    },
    semantic_analysis::*,
    transform::to_parsed_lang::type_name_to_type_info_opt,
    type_system::*,
    Engines,
};

use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{integer_bits::IntegerBits, Ident, Named, Span, Spanned};

use rustc_hash::FxHashSet;

use std::collections::{HashMap, VecDeque};

#[allow(clippy::too_many_arguments)]
impl ty::TyExpression {
    pub(crate) fn core_ops_eq(
        ctx: TypeCheckContext,
        arguments: Vec<ty::TyExpression>,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

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
        let (decl_ref, _) = check!(
            resolve_method_name(ctx, &mut method_name_binding, arguments.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let method = decl_engine.get_function(&decl_ref);
        // check that the number of parameters and the number of the arguments is the same
        check!(
            check_function_arguments_arity(arguments.len(), &method, &call_path, false),
            return err(warnings, errors),
            warnings,
            errors
        );
        let return_type = method.return_type;
        let args_and_names = method
            .parameters
            .into_iter()
            .zip(arguments.into_iter())
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
            },
            return_type: return_type.type_id,
            span,
        };
        ok(exp, warnings, errors)
    }

    pub(crate) fn type_check(mut ctx: TypeCheckContext, expr: Expression) -> CompileResult<Self> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        let expr_span = expr.span();
        let span = expr_span.clone();
        let res = match expr.kind {
            // We've already emitted an error for the `::Error` case.
            ExpressionKind::Error(_) => ok(ty::TyExpression::error(span, engines), vec![], vec![]),
            ExpressionKind::Literal(lit) => Self::type_check_literal(engines, lit, span),
            ExpressionKind::AmbiguousVariableExpression(name) => {
                let call_path = CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                };
                if matches!(
                    ctx.namespace.resolve_call_path(&call_path).value,
                    Some(ty::TyDecl::EnumVariantDecl { .. })
                ) {
                    Self::type_check_delineated_path(
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
                    Self::type_check_variable_expression(ctx.by_ref(), name, span)
                }
            }
            ExpressionKind::Variable(name) => {
                Self::type_check_variable_expression(ctx.by_ref(), name, span)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    arguments,
                } = *function_application_expression;
                Self::type_check_function_application(
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
                Self::type_check_lazy_operator(ctx, op, *lhs, *rhs, span)
            }
            ExpressionKind::CodeBlock(contents) => {
                Self::type_check_code_block(ctx.by_ref(), contents, span)
            }
            // TODO if _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
            }) => Self::type_check_if_expression(
                ctx.by_ref().with_help_text(""),
                *condition,
                *then,
                r#else.map(|e| *e),
                span,
            ),
            ExpressionKind::Match(MatchExpression { value, branches }) => {
                Self::type_check_match_expression(
                    ctx.by_ref().with_help_text(""),
                    *value,
                    branches,
                    span,
                )
            }
            ExpressionKind::Asm(asm) => Self::type_check_asm_expression(ctx.by_ref(), *asm, span),
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    fields,
                } = *struct_expression;
                struct_instantiation(ctx.by_ref(), call_path_binding, fields, span)
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
            }) => {
                Self::type_check_subfield_expression(ctx.by_ref(), *prefix, span, field_to_access)
            }
            ExpressionKind::MethodApplication(method_application_expression) => {
                let MethodApplicationExpression {
                    method_name_binding,
                    contract_call_params,
                    arguments,
                } = *method_application_expression;
                type_check_method_application(
                    ctx.by_ref(),
                    method_name_binding,
                    contract_call_params,
                    arguments,
                    span,
                )
            }
            ExpressionKind::Tuple(fields) => Self::type_check_tuple(ctx.by_ref(), fields, span),
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix,
                index,
                index_span,
            }) => Self::type_check_tuple_index(ctx.by_ref(), *prefix, index, index_span, span),
            ExpressionKind::AmbiguousPathExpression(e) => {
                let AmbiguousPathExpression {
                    call_path_binding,
                    args,
                    qualified_path_root,
                } = *e;
                Self::type_check_ambiguous_path(
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
                Self::type_check_delineated_path(ctx.by_ref(), call_path_binding, span, args)
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                let AbiCastExpression { abi_name, address } = *abi_cast_expression;
                Self::type_check_abi_cast(ctx.by_ref(), abi_name, *address, span)
            }
            ExpressionKind::Array(array_expression) => {
                Self::type_check_array(ctx.by_ref(), array_expression.contents, span)
            }
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_array_index(ctx, *prefix, *index, span)
            }
            ExpressionKind::StorageAccess(StorageAccessExpression {
                field_names,
                storage_keyword_span,
            }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_storage_access(ctx, field_names, storage_keyword_span, &span)
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                kind_binding,
                arguments,
                ..
            }) => Self::type_check_intrinsic_function(ctx.by_ref(), kind_binding, arguments, span),
            ExpressionKind::WhileLoop(WhileLoopExpression { condition, body }) => {
                Self::type_check_while_loop(ctx.by_ref(), *condition, body, span)
            }
            ExpressionKind::Break => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Break,
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    span,
                };
                ok(expr, vec![], vec![])
            }
            ExpressionKind::Continue => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Continue,
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    span,
                };
                ok(expr, vec![], vec![])
            }
            ExpressionKind::Reassignment(ReassignmentExpression { lhs, rhs }) => {
                Self::type_check_reassignment(ctx.by_ref(), lhs, *rhs, span)
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
                let mut warnings = vec![];
                let mut errors = vec![];
                let expr_span = expr.span();
                let expr = check!(
                    ty::TyExpression::type_check(ctx, *expr),
                    ty::TyExpression::error(expr_span, engines),
                    warnings,
                    errors,
                );
                let typed_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Return(Box::new(expr)),
                    return_type: type_engine.insert(engines, TypeInfo::Unknown),
                    // FIXME: This should be Yes?
                    span,
                };
                ok(typed_expr, warnings, errors)
            }
        };
        let mut typed_expression = match res.value {
            Some(r) => r,
            None => return res,
        };
        let mut warnings = res.warnings;
        let mut errors = res.errors;

        // if the return type cannot be cast into the annotation type then it is a type error
        append!(
            ctx.unify_with_self(typed_expression.return_type, &expr_span),
            warnings,
            errors
        );

        // The annotation may result in a cast, which is handled in the type engine.
        typed_expression.return_type = check!(
            ctx.resolve_type_with_self(
                typed_expression.return_type,
                &expr_span,
                EnforceTypeArguments::No,
                None
            ),
            type_engine.insert(engines, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // Literals of type Numeric can now be resolved if typed_expression.return_type is
        // an UnsignedInteger or a Numeric
        if let ty::TyExpressionVariant::Literal(lit) = typed_expression.clone().expression {
            if let Literal::Numeric(_) = lit {
                match type_engine.get(typed_expression.return_type) {
                    TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => {
                        typed_expression = check!(
                            Self::resolve_numeric_literal(
                                ctx,
                                lit,
                                expr_span,
                                typed_expression.return_type
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    _ => {}
                }
            }
        }

        ok(typed_expression, warnings, errors)
    }

    fn type_check_literal(
        engines: &Engines,
        lit: Literal,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let type_engine = engines.te();
        let return_type = match &lit {
            Literal::String(s) => TypeInfo::Str(Length::new(s.as_str().len(), s.clone())),
            Literal::Numeric(_) => TypeInfo::Numeric,
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::B256(_) => TypeInfo::B256,
        };
        let id = type_engine.insert(engines, return_type);
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(lit),
            return_type: id,
            span,
        };
        ok(exp, vec![], vec![])
    }

    pub(crate) fn type_check_variable_expression(
        ctx: TypeCheckContext,
        name: Ident,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let warnings = vec![];
        let mut errors = vec![];

        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let exp = match ctx.namespace.resolve_symbol(&name).value {
            Some(ty::TyDecl::VariableDecl(decl)) => {
                let ty::TyVariableDecl {
                    name: decl_name,
                    mutability,
                    return_type,
                    ..
                } = &**decl;
                ty::TyExpression {
                    return_type: *return_type,
                    expression: ty::TyExpressionVariant::VariableExpression {
                        name: decl_name.clone(),
                        span: name.span(),
                        mutability: *mutability,
                        call_path: Some(
                            CallPath::from(decl_name.clone()).to_fullpath(ctx.namespace),
                        ),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. })) => {
                let const_decl = decl_engine.get_constant(decl_id);
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
                let decl = decl_engine.get_abi(decl_id);
                ty::TyExpression {
                    return_type: decl.create_type_id(engines),
                    expression: ty::TyExpressionVariant::AbiName(AbiName::Known(decl.name.into())),
                    span,
                }
            }
            Some(a) => {
                errors.push(CompileError::NotAVariable {
                    name: name.clone(),
                    what_it_is: a.friendly_type_name(),
                    span,
                });
                ty::TyExpression::error(name.span(), engines)
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.clone(),
                    span,
                });
                ty::TyExpression::error(name.span(), engines)
            }
        };
        ok(exp, warnings, errors)
    }

    fn type_check_function_application(
        mut ctx: TypeCheckContext,
        mut call_path_binding: TypeBinding<CallPath>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // Grab the fn declaration.
        let (fn_ref, _, _): (DeclRefFunction, _, _) = check!(
            TypeBinding::type_check(&mut call_path_binding, ctx.by_ref()),
            return err(warnings, errors),
            warnings,
            errors
        );

        instantiate_function_application(ctx, fn_ref, call_path_binding, Some(arguments), span)
    }

    fn type_check_lazy_operator(
        ctx: TypeCheckContext,
        op: LazyOp,
        lhs: Expression,
        rhs: Expression,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut ctx = ctx.with_help_text("");
        let engines = ctx.engines();
        let typed_lhs = check!(
            ty::TyExpression::type_check(ctx.by_ref(), lhs.clone()),
            ty::TyExpression::error(lhs.span(), engines),
            warnings,
            errors
        );

        let typed_rhs = check!(
            ty::TyExpression::type_check(ctx.by_ref(), rhs.clone()),
            ty::TyExpression::error(rhs.span(), engines),
            warnings,
            errors
        );

        let type_annotation = ctx.type_annotation();
        let exp = instantiate_lazy_operator(op, typed_lhs, typed_rhs, type_annotation, span);
        ok(exp, warnings, errors)
    }

    fn type_check_code_block(
        mut ctx: TypeCheckContext,
        contents: CodeBlock,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let (typed_block, block_return_type) = check!(
            ty::TyCodeBlock::type_check(ctx.by_ref(), contents),
            (
                ty::TyCodeBlock { contents: vec![] },
                type_engine.insert(engines, TypeInfo::Tuple(Vec::new()))
            ),
            warnings,
            errors
        );

        append!(
            ctx.unify_with_self(block_return_type, &span),
            warnings,
            errors
        );

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                contents: typed_block.contents,
            }),
            return_type: block_return_type,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::type_complexity)]
    fn type_check_if_expression(
        mut ctx: TypeCheckContext,
        condition: Expression,
        then: Expression,
        r#else: Option<Expression>,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let condition = {
            let ctx = ctx
                .by_ref()
                .with_help_text("The condition of an if expression must be a boolean expression.")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Boolean));
            check!(
                ty::TyExpression::type_check(ctx, condition.clone()),
                ty::TyExpression::error(condition.span(), engines),
                warnings,
                errors
            )
        };
        let then = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, then.clone()),
                ty::TyExpression::error(then.span(), engines),
                warnings,
                errors
            )
        };
        let r#else = r#else.map(|expr| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, expr.clone()),
                ty::TyExpression::error(expr.span(), engines),
                warnings,
                errors
            )
        });
        let exp = check!(
            instantiate_if_expression(ctx, condition, then, r#else, span,),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_match_expression(
        mut ctx: TypeCheckContext,
        value: Expression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // type check the value
        let typed_value = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, value.clone()),
                ty::TyExpression::error(value.span(), engines),
                warnings,
                errors
            )
        };
        let type_id = typed_value.return_type;

        // check to make sure that the type of the value is something that can be matched upon
        check!(
            type_engine
                .get(type_id)
                .expect_is_supported_in_match_expressions(&typed_value.span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the match expression and create a ty::TyMatchExpression object
        let (typed_match_expression, typed_scrutinees) = {
            let ctx = ctx.by_ref().with_help_text("");
            check!(
                ty::TyMatchExpression::type_check(ctx, typed_value, branches, span.clone()),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        // check to see if the match expression is exhaustive and if all match arms are reachable
        let (witness_report, arms_reachability) = check!(
            check_match_expression_usefulness(
                engines,
                type_id,
                typed_scrutinees.clone(),
                span.clone()
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        for reachable_report in arms_reachability.into_iter() {
            if !reachable_report.reachable {
                warnings.push(CompileWarning {
                    span: reachable_report.span,
                    warning_content: Warning::MatchExpressionUnreachableArm,
                });
            }
        }
        if witness_report.has_witnesses() {
            errors.push(CompileError::MatchExpressionNonExhaustive {
                missing_patterns: format!("{witness_report}"),
                span,
            });
            return err(warnings, errors);
        }

        // desugar the typed match expression to a typed if expression
        let typed_if_exp = check!(
            typed_match_expression.convert_to_typed_if_expression(ctx),
            return err(warnings, errors),
            warnings,
            errors
        );

        let match_exp = ty::TyExpression {
            span: typed_if_exp.span.clone(),
            return_type: typed_if_exp.return_type,
            expression: ty::TyExpressionVariant::MatchExp {
                desugared: Box::new(typed_if_exp),
                scrutinees: typed_scrutinees,
            },
        };

        ok(match_exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        mut ctx: TypeCheckContext,
        asm: AsmExpression,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Various checks that we can catch early to check that the assembly is valid. For now,
        // this includes two checks:
        // 1. Check that no control flow opcodes are used.
        // 2. Check that initialized registers are not reassigned in the `asm` block.
        check!(
            check_asm_block_validity(&asm),
            return err(warnings, errors),
            warnings,
            errors
        );

        let asm_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());
        let return_type = check!(
            ctx.resolve_type_with_self(
                type_engine.insert(engines, asm.return_type.clone()),
                &asm_span,
                EnforceTypeArguments::No,
                None
            ),
            type_engine.insert(engines, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

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
                            check!(
                                ty::TyExpression::type_check(ctx, initializer.clone()),
                                ty::TyExpression::error(initializer.span(), engines),
                                warnings,
                                errors
                            )
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
        ok(exp, warnings, errors)
    }

    fn type_check_subfield_expression(
        ctx: TypeCheckContext,
        prefix: Expression,
        span: Span,
        field_to_access: Ident,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
        let parent = check!(
            ty::TyExpression::type_check(ctx, prefix),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = check!(
            instantiate_struct_field_access(engines, parent, field_to_access, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_tuple(
        mut ctx: TypeCheckContext,
        fields: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

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
            let typed_field = check!(
                ty::TyExpression::type_check(ctx, field),
                ty::TyExpression::error(field_span, engines),
                warnings,
                errors
            );
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
        ok(exp, warnings, errors)
    }

    /// Look up the current global storage state that has been created by storage declarations.
    /// If there isn't any storage, then this is an error. If there is storage, find the corresponding
    /// field that has been specified and return that value.
    fn type_check_storage_access(
        ctx: TypeCheckContext,
        checkee: Vec<Ident>,
        storage_keyword_span: Span,
        span: &Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        if !ctx.namespace.has_storage_declared() {
            errors.push(CompileError::NoDeclaredStorage { span: span.clone() });
            return err(warnings, errors);
        }

        let storage_fields = check!(
            ctx.namespace.get_storage_field_descriptors(decl_engine),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Do all namespace checking here!
        let (storage_access, mut access_type) = check!(
            ctx.namespace.apply_storage_load(
                ctx.engines,
                checkee,
                &storage_fields,
                storage_keyword_span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // The type of a storage access is `core::storage::StorageKey`. This is
        // the path to it.
        let storage_key_mod_path = vec![
            Ident::new_with_override("core".into(), span.clone()),
            Ident::new_with_override("storage".into(), span.clone()),
        ];
        let storage_key_ident = Ident::new_with_override("StorageKey".into(), span.clone());

        // Search for the struct declaration with the call path above.
        let storage_key_decl_opt = check!(
            ctx.namespace
                .root()
                .resolve_symbol(&storage_key_mod_path, &storage_key_ident),
            return err(warnings, errors),
            warnings,
            errors
        );
        let storage_key_struct_decl_ref = check!(
            storage_key_decl_opt.to_struct_ref(engines),
            return err(warnings, errors),
            warnings,
            errors
        );
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
        check!(
            ctx.monomorphize(
                &mut storage_key_struct_decl,
                &mut type_arguments,
                EnforceTypeArguments::Yes,
                span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Update `access_type` to be the type of the monomorphized struct after inserting it
        // into the type engine
        let storage_key_struct_decl_ref = ctx.engines().de().insert(storage_key_struct_decl);
        access_type = type_engine.insert(engines, TypeInfo::Struct(storage_key_struct_decl_ref));

        // take any trait items that apply to `StorageKey<T>` and copy them to the
        // monomorphized type
        ctx.namespace
            .insert_trait_implementation_for_type(engines, access_type);

        ok(
            ty::TyExpression {
                expression: ty::TyExpressionVariant::StorageAccess(storage_access),
                return_type: access_type,
                span: span.clone(),
            },
            warnings,
            errors,
        )
    }

    fn type_check_tuple_index(
        ctx: TypeCheckContext,
        prefix: Expression,
        index: usize,
        index_span: Span,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
        let parent = check!(
            ty::TyExpression::type_check(ctx, prefix),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = check!(
            instantiate_tuple_index_access(engines, parent, index, index_span, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_ambiguous_path(
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
    ) -> CompileResult<ty::TyExpression> {
        let decl_engine = ctx.engines.de();

        if let Some(QualifiedPathRootTypes { ty, as_trait, .. }) = qualified_path_root {
            if !prefixes.is_empty() || before.is_some() {
                return err(
                    vec![],
                    vec![
                        ConvertParseTreeError::UnexpectedCallPathPrefixAfterQualifiedRoot {
                            span: path_span,
                        }
                        .into(),
                    ],
                );
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
                ctx.namespace
                    .resolve_call_path(&call_path_binding.inner)
                    .value,
                Some(ty::TyDecl::EnumVariantDecl { .. })
            ) {
                return Self::type_check_delineated_path(ctx, call_path_binding, span, Some(args));
            } else {
                return Self::type_check_function_application(
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
        let not_module = ctx.namespace.check_submodule(&path).value.is_none();

        // Not a module? Not a `Enum::Variant` either?
        // Type check as an associated function call instead.
        let is_associated_call = not_module && {
            let probe_call_path = CallPath {
                prefixes: prefixes.clone(),
                suffix: before.inner.clone(),
                is_absolute,
            };
            ctx.namespace
                .resolve_call_path(&probe_call_path)
                .flat_map(|decl| decl.to_enum_ref(ctx.engines()))
                .map(|decl_ref| decl_engine.get_enum(&decl_ref))
                .flat_map(|decl| decl.expect_variant_from_name(&suffix).map(drop))
                .value
                .is_none()
        };

        if is_associated_call {
            let before_span = before.span();
            let type_name = before.inner;
            let type_info = type_name_to_type_info_opt(&type_name).unwrap_or(TypeInfo::Custom {
                call_path: type_name.clone().into(),
                type_arguments: None,
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
            type_check_method_application(ctx.by_ref(), method_name_binding, Vec::new(), args, span)
        } else {
            let mut type_arguments = type_arguments;
            if let TypeArgs::Regular(vec) = before.type_arguments {
                if !vec.is_empty() {
                    if !type_arguments.to_vec().is_empty() {
                        return err(
                            vec![],
                            vec![
                                ConvertParseTreeError::MultipleGenericsNotSupported { span }.into()
                            ],
                        );
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
            Self::type_check_delineated_path(ctx, call_path_binding, span, Some(args))
        }
    }

    fn type_check_delineated_path(
        mut ctx: TypeCheckContext,
        unknown_call_path_binding: TypeBinding<CallPath>,
        span: Span,
        args: Option<Vec<Expression>>,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // The first step is to determine if the call path refers to a module,
        // enum, function or constant.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        // Check if this could be a module
        let mut module_probe_warnings = Vec::new();
        let mut module_probe_errors = Vec::new();
        let is_module = {
            let call_path_binding = unknown_call_path_binding.clone();
            ctx.namespace
                .check_submodule(
                    &[
                        call_path_binding.inner.prefixes,
                        vec![call_path_binding.inner.suffix],
                    ]
                    .concat(),
                )
                .ok(&mut module_probe_warnings, &mut module_probe_errors)
                .is_some()
        };

        // Check if this could be a function
        let mut function_probe_warnings = Vec::new();
        let mut function_probe_errors = Vec::new();
        let maybe_function: Option<(DeclRefFunction, _)> = {
            let mut call_path_binding = unknown_call_path_binding.clone();
            TypeBinding::type_check(&mut call_path_binding, ctx.by_ref())
                .ok(&mut function_probe_warnings, &mut function_probe_errors)
                .map(|(fn_ref, _, _)| (fn_ref, call_path_binding))
        };

        // Check if this could be an enum
        let mut enum_probe_warnings = vec![];
        let mut enum_probe_errors = vec![];
        let maybe_enum: Option<(DeclRefEnum, _, _, _)> = {
            let call_path_binding = unknown_call_path_binding.clone();
            let variant_name = call_path_binding.inner.suffix.clone();
            let enum_call_path = call_path_binding.inner.rshift();

            let mut call_path_binding = TypeBinding {
                inner: enum_call_path,
                type_arguments: call_path_binding.type_arguments,
                span: call_path_binding.span,
            };
            TypeBinding::type_check(&mut call_path_binding, ctx.by_ref())
                .ok(&mut enum_probe_warnings, &mut enum_probe_errors)
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
        let mut const_probe_warnings = vec![];
        let mut const_probe_errors = vec![];
        let maybe_const = {
            Self::probe_const_decl(
                &unknown_call_path_binding,
                &mut ctx,
                &mut const_probe_warnings,
                &mut const_probe_errors,
            )
        };

        // compare the results of the checks
        let exp = match (is_module, maybe_function, maybe_enum, maybe_const) {
            (
                false,
                None,
                Some((enum_ref, variant_name, call_path_binding, call_path_decl)),
                None,
            ) => {
                warnings.append(&mut enum_probe_warnings);
                errors.append(&mut enum_probe_errors);
                check!(
                    instantiate_enum(
                        ctx,
                        enum_ref,
                        variant_name,
                        args,
                        call_path_binding,
                        call_path_decl,
                        &span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (false, Some((fn_ref, call_path_binding)), None, None) => {
                warnings.append(&mut function_probe_warnings);
                errors.append(&mut function_probe_errors);
                // In case `foo::bar::<TyArgs>::baz(...)` throw an error.
                if let TypeArgs::Prefix(_) = call_path_binding.type_arguments {
                    errors.push(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: call_path_binding.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                check!(
                    instantiate_function_application(ctx, fn_ref, call_path_binding, args, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (true, None, None, None) => {
                module_probe_errors.push(CompileError::Unimplemented(
                    "this case is not yet implemented",
                    span,
                ));
                return err(module_probe_warnings, module_probe_errors);
            }
            (false, None, None, Some((const_ref, call_path_binding))) => {
                warnings.append(&mut const_probe_warnings);
                errors.append(&mut const_probe_errors);
                if !call_path_binding.type_arguments.to_vec().is_empty() {
                    // In case `foo::bar::CONST::<TyArgs>` throw an error.
                    // In case `foo::bar::<TyArgs>::CONST` throw an error.
                    errors.push(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: unknown_call_path_binding.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                check!(
                    instantiate_constant_expression(ctx, const_ref, call_path_binding),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (false, None, None, None) => {
                errors.push(CompileError::SymbolNotFound {
                    name: unknown_call_path_binding.inner.suffix.clone(),
                    span: unknown_call_path_binding.inner.suffix.span(),
                });
                return err(warnings, errors);
            }
            _ => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
        };
        ok(exp, warnings, errors)
    }

    fn probe_const_decl(
        unknown_call_path_binding: &TypeBinding<CallPath>,
        ctx: &mut TypeCheckContext,
        const_probe_warnings: &mut Vec<CompileWarning>,
        const_probe_errors: &mut Vec<CompileError>,
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
                })
            });

        if let Some(TypeInfo::SelfType) = type_info_opt {
            call_path_binding.strip_prefixes();
        }

        let const_opt: Option<(DeclRefConstant, _)> =
            TypeBinding::type_check(&mut call_path_binding, ctx.by_ref())
                .ok(const_probe_warnings, const_probe_errors)
                .map(|(const_ref, _, _)| (const_ref, call_path_binding.clone()));
        if const_opt.is_some() {
            return const_opt;
        }

        *const_probe_warnings = vec![];
        *const_probe_errors = vec![];

        // If we didn't find a constant, check for the constant inside the impl.
        let suffix = call_path_binding.inner.suffix.clone();
        let const_call_path = call_path_binding.inner.rshift();

        let mut const_call_path_binding = TypeBinding {
            inner: const_call_path,
            type_arguments: call_path_binding.type_arguments.clone(),
            span: call_path_binding.span.clone(),
        };

        let (_, struct_type_id, _): (DeclRefStruct, _, _) = check!(
            TypeBinding::type_check(&mut const_call_path_binding, ctx.by_ref()),
            return None,
            const_probe_warnings,
            const_probe_errors
        );

        let const_decl_ref = check!(
            ctx.namespace.find_constant_for_type(
                struct_type_id.unwrap(),
                &suffix,
                ctx.self_type(),
                ctx.engines(),
            ),
            return None,
            const_probe_warnings,
            const_probe_errors
        );

        Some((const_decl_ref, call_path_binding.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        mut ctx: TypeCheckContext,
        abi_name: CallPath,
        address: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

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
            check!(
                ty::TyExpression::type_check(ctx, address),
                ty::TyExpression::error(err_span, engines),
                warnings,
                errors
            )
        };

        // look up the call path and get the declaration it references
        let abi = check!(
            ctx.namespace.resolve_call_path(&abi_name).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
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
                        errors.push(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: abi.friendly_type_name(),
                        });
                        return err(warnings, errors);
                    }
                };
                match abi_name {
                    // look up the call path and get the declaration it references
                    AbiName::Known(abi_name) => {
                        let unknown_decl = check!(
                            ctx.namespace.resolve_call_path(&abi_name).cloned(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        check!(
                            unknown_decl.to_abi_ref(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    AbiName::Deferred => {
                        return ok(
                            ty::TyExpression {
                                return_type: type_engine.insert(
                                    engines,
                                    TypeInfo::ContractCaller {
                                        abi_name: AbiName::Deferred,
                                        address: None,
                                    },
                                ),
                                expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                                span,
                            },
                            warnings,
                            errors,
                        )
                    }
                }
            }
            a => {
                errors.push(CompileError::NotAnAbi {
                    span: abi_name.span(),
                    actually_is: a.friendly_type_name(),
                });
                return err(warnings, errors);
            }
        };
        let ty::TyAbiDecl {
            interface_surface,
            items,
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
                            .insert(method.to_dummy_func(AbiMode::ImplAbiFn))
                            .with_parent(decl_engine, (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(&decl_ref);
                    abi_items.push(TyImplItem::Constant(decl_engine.insert(const_decl)));
                }
            }
        }

        // Retrieve the items for this abi.
        abi_items.append(&mut items.into_iter().collect::<Vec<_>>());

        // Insert the abi methods into the namespace.
        check!(
            ctx.namespace.insert_trait_implementation(
                abi_name.clone(),
                vec![],
                return_type,
                &abi_items,
                &span,
                Some(span.clone()),
                false,
                engines,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::AbiCast {
                abi_name,
                address: Box::new(address_expr),
                span: span.clone(),
            },
            return_type,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_array(
        mut ctx: TypeCheckContext,
        contents: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        if contents.is_empty() {
            let unknown_type = type_engine.insert(engines, TypeInfo::Unknown);
            return ok(
                ty::TyExpression {
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
                },
                Vec::new(),
                Vec::new(),
            );
        };

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_contents: Vec<ty::TyExpression> = contents
            .into_iter()
            .map(|expr| {
                let span = expr.span();
                let ctx = ctx
                    .by_ref()
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
                check!(
                    Self::type_check(ctx, expr),
                    ty::TyExpression::error(span, engines),
                    warnings,
                    errors
                )
            })
            .collect();

        let elem_type = typed_contents[0].return_type;
        for typed_elem in &typed_contents[1..] {
            let (mut new_warnings, mut new_errors) = ctx
                .by_ref()
                .with_type_annotation(elem_type)
                .unify_with_self(typed_elem.return_type, &typed_elem.span);
            let no_warnings = new_warnings.is_empty();
            let no_errors = new_errors.is_empty();
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors);
            // In both cases, if there are warnings or errors then break here, since we don't
            // need to spam type errors for every element once we have one.
            if !no_warnings && !no_errors {
                break;
            }
        }

        let array_count = typed_contents.len();
        ok(
            ty::TyExpression {
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
            },
            warnings,
            errors,
        )
    }

    fn type_check_array_index(
        mut ctx: TypeCheckContext,
        prefix: Expression,
        index: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let prefix_te = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, prefix.clone()),
                return err(warnings, errors),
                warnings,
                errors
            )
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
            let index_te = check!(
                ty::TyExpression::type_check(ctx, index),
                return err(warnings, errors),
                warnings,
                errors
            );

            ok(
                ty::TyExpression {
                    expression: ty::TyExpressionVariant::ArrayIndex {
                        prefix: Box::new(prefix_te),
                        index: Box::new(index_te),
                    },
                    return_type: elem_type.type_id,
                    span,
                },
                warnings,
                errors,
            )
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
            type_check_method_application(ctx, method_name, vec![], vec![prefix, index], span)
        }
    }

    fn type_check_intrinsic_function(
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (intrinsic_function, return_type) = check!(
            ty::TyIntrinsicFunctionKind::type_check(ctx, kind_binding, arguments, span.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::IntrinsicFunction(intrinsic_function),
            return_type,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_while_loop(
        mut ctx: TypeCheckContext,
        condition: Expression,
        body: CodeBlock,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let typed_condition = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Boolean))
                .with_help_text("A while loop's loop condition must be a boolean expression.");
            check!(
                ty::TyExpression::type_check(ctx, condition),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        let unit_ty = type_engine.insert(engines, TypeInfo::Tuple(Vec::new()));
        let ctx = ctx.with_type_annotation(unit_ty).with_help_text(
            "A while loop's loop body cannot implicitly return a value. Try \
                 assigning it to a mutable variable declared outside of the loop \
                 instead.",
        );
        let (typed_body, _block_implicit_return) = check!(
            ty::TyCodeBlock::type_check(ctx, body),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::WhileLoop {
                condition: Box::new(typed_condition),
                body: typed_body,
            },
            return_type: unit_ty,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_reassignment(
        ctx: TypeCheckContext,
        lhs: ReassignmentTarget,
        rhs: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

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
                            let unknown_decl = check!(
                                ctx.namespace.resolve_symbol(&name).cloned(),
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
                            if !variable_decl.mutability.is_mutable() {
                                errors.push(CompileError::AssignmentToNonMutable { name, span });
                                return err(warnings, errors);
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
                            let typed_index = check!(
                                ty::TyExpression::type_check(ctx, index.as_ref().clone()),
                                ty::TyExpression::error(span.clone(), engines),
                                warnings,
                                errors
                            );
                            names_vec.push(ty::ProjectionKind::ArrayIndex {
                                index: Box::new(typed_index),
                                index_span: index.span(),
                            });
                            expr = prefix;
                        }
                        _ => {
                            errors.push(CompileError::InvalidExpressionOnLhs { span });
                            return err(warnings, errors);
                        }
                    }
                };
                let names_vec = names_vec.into_iter().rev().collect::<Vec<_>>();
                let (ty_of_field, _ty_of_parent) = check!(
                    ctx.namespace
                        .find_subfield_type(ctx.engines(), &base_name, &names_vec),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the reassignment
                let ctx = ctx.with_type_annotation(ty_of_field).with_help_text("");
                let rhs_span = rhs.span();
                let rhs = check!(
                    ty::TyExpression::type_check(ctx, rhs),
                    ty::TyExpression::error(rhs_span, engines),
                    warnings,
                    errors
                );

                ok(
                    ty::TyExpression {
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
                    },
                    warnings,
                    errors,
                )
            }
        }
    }

    fn resolve_numeric_literal(
        ctx: TypeCheckContext,
        lit: Literal,
        span: Span,
        new_type: TypeId,
    ) -> CompileResult<ty::TyExpression> {
        let mut errors = vec![];

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
                ok(exp, vec![], vec![])
            }
            Err(e) => {
                errors.push(e);
                let exp = ty::TyExpression::error(span, engines);
                ok(exp, vec![], errors)
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
        engines: &Engines,
        expr: Expression,
        type_annotation: TypeId,
    ) -> CompileResult<ty::TyExpression> {
        let mut namespace = Namespace::init_root(namespace::Module::default());
        let ctx = TypeCheckContext::from_root(&mut namespace, engines)
            .with_type_annotation(type_annotation);
        ty::TyExpression::type_check(ctx, expr)
    }

    fn do_type_check_for_boolx2(expr: Expression) -> CompileResult<ty::TyExpression> {
        let engines = Engines::default();
        do_type_check(
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

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
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

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 2);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected == "u64"
                                && received == "bool"));
        assert!(matches!(&comp_res.errors[1],
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

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
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
        let comp_res = do_type_check(
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
        assert!(comp_res.warnings.is_empty() && comp_res.errors.is_empty());
    }
}

fn check_asm_block_validity(asm: &AsmExpression) -> CompileResult<()> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Collect all asm block instructions in the form of `VirtualOp`s
    let mut opcodes = vec![];
    for op in &asm.body {
        let registers = op
            .op_args
            .iter()
            .map(|reg_name| VirtualRegister::Virtual(reg_name.to_string()))
            .collect::<Vec<VirtualRegister>>();

        opcodes.push((
            check!(
                crate::asm_lang::Op::parse_opcode(
                    &op.op_name,
                    &registers,
                    &op.immediate,
                    op.span.clone(),
                ),
                return err(warnings, errors),
                warnings,
                errors
            ),
            op.op_name.clone(),
            op.span.clone(),
        ));
    }

    // Check #1: Disallow control flow instructions
    //
    errors.extend(
        opcodes
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
            .collect::<Vec<_>>(),
    );

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
    errors.extend(
        asm.registers
            .iter()
            .filter(|reg| {
                initialized_and_assigned_registers
                    .contains(&VirtualRegister::Virtual(reg.name.to_string()))
            })
            .map(|reg| CompileError::InitializedRegisterReassignment {
                name: reg.name.to_string(),
                span: reg.name.span(),
            })
            .collect::<Vec<_>>(),
    );

    ok((), vec![], errors)
}
