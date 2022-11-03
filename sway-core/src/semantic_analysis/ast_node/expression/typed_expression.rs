mod constant_declaration;
mod enum_instantiation;
mod function_application;
mod if_expression;
mod lazy_operator;
mod method_application;
mod struct_field_access;
mod tuple_index_access;
mod unsafe_downcast;

use self::constant_declaration::instantiate_constant_decl;
pub(crate) use self::{
    enum_instantiation::*, function_application::*, if_expression::*, lazy_operator::*,
    method_application::*, struct_field_access::*, tuple_index_access::*, unsafe_downcast::*,
};

use crate::{
    asm_lang::virtual_register::VirtualRegister,
    declaration_engine::declaration_engine::*,
    error::*,
    language::{parsed::*, ty, *},
    semantic_analysis::*,
    transform::to_parsed_lang::type_name_to_type_info_opt,
    type_system::*,
};

use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{integer_bits::IntegerBits, Ident, Span, Spanned};

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
        let call_path = CallPath {
            prefixes: vec![
                Ident::new_with_override("core", span.clone()),
                Ident::new_with_override("ops", span.clone()),
            ],
            suffix: Op {
                op_variant: OpVariant::Equals,
                span: span.clone(),
            }
            .to_var_name(),
            is_absolute: true,
        };
        let method_name_binding = TypeBinding {
            inner: MethodName::FromTrait {
                call_path: call_path.clone(),
            },
            type_arguments: vec![],
            span: call_path.span(),
        };
        let arguments = VecDeque::from(arguments);
        let decl_id = check!(
            resolve_method_name(ctx, &method_name_binding, arguments.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let method = check!(
            CompileResult::from(de_get_function(
                decl_id.clone(),
                &method_name_binding.span()
            )),
            return err(warnings, errors),
            warnings,
            errors
        );
        // check that the number of parameters and the number of the arguments is the same
        check!(
            check_function_arguments_arity(arguments.len(), &method, &call_path),
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
                function_decl_id: decl_id,
                self_state_idx: None,
                selector: None,
            },
            return_type,
            span,
        };
        ok(exp, warnings, errors)
    }

    pub(crate) fn type_check(mut ctx: TypeCheckContext, expr: Expression) -> CompileResult<Self> {
        let expr_span = expr.span();
        let span = expr_span.clone();
        let res = match expr.kind {
            // We've already emitted an error for the `::Error` case.
            ExpressionKind::Error(_) => ok(ty::TyExpression::error(span), vec![], vec![]),
            ExpressionKind::Literal(lit) => Self::type_check_literal(lit, span),
            ExpressionKind::Variable(name) => {
                Self::type_check_variable_expression(ctx.namespace, name, span)
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
                    .with_type_annotation(insert_type(TypeInfo::Boolean));
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
                Self::type_check_struct_expression(ctx.by_ref(), call_path_binding, fields, span)
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
                } = *e;
                Self::type_check_ambiguous_path(ctx.by_ref(), call_path_binding, span, args)
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
            ExpressionKind::Array(contents) => Self::type_check_array(ctx.by_ref(), contents, span),
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_array_index(ctx, *prefix, *index, span)
            }
            ExpressionKind::StorageAccess(StorageAccessExpression { field_names }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_storage_load(ctx, field_names, &span)
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                kind_binding,
                arguments,
            }) => Self::type_check_intrinsic_function(ctx.by_ref(), kind_binding, arguments, span),
            ExpressionKind::WhileLoop(WhileLoopExpression { condition, body }) => {
                Self::type_check_while_loop(ctx.by_ref(), *condition, body, span)
            }
            ExpressionKind::Break => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Break,
                    return_type: insert_type(TypeInfo::Unknown),
                    span,
                };
                ok(expr, vec![], vec![])
            }
            ExpressionKind::Continue => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Continue,
                    return_type: insert_type(TypeInfo::Unknown),
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
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text(
                        "Returned value must match up with the function return type \
                        annotation.",
                    );
                let mut warnings = vec![];
                let mut errors = vec![];
                let expr_span = expr.span();
                let expr = check!(
                    ty::TyExpression::type_check(ctx, *expr),
                    ty::TyExpression::error(expr_span),
                    warnings,
                    errors,
                );
                let typed_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Return(Box::new(expr)),
                    return_type: insert_type(TypeInfo::Unknown),
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
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // Literals of type Numeric can now be resolved if typed_expression.return_type is
        // an UnsignedInteger or a Numeric
        if let ty::TyExpressionVariant::Literal(lit) = typed_expression.clone().expression {
            if let Literal::Numeric(_) = lit {
                match look_up_type_id(typed_expression.return_type) {
                    TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => {
                        typed_expression = check!(
                            Self::resolve_numeric_literal(
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

    fn type_check_literal(lit: Literal, span: Span) -> CompileResult<ty::TyExpression> {
        let return_type = match &lit {
            Literal::String(s) => TypeInfo::Str(s.as_str().len() as u64),
            Literal::Numeric(_) => TypeInfo::Numeric,
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::B256(_) => TypeInfo::B256,
        };
        let id = crate::type_system::insert_type(return_type);
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(lit),
            return_type: id,
            span,
        };
        ok(exp, vec![], vec![])
    }

    pub(crate) fn type_check_variable_expression(
        namespace: &Namespace,
        name: Ident,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let exp = match namespace.resolve_symbol(&name).value {
            Some(ty::TyDeclaration::VariableDeclaration(decl)) => {
                let ty::TyVariableDeclaration {
                    name: decl_name,
                    body,
                    mutability,
                    ..
                } = &**decl;
                ty::TyExpression {
                    return_type: body.return_type,
                    expression: ty::TyExpressionVariant::VariableExpression {
                        name: decl_name.clone(),
                        span: name.span(),
                        mutability: *mutability,
                    },
                    span,
                }
            }
            Some(ty::TyDeclaration::ConstantDeclaration(decl_id)) => {
                let ty::TyConstantDeclaration {
                    name: decl_name,
                    value,
                    ..
                } = check!(
                    CompileResult::from(de_get_constant(decl_id.clone(), &span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ty::TyExpression {
                    return_type: value.return_type,
                    // Although this isn't strictly a 'variable' expression we can treat it as one for
                    // this context.
                    expression: ty::TyExpressionVariant::VariableExpression {
                        name: decl_name,
                        span: name.span(),
                        mutability: ty::VariableMutability::Immutable,
                    },
                    span,
                }
            }
            Some(ty::TyDeclaration::AbiDeclaration(decl_id)) => {
                let decl = check!(
                    CompileResult::from(de_get_abi(decl_id.clone(), &span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ty::TyExpression {
                    return_type: decl.create_type_id(),
                    expression: ty::TyExpressionVariant::AbiName(AbiName::Known(decl.name.into())),
                    span,
                }
            }
            Some(a) => {
                errors.push(CompileError::NotAVariable {
                    name: name.clone(),
                    what_it_is: a.friendly_name(),
                });
                ty::TyExpression::error(name.span())
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.clone(),
                });
                ty::TyExpression::error(name.span())
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

        // type check the declaration
        let unknown_decl = check!(
            TypeBinding::type_check_with_ident(&mut call_path_binding, ctx.by_ref()),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check that the decl is a function decl
        let function_decl = check!(
            unknown_decl.expect_function(&span),
            return err(warnings, errors),
            warnings,
            errors
        );

        instantiate_function_application(ctx, function_decl, call_path_binding.inner, arguments)
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
        let typed_lhs = check!(
            ty::TyExpression::type_check(ctx.by_ref(), lhs.clone()),
            ty::TyExpression::error(lhs.span()),
            warnings,
            errors
        );

        let typed_rhs = check!(
            ty::TyExpression::type_check(ctx.by_ref(), rhs.clone()),
            ty::TyExpression::error(rhs.span()),
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
        let (typed_block, block_return_type) = check!(
            ty::TyCodeBlock::type_check(ctx.by_ref(), contents),
            (
                ty::TyCodeBlock { contents: vec![] },
                crate::type_system::insert_type(TypeInfo::Tuple(Vec::new()))
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
        let condition = {
            let ctx = ctx
                .by_ref()
                .with_help_text("The condition of an if expression must be a boolean expression.")
                .with_type_annotation(insert_type(TypeInfo::Boolean));
            check!(
                ty::TyExpression::type_check(ctx, condition.clone()),
                ty::TyExpression::error(condition.span()),
                warnings,
                errors
            )
        };
        let then = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, then.clone()),
                ty::TyExpression::error(then.span()),
                warnings,
                errors
            )
        };
        let r#else = r#else.map(|expr| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, expr.clone()),
                ty::TyExpression::error(expr.span()),
                warnings,
                errors
            )
        });
        let exp = check!(
            instantiate_if_expression(
                condition,
                then,
                r#else,
                span,
                ctx.type_annotation(),
                ctx.self_type(),
            ),
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

        // type check the value
        let typed_value = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, value.clone()),
                ty::TyExpression::error(value.span()),
                warnings,
                errors
            )
        };
        let type_id = typed_value.return_type;

        // check to make sure that the type of the value is something that can be matched upon
        check!(
            look_up_type_id(type_id).expect_is_supported_in_match_expressions(&typed_value.span),
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
                ctx.namespace,
                type_id,
                typed_scrutinees,
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
                missing_patterns: format!("{}", witness_report),
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

        ok(typed_if_exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        mut ctx: TypeCheckContext,
        asm: AsmExpression,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let asm_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());
        let return_type = check!(
            ctx.resolve_type_with_self(
                insert_type(asm.return_type.clone()),
                &asm_span,
                EnforceTypeArguments::No,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // type check the initializers
        let typed_registers = asm
            .registers
            .clone()
            .into_iter()
            .map(
                |AsmRegisterDeclaration { name, initializer }| ty::TyAsmRegisterDeclaration {
                    name,
                    initializer: initializer.map(|initializer| {
                        let ctx = ctx
                            .by_ref()
                            .with_help_text("")
                            .with_type_annotation(insert_type(TypeInfo::Unknown));
                        check!(
                            ty::TyExpression::type_check(ctx, initializer.clone()),
                            ty::TyExpression::error(initializer.span()),
                            warnings,
                            errors
                        )
                    }),
                },
            )
            .collect();

        // Make sure that all registers that are initialized are *not* assigned again.
        check!(
            disallow_assigning_initialized_registers(&asm),
            return err(warnings, errors),
            warnings,
            errors
        );

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

    #[allow(clippy::too_many_arguments)]
    fn type_check_struct_expression(
        mut ctx: TypeCheckContext,
        call_path_binding: TypeBinding<CallPath>,
        fields: Vec<StructExpressionField>,
        span: Span,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypeBinding {
            inner: CallPath {
                prefixes, suffix, ..
            },
            type_arguments,
            span: inner_span,
        } = call_path_binding;
        let type_info = match (suffix.as_str(), type_arguments.is_empty()) {
            ("Self", true) => TypeInfo::SelfType,
            ("Self", false) => {
                errors.push(CompileError::TypeArgumentsNotAllowed {
                    span: suffix.span(),
                });
                return err(warnings, errors);
            }
            (_, true) => TypeInfo::Custom {
                name: suffix,
                type_arguments: None,
            },
            (_, false) => TypeInfo::Custom {
                name: suffix,
                type_arguments: Some(type_arguments),
            },
        };

        // find the module that the struct decl is in
        let type_info_prefix = ctx.namespace.find_module_path(&prefixes);
        check!(
            ctx.namespace.root().check_submodule(&type_info_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // resolve the type of the struct decl
        let type_id = check!(
            ctx.resolve_type_with_self(
                insert_type(type_info),
                &inner_span,
                EnforceTypeArguments::No,
                Some(&type_info_prefix)
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        // extract the struct name and fields from the type info
        let type_info = look_up_type_id(type_id);
        let (struct_name, struct_fields) = check!(
            type_info.expect_struct(&span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut struct_fields = struct_fields.clone();

        // match up the names with their type annotations from the declaration
        let mut typed_fields_buf = vec![];
        for def_field in struct_fields.iter_mut() {
            let expr_field: StructExpressionField =
                match fields.iter().find(|x| x.name == def_field.name) {
                    Some(val) => val.clone(),
                    None => {
                        errors.push(CompileError::StructMissingField {
                            field_name: def_field.name.clone(),
                            struct_name: struct_name.clone(),
                            span: span.clone(),
                        });
                        typed_fields_buf.push(ty::TyStructExpressionField {
                            name: def_field.name.clone(),
                            value: ty::TyExpression {
                                expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                                return_type: insert_type(TypeInfo::ErrorRecovery),
                                span: span.clone(),
                            },
                        });
                        continue;
                    }
                };

            let ctx = ctx
                .by_ref()
                .with_help_text(
                    "Struct field's type must match up with the type specified in its declaration.",
                )
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            let typed_field = check!(
                ty::TyExpression::type_check(ctx, expr_field.value),
                continue,
                warnings,
                errors
            );
            append!(
                unify_adt(
                    typed_field.return_type,
                    def_field.type_id,
                    &typed_field.span,
                    "Struct field's type must match up with the type specified in its declaration.",
                ),
                warnings,
                errors
            );

            def_field.span = typed_field.span.clone();
            typed_fields_buf.push(ty::TyStructExpressionField {
                value: typed_field,
                name: expr_field.name.clone(),
            });
        }

        // check that there are no extra fields
        for field in fields {
            if !struct_fields.iter().any(|x| x.name == field.name) {
                errors.push(CompileError::StructDoesNotHaveField {
                    field_name: field.name.clone(),
                    struct_name: struct_name.clone(),
                    span: field.span,
                });
            }
        }
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::StructExpression {
                struct_name: struct_name.clone(),
                fields: typed_fields_buf,
                span: inner_span,
            },
            return_type: type_id,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_subfield_expression(
        ctx: TypeCheckContext,
        prefix: Expression,
        span: Span,
        field_to_access: Ident,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        let parent = check!(
            ty::TyExpression::type_check(ctx, prefix),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = check!(
            instantiate_struct_field_access(parent, field_to_access, span),
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
        let field_type_opt = match look_up_type_id(ctx.type_annotation()) {
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
                .unwrap_or_default();
            let field_span = field.span();
            let ctx = ctx
                .by_ref()
                .with_help_text("tuple field type does not match the expected type")
                .with_type_annotation(field_type.type_id);
            let typed_field = check!(
                ty::TyExpression::type_check(ctx, field),
                ty::TyExpression::error(field_span),
                warnings,
                errors
            );
            typed_field_types.push(TypeArgument {
                type_id: typed_field.return_type,
                initial_type_id: field_type.type_id,
                span: typed_field.span.clone(),
            });
            typed_fields.push(typed_field);
        }
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::Tuple {
                fields: typed_fields,
            },
            return_type: crate::type_system::insert_type(TypeInfo::Tuple(typed_field_types)),
            span,
        };
        ok(exp, warnings, errors)
    }

    /// Look up the current global storage state that has been created by storage declarations.
    /// If there isn't any storage, then this is an error. If there is storage, find the corresponding
    /// field that has been specified and return that value.
    fn type_check_storage_load(
        ctx: TypeCheckContext,
        checkee: Vec<Ident>,
        span: &Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if !ctx.namespace.has_storage_declared() {
            errors.push(CompileError::NoDeclaredStorage { span: span.clone() });
            return err(warnings, errors);
        }

        let storage_fields = check!(
            ctx.namespace.get_storage_field_descriptors(span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Do all namespace checking here!
        let (storage_access, return_type) = check!(
            ctx.namespace
                .apply_storage_load(checkee, &storage_fields, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(
            ty::TyExpression {
                expression: ty::TyExpressionVariant::StorageAccess(storage_access),
                return_type,
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
        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        let parent = check!(
            ty::TyExpression::type_check(ctx, prefix),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = check!(
            instantiate_tuple_index_access(parent, index, index_span, span),
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
    ) -> CompileResult<ty::TyExpression> {
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
                .flat_map(|decl| decl.expect_enum(&before.inner.span()))
                .flat_map(|decl| decl.expect_variant_from_name(&suffix).map(drop))
                .value
                .is_none()
        };

        if is_associated_call {
            let before_span = before.span();
            let type_name = before.inner;
            let type_info_span = type_name.span();
            let type_info = type_name_to_type_info_opt(&type_name).unwrap_or(TypeInfo::Custom {
                name: type_name,
                type_arguments: None,
            });

            let method_name_binding = TypeBinding {
                inner: MethodName::FromType {
                    call_path_binding: TypeBinding {
                        span: before_span,
                        type_arguments: before.type_arguments,
                        inner: CallPath {
                            prefixes,
                            suffix: (type_info, type_info_span),
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
            let call_path_binding = TypeBinding {
                inner: CallPath {
                    prefixes: path,
                    suffix,
                    is_absolute,
                },
                type_arguments,
                span: path_span,
            };
            let mut res = Self::type_check_delineated_path(ctx, call_path_binding, span, args);

            // In case `before` has type args, this would be e.g., `foo::bar::<TyArgs>::baz(...)`.
            // So, we would need, but don't have, parametric modules to apply arguments to.
            // Emit an error and ignore the type args.
            //
            // TODO: This also bans `Enum::<TyArgs>::Variant` but there's no good reason to ban that.
            // Instead, we should allow this but ban `Enum::Variant::<TyArgs>`, which Rust does allow,
            // but shouldn't, because with GADTs, we could ostensibly have the equivalent of:
            // ```haskell
            // {-# LANGUAGE GADTs, RankNTypes #-}
            // data Foo where Bar :: forall a. Show a => a -> Foo
            // ```
            // or to illustrate with Sway-ish syntax:
            // ```rust
            // enum Foo {
            //     Bar<A: Debug>: A, // Let's ignore memory representation, etc.
            // }
            // ```
            if !before.type_arguments.is_empty() {
                res.errors.push(
                    ConvertParseTreeError::GenericsNotSupportedHere {
                        span: Span::join_all(before.type_arguments.iter().map(|t| t.span())),
                    }
                    .into(),
                );
            }

            res
        }
    }

    fn type_check_delineated_path(
        mut ctx: TypeCheckContext,
        call_path_binding: TypeBinding<CallPath>,
        span: Span,
        args: Vec<Expression>,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // The first step is to determine if the call path refers to a module, enum, or function.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        // Check if this could be a module
        let mut module_probe_warnings = Vec::new();
        let mut module_probe_errors = Vec::new();
        let is_module = {
            let call_path_binding = call_path_binding.clone();
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
        let maybe_function = {
            let mut call_path_binding = call_path_binding.clone();
            TypeBinding::type_check_with_ident(&mut call_path_binding, ctx.by_ref())
                .flat_map(|unknown_decl| unknown_decl.expect_function(&span))
                .ok(&mut function_probe_warnings, &mut function_probe_errors)
        };

        // Check if this could be an enum
        let mut enum_probe_warnings = vec![];
        let mut enum_probe_errors = vec![];
        let maybe_enum = {
            let call_path_binding = call_path_binding.clone();
            let enum_name = call_path_binding.inner.prefixes[0].clone();
            let variant_name = call_path_binding.inner.suffix.clone();
            let enum_call_path = call_path_binding.inner.rshift();
            let mut call_path_binding = TypeBinding {
                inner: enum_call_path,
                type_arguments: call_path_binding.type_arguments,
                span: call_path_binding.span,
            };
            TypeBinding::type_check_with_ident(&mut call_path_binding, ctx.by_ref())
                .flat_map(|unknown_decl| unknown_decl.expect_enum(&call_path_binding.span()))
                .ok(&mut enum_probe_warnings, &mut enum_probe_errors)
                .map(|enum_decl| (enum_decl, enum_name, variant_name))
        };

        // Check if this could be a constant
        let mut const_probe_warnings = vec![];
        let mut const_probe_errors = vec![];
        let maybe_const = {
            let mut call_path_binding = call_path_binding.clone();
            TypeBinding::type_check_with_ident(&mut call_path_binding, ctx.by_ref())
                .flat_map(|unknown_decl| unknown_decl.expect_const(&call_path_binding.span()))
                .ok(&mut const_probe_warnings, &mut const_probe_errors)
                .map(|const_decl| (const_decl, call_path_binding.span()))
        };

        // compare the results of the checks
        let exp = match (is_module, maybe_function, maybe_enum, maybe_const) {
            (false, None, Some((enum_decl, enum_name, variant_name)), None) => {
                warnings.append(&mut enum_probe_warnings);
                errors.append(&mut enum_probe_errors);
                check!(
                    instantiate_enum(ctx, enum_decl, enum_name, variant_name, args),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (false, Some(func_decl), None, None) => {
                warnings.append(&mut function_probe_warnings);
                errors.append(&mut function_probe_errors);
                check!(
                    instantiate_function_application(ctx, func_decl, call_path_binding.inner, args,),
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
            (false, None, None, Some((const_decl, span))) => {
                warnings.append(&mut const_probe_warnings);
                errors.append(&mut const_probe_errors);
                check!(
                    instantiate_constant_decl(const_decl, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (false, None, None, None) => {
                errors.push(CompileError::SymbolNotFound {
                    name: call_path_binding.inner.suffix,
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

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        mut ctx: TypeCheckContext,
        abi_name: CallPath,
        address: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // TODO use lib-std's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span();
        let address_expr = {
            let ctx = ctx
                .by_ref()
                .with_help_text("An address that is being ABI cast must be of type b256")
                .with_type_annotation(insert_type(TypeInfo::B256));
            check!(
                ty::TyExpression::type_check(ctx, address),
                ty::TyExpression::error(err_span),
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
        let ty::TyAbiDeclaration {
            name,
            interface_surface,
            mut methods,
            span,
            ..
        } = match abi {
            ty::TyDeclaration::AbiDeclaration(decl_id) => {
                check!(
                    CompileResult::from(de_get_abi(decl_id, &span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            ty::TyDeclaration::VariableDeclaration(ref decl) => {
                let ty::TyVariableDeclaration { body: expr, .. } = &**decl;
                let ret_ty = look_up_type_id(expr.return_type);
                let abi_name = match ret_ty {
                    TypeInfo::ContractCaller { abi_name, .. } => abi_name,
                    _ => {
                        errors.push(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: abi.friendly_name(),
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
                            unknown_decl.expect_abi(&span),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    AbiName::Deferred => {
                        return ok(
                            ty::TyExpression {
                                return_type: insert_type(TypeInfo::ContractCaller {
                                    abi_name: AbiName::Deferred,
                                    address: None,
                                }),
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
                    actually_is: a.friendly_name(),
                });
                return err(warnings, errors);
            }
        };

        let return_type = insert_type(TypeInfo::ContractCaller {
            abi_name: AbiName::Known(abi_name.clone()),
            address: Some(Box::new(address_expr.clone())),
        });

        // Retrieve the interface surface for this abi.
        let mut abi_methods = vec![];
        for decl_id in interface_surface.into_iter() {
            let method = check!(
                CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            abi_methods.push(
                de_insert_function(method.to_dummy_func(Mode::ImplAbiFn)).with_parent(decl_id),
            );
        }

        // Retrieve the methods for this abi.
        abi_methods.append(&mut methods);

        // Insert the abi methods into the namespace.
        check!(
            ctx.namespace.insert_trait_implementation(
                abi_name.clone(),
                vec![],
                return_type,
                &abi_methods,
                &span,
                false
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
        if contents.is_empty() {
            let unknown_type = insert_type(TypeInfo::Unknown);
            return ok(
                ty::TyExpression {
                    expression: ty::TyExpressionVariant::Array {
                        contents: Vec::new(),
                    },
                    return_type: insert_type(TypeInfo::Array(unknown_type, 0, unknown_type)),
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
                    .with_type_annotation(insert_type(TypeInfo::Unknown));
                check!(
                    Self::type_check(ctx, expr),
                    ty::TyExpression::error(span),
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
                    contents: typed_contents,
                },
                return_type: insert_type(TypeInfo::Array(elem_type, array_count, elem_type)), // Maybe?
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

        let prefix_te = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, prefix.clone()),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        // If the return type is a static array then create a `ty::TyExpressionVariant::ArrayIndex`.
        if let TypeInfo::Array(elem_type_id, _, _) = look_up_type_id(prefix_te.return_type) {
            let type_info_u64 = TypeInfo::UnsignedInteger(IntegerBits::SixtyFour);
            let ctx = ctx
                .with_help_text("")
                .with_type_annotation(insert_type(type_info_u64));
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
                    return_type: elem_type_id,
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
                            Ident::new_with_override("core", span.clone()),
                            Ident::new_with_override("ops", span.clone()),
                        ],
                        suffix: Ident::new_with_override("index", span.clone()),
                        is_absolute: true,
                    },
                },
                type_arguments: vec![],
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
        let typed_condition = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(insert_type(TypeInfo::Boolean))
                .with_help_text("A while loop's loop condition must be a boolean expression.");
            check!(
                ty::TyExpression::type_check(ctx, condition),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        let unit_ty = insert_type(TypeInfo::Tuple(Vec::new()));
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
        let ctx = ctx
            .with_type_annotation(insert_type(TypeInfo::Unknown))
            .with_help_text("");
        // ensure that the lhs is a variable expression or struct field access
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
                                errors.push(CompileError::AssignmentToNonMutable { name });
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
                        _ => {
                            errors.push(CompileError::InvalidExpressionOnLhs { span });
                            return err(warnings, errors);
                        }
                    }
                };
                let names_vec = names_vec.into_iter().rev().collect::<Vec<_>>();
                let (ty_of_field, _ty_of_parent) = check!(
                    ctx.namespace.find_subfield_type(&base_name, &names_vec),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the reassignment
                let ctx = ctx.with_type_annotation(ty_of_field).with_help_text("");
                let rhs_span = rhs.span();
                let rhs = check!(
                    ty::TyExpression::type_check(ctx, rhs),
                    ty::TyExpression::error(rhs_span),
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
                        return_type: crate::type_system::insert_type(TypeInfo::Tuple(Vec::new())),
                        span,
                    },
                    warnings,
                    errors,
                )
            }
            ReassignmentTarget::StorageField(fields) => {
                let ctx = ctx
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                let reassignment = check!(
                    reassign_storage_subfield(ctx, fields, rhs, span.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors,
                );
                ok(
                    ty::TyExpression {
                        expression: ty::TyExpressionVariant::StorageReassignment(Box::new(
                            reassignment,
                        )),
                        return_type: crate::type_system::insert_type(TypeInfo::Tuple(Vec::new())),
                        span,
                    },
                    warnings,
                    errors,
                )
            }
        }
    }

    fn resolve_numeric_literal(
        lit: Literal,
        span: Span,
        new_type: TypeId,
    ) -> CompileResult<ty::TyExpression> {
        let mut errors = vec![];

        // Parse and resolve a Numeric(span) based on new_type.
        let (val, new_integer_type) = match lit {
            Literal::Numeric(num) => match look_up_type_id(new_type) {
                TypeInfo::UnsignedInteger(n) => match n {
                    IntegerBits::Eight => (
                        num.to_string().parse().map(Literal::U8).map_err(|e| {
                            Literal::handle_parse_int_error(
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
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                },
                TypeInfo::Numeric => (
                    num.to_string().parse().map(Literal::U64).map_err(|e| {
                        Literal::handle_parse_int_error(
                            e,
                            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                            span.clone(),
                        )
                    }),
                    insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
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
                let exp = ty::TyExpression::error(span);
                ok(exp, vec![], errors)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_error::type_error::TypeError;

    fn do_type_check(expr: Expression, type_annotation: TypeId) -> CompileResult<ty::TyExpression> {
        let mut namespace = Namespace::init_root(namespace::Module::default());
        let ctx = TypeCheckContext::from_root(&mut namespace).with_type_annotation(type_annotation);
        ty::TyExpression::type_check(ctx, expr)
    }

    fn do_type_check_for_boolx2(expr: Expression) -> CompileResult<ty::TyExpression> {
        do_type_check(
            expr,
            insert_type(TypeInfo::Array(
                insert_type(TypeInfo::Boolean),
                2,
                insert_type(TypeInfo::Boolean),
            )),
        )
    }

    #[test]
    fn test_array_type_check_non_homogeneous_0() {
        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(vec![
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::U64(0)),
                    span: Span::dummy(),
                },
            ]),
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
            kind: ExpressionKind::Array(vec![
                Expression {
                    kind: ExpressionKind::Literal(Literal::U64(0)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
            ]),
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
            kind: ExpressionKind::Array(vec![
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
            ]),
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
            kind: ExpressionKind::Array(Vec::new()),
            span: Span::dummy(),
        };

        let comp_res = do_type_check(
            expr,
            insert_type(TypeInfo::Array(
                insert_type(TypeInfo::Boolean),
                0,
                insert_type(TypeInfo::Boolean),
            )),
        );
        assert!(comp_res.warnings.is_empty() && comp_res.errors.is_empty());
    }
}

fn disallow_assigning_initialized_registers(asm: &AsmExpression) -> CompileResult<()> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Collect all registers that have initializers in the list of arguments
    let initialized_registers = asm
        .registers
        .iter()
        .filter(|reg| reg.initializer.is_some())
        .map(|reg| VirtualRegister::Virtual(reg.name.to_string()))
        .collect::<FxHashSet<_>>();

    // Collect all asm block instructions in the form of `VirtualOp`s
    let mut opcodes = vec![];
    for op in &asm.body {
        let registers = op
            .op_args
            .iter()
            .map(|reg_name| VirtualRegister::Virtual(reg_name.to_string()))
            .collect::<Vec<VirtualRegister>>();

        opcodes.push(check!(
            crate::asm_lang::Op::parse_opcode(
                &op.op_name,
                &registers,
                &op.immediate,
                op.span.clone(),
            ),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }

    // From the list of `VirtualOp`s, figure out what registers are assigned
    let assigned_registers: FxHashSet<VirtualRegister> =
        opcodes.iter().fold(FxHashSet::default(), |mut acc, op| {
            for u in op.def_registers() {
                acc.insert(u.clone());
            }
            acc
        });

    // Intersect the list of assigned registers with the list of initialized registers
    let initialized_and_assigned_registers = assigned_registers
        .intersection(&initialized_registers)
        .collect::<FxHashSet<_>>();

    // Form all the compile errors given the violating registers above. Obtain span information
    // from the original `asm.registers` vector.
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
