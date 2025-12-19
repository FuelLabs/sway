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
        ty::{
            self, GetDeclIdent, StructAccessInfo, TyCodeBlock, TyDecl, TyExpression,
            TyExpressionVariant, TyImplItem, TyReassignmentTarget, VariableMutability,
        },
        *,
    },
    namespace::{IsExtendingExistingImpl, IsImplInterfaceSurface, IsImplSelf, TraitMap},
    semantic_analysis::{expression::ReachableReport, *},
    transform::to_parsed_lang::type_name_to_type_info_opt,
    type_system::*,
    Engines,
};

use ast_elements::{type_argument::GenericTypeArgument, type_parameter::ConstGenericExpr};
use ast_node::declaration::{insert_supertraits_into_namespace, SupertraitOf};
use either::Either;
use indexmap::IndexMap;
use namespace::{LexicalScope, Module, ResolvedDeclaration};
use rustc_hash::FxHashSet;
use std::collections::{BTreeMap, HashMap, VecDeque};
use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{integer_bits::IntegerBits, u256::U256, BaseIdent, Ident, Named, Span, Spanned};
use symbol_collection_context::SymbolCollectionContext;
use type_resolve::{resolve_call_path, VisibilityCheck};

#[allow(clippy::too_many_arguments)]
impl ty::TyExpression {
    pub(crate) fn std_ops_eq(
        handler: &Handler,
        ctx: TypeCheckContext,
        arguments: Vec<ty::TyExpression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let ctx = ctx.with_type_annotation(type_engine.id_of_bool());
        Self::std_ops(handler, ctx, OpVariant::Equals, arguments, span)
    }

    pub(crate) fn std_ops_neq(
        handler: &Handler,
        ctx: TypeCheckContext,
        arguments: Vec<ty::TyExpression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let ctx = ctx.with_type_annotation(type_engine.id_of_bool());
        Self::std_ops(handler, ctx, OpVariant::NotEquals, arguments, span)
    }

    fn std_ops(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        op_variant: OpVariant,
        arguments: Vec<ty::TyExpression>,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let decl_engine = ctx.engines.de();

        let call_path = CallPath {
            prefixes: vec![
                Ident::new_with_override("std".into(), span.clone()),
                Ident::new_with_override("ops".into(), span.clone()),
            ],
            suffix: Op {
                op_variant,
                span: span.clone(),
            }
            .to_method_name(),
            callpath_type: CallPathType::Full,
        };
        let mut method_name_binding = TypeBinding {
            inner: MethodName::FromTrait {
                call_path: call_path.clone(),
            },
            type_arguments: TypeArgs::Regular(vec![]),
            span: call_path.span(),
        };
        let arguments = VecDeque::from(arguments);
        let arguments_types = arguments.iter().map(|a| a.return_type).collect::<Vec<_>>();
        let (mut decl_ref, _) = resolve_method_name(
            handler,
            ctx.by_ref(),
            &method_name_binding,
            &arguments_types,
        )?;
        decl_ref = monomorphize_method(
            handler,
            ctx,
            decl_ref.clone(),
            method_name_binding.type_arguments.to_vec_mut(),
            BTreeMap::new(),
        )?;
        let method = decl_engine.get_function(&decl_ref);
        // check that the number of parameters and the number of the arguments is the same
        check_function_arguments_arity(handler, arguments.len(), &method, &call_path, false)?;
        let return_type = &method.return_type;
        let args_and_names = method
            .parameters
            .iter()
            .zip(arguments)
            .map(|(param, arg)| (param.name.clone(), arg))
            .collect::<Vec<(_, _)>>();
        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::FunctionApplication {
                call_path,
                arguments: args_and_names,
                fn_ref: decl_ref,
                selector: None,
                type_binding: None,
                method_target: None,
                contract_call_params: IndexMap::new(),
                contract_caller: None,
            },
            return_type: return_type.type_id,
            span,
        };
        Ok(exp)
    }

    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        expr: &Expression,
    ) -> Result<(), ErrorEmitted> {
        match &expr.kind {
            ExpressionKind::Error(_, _) => {}
            ExpressionKind::Literal(_) => {}
            ExpressionKind::AmbiguousPathExpression(expr) => {
                expr.args
                    .iter()
                    .map(|arg_expr| Self::collect(handler, engines, ctx, arg_expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::FunctionApplication(expr) => {
                expr.arguments
                    .iter()
                    .map(|arg_expr| Self::collect(handler, engines, ctx, arg_expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::LazyOperator(expr) => {
                Self::collect(handler, engines, ctx, &expr.lhs)?;
                Self::collect(handler, engines, ctx, &expr.rhs)?;
            }
            ExpressionKind::AmbiguousVariableExpression(_) => {}
            ExpressionKind::Variable(_) => {}
            ExpressionKind::Tuple(exprs) => {
                exprs
                    .iter()
                    .map(|expr| Self::collect(handler, engines, ctx, expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::TupleIndex(expr) => {
                Self::collect(handler, engines, ctx, &expr.prefix)?;
            }
            ExpressionKind::Array(ArrayExpression::Explicit { contents, .. }) => {
                contents
                    .iter()
                    .map(|expr| Self::collect(handler, engines, ctx, expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::Array(ArrayExpression::Repeat { value, length }) => {
                Self::collect(handler, engines, ctx, value)?;
                Self::collect(handler, engines, ctx, length)?;
            }
            ExpressionKind::Struct(expr) => {
                expr.fields
                    .iter()
                    .map(|field| Self::collect(handler, engines, ctx, &field.value))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::CodeBlock(code_block) => {
                TyCodeBlock::collect(handler, engines, ctx, code_block)?
            }
            ExpressionKind::If(if_expr) => {
                Self::collect(handler, engines, ctx, &if_expr.condition)?;
                Self::collect(handler, engines, ctx, &if_expr.then)?;
                if let Some(r#else) = &if_expr.r#else {
                    Self::collect(handler, engines, ctx, r#else)?
                }
            }
            ExpressionKind::Match(expr) => {
                Self::collect(handler, engines, ctx, &expr.value)?;
                expr.branches
                    .iter()
                    .map(|branch| {
                        // create a new namespace for this branch result
                        ctx.scoped(engines, branch.span.clone(), None, |scoped_ctx| {
                            Self::collect(handler, engines, scoped_ctx, &branch.result)
                        })
                        .0
                    })
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::Asm(_) => {}
            ExpressionKind::MethodApplication(expr) => {
                expr.arguments
                    .iter()
                    .map(|expr| Self::collect(handler, engines, ctx, expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::Subfield(expr) => {
                Self::collect(handler, engines, ctx, &expr.prefix)?;
            }
            ExpressionKind::DelineatedPath(expr) => {
                if let Some(expr_args) = &expr.args {
                    expr_args
                        .iter()
                        .map(|arg_expr| Self::collect(handler, engines, ctx, arg_expr))
                        .collect::<Result<Vec<_>, ErrorEmitted>>()?;
                }
            }
            ExpressionKind::AbiCast(expr) => {
                Self::collect(handler, engines, ctx, &expr.address)?;
            }
            ExpressionKind::ArrayIndex(expr) => {
                Self::collect(handler, engines, ctx, &expr.prefix)?;
                Self::collect(handler, engines, ctx, &expr.index)?;
            }
            ExpressionKind::StorageAccess(_) => {}
            ExpressionKind::IntrinsicFunction(expr) => {
                expr.arguments
                    .iter()
                    .map(|arg_expr| Self::collect(handler, engines, ctx, arg_expr))
                    .collect::<Result<Vec<_>, ErrorEmitted>>()?;
            }
            ExpressionKind::WhileLoop(expr) => {
                Self::collect(handler, engines, ctx, &expr.condition)?;
                TyCodeBlock::collect(handler, engines, ctx, &expr.body)?
            }
            ExpressionKind::ForLoop(expr) => {
                Self::collect(handler, engines, ctx, &expr.desugared)?;
            }
            ExpressionKind::Break => {}
            ExpressionKind::Continue => {}
            ExpressionKind::Reassignment(expr) => {
                match &expr.lhs {
                    ReassignmentTarget::ElementAccess(expr) => {
                        Self::collect(handler, engines, ctx, expr)?;
                    }
                    ReassignmentTarget::Deref(expr) => {
                        Self::collect(handler, engines, ctx, expr)?;
                    }
                }
                Self::collect(handler, engines, ctx, &expr.rhs)?;
            }
            ExpressionKind::ImplicitReturn(expr) => Self::collect(handler, engines, ctx, expr)?,
            ExpressionKind::Return(expr) => {
                Self::collect(handler, engines, ctx, expr)?;
            }
            ExpressionKind::Panic(expr) => {
                Self::collect(handler, engines, ctx, expr)?;
            }
            ExpressionKind::Ref(expr) => {
                Self::collect(handler, engines, ctx, &expr.value)?;
            }
            ExpressionKind::Deref(expr) => {
                Self::collect(handler, engines, ctx, expr)?;
            }
        }
        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        expr: &Expression,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        let expr_span = expr.span();
        let span = expr_span.clone();
        let res = match &expr.kind {
            // We've already emitted an error for the `::Error` case.
            ExpressionKind::Error(_, err) => Ok(ty::TyExpression::error(*err, span, engines)),
            ExpressionKind::Literal(lit) => {
                Ok(Self::type_check_literal(engines, lit.clone(), span))
            }
            ExpressionKind::AmbiguousVariableExpression(name) => {
                if matches!(
                    ctx.resolve_symbol(&Handler::default(), name).ok(),
                    Some(ty::TyDecl::EnumVariantDecl { .. })
                ) {
                    let call_path = CallPath {
                        prefixes: vec![],
                        suffix: name.clone(),
                        callpath_type: CallPathType::Ambiguous,
                    };

                    Self::type_check_delineated_path(
                        handler,
                        ctx.by_ref(),
                        TypeBinding {
                            span: call_path.span(),
                            inner: QualifiedCallPath {
                                call_path,
                                qualified_path_root: None,
                            },
                            type_arguments: TypeArgs::Regular(vec![]),
                        },
                        span,
                        None,
                    )
                } else {
                    Self::type_check_variable_expression(handler, ctx.by_ref(), name.clone(), span)
                }
            }
            ExpressionKind::Variable(name) => {
                Self::type_check_variable_expression(handler, ctx.by_ref(), name.clone(), span)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    resolved_call_path_binding: _,
                    ref arguments,
                } = *function_application_expression.clone();
                Self::type_check_function_application(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    arguments,
                    span,
                )
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { op, lhs, rhs }) => {
                let ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_bool());
                Self::type_check_lazy_operator(handler, ctx, op.clone(), lhs, rhs, span)
            }
            ExpressionKind::CodeBlock(contents) => {
                Self::type_check_code_block(handler, ctx.by_ref(), contents, span)
            }
            // TODO: If _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch. Think at which stage to do it because
            // the same optimization should be done on desugared match expressions.
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
            }) => Self::type_check_if_expression(
                handler,
                ctx.by_ref().with_help_text(""),
                *condition.clone(),
                *then.clone(),
                r#else.as_ref().map(|e| *e.clone()),
                span,
            ),
            ExpressionKind::Match(MatchExpression { value, branches }) => {
                Self::type_check_match_expression(
                    handler,
                    ctx.by_ref().with_help_text(""),
                    value,
                    branches.clone(),
                    span,
                )
            }
            ExpressionKind::Asm(asm) => {
                Self::type_check_asm_expression(handler, ctx.by_ref(), *asm.clone(), span)
            }
            ExpressionKind::Struct(struct_expression) => struct_instantiation(
                handler,
                ctx.by_ref(),
                struct_expression.call_path_binding.clone(),
                &struct_expression.fields,
                span,
            ),
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
            }) => Self::type_check_subfield_expression(
                handler,
                ctx.by_ref(),
                prefix,
                span,
                field_to_access.clone(),
            ),
            ExpressionKind::MethodApplication(method_application_expression) => {
                let MethodApplicationExpression {
                    method_name_binding,
                    contract_call_params,
                    ref arguments,
                } = *method_application_expression.clone();
                type_check_method_application(
                    handler,
                    ctx.by_ref(),
                    method_name_binding,
                    contract_call_params,
                    arguments,
                    span,
                )
            }
            ExpressionKind::Tuple(ref fields) => {
                Self::type_check_tuple(handler, ctx.by_ref(), fields, span)
            }
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix,
                index,
                index_span,
            }) => Self::type_check_tuple_index(
                handler,
                ctx.by_ref(),
                *prefix.clone(),
                *index,
                index_span.clone(),
                span,
            ),
            ExpressionKind::AmbiguousPathExpression(e) => {
                let AmbiguousPathExpression {
                    call_path_binding,
                    ref args,
                    qualified_path_root,
                } = *e.clone();
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
                } = *delineated_path_expression.clone();
                Self::type_check_delineated_path(
                    handler,
                    ctx.by_ref(),
                    call_path_binding,
                    span,
                    args.as_deref(),
                )
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                let AbiCastExpression { abi_name, address } = &**abi_cast_expression;
                Self::type_check_abi_cast(handler, ctx.by_ref(), abi_name.clone(), address, span)
            }
            ExpressionKind::Array(ArrayExpression::Explicit { contents, .. }) => {
                Self::type_check_array_explicit(handler, ctx.by_ref(), contents, span)
            }
            ExpressionKind::Array(ArrayExpression::Repeat { value, length }) => {
                Self::type_check_array_repeat(handler, ctx.by_ref(), value, length, span)
            }
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.new_unknown())
                    .with_help_text("");
                Self::type_check_array_index(handler, ctx, prefix, index, span)
            }
            ExpressionKind::StorageAccess(StorageAccessExpression {
                namespace_names,
                field_names,
                storage_keyword_span,
            }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_engine.new_unknown())
                    .with_help_text("");
                Self::type_check_storage_access(
                    handler,
                    ctx,
                    namespace_names,
                    field_names,
                    storage_keyword_span.clone(),
                    &span,
                )
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                kind_binding,
                ref arguments,
                ..
            }) => Self::type_check_intrinsic_function(
                handler,
                ctx.by_ref(),
                kind_binding.clone(),
                arguments,
                span,
            ),
            ExpressionKind::WhileLoop(WhileLoopExpression {
                condition,
                body,
                is_desugared_for_loop,
            }) => Self::type_check_while_loop(
                handler,
                ctx.by_ref(),
                condition,
                body,
                *is_desugared_for_loop,
                span,
            ),
            ExpressionKind::ForLoop(ForLoopExpression { desugared }) => {
                Self::type_check_for_loop(handler, ctx.by_ref(), desugared)
            }
            ExpressionKind::Break => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Break,
                    return_type: type_engine.id_of_never(),
                    span,
                };
                Ok(expr)
            }
            ExpressionKind::Continue => {
                let expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Continue,
                    return_type: type_engine.id_of_never(),
                    span,
                };
                Ok(expr)
            }
            ExpressionKind::Reassignment(ReassignmentExpression { lhs, rhs }) => {
                Self::type_check_reassignment(handler, ctx.by_ref(), lhs.clone(), rhs, span)
            }
            ExpressionKind::ImplicitReturn(expr) => {
                let ctx = ctx
                    .by_ref()
                    .with_help_text("Implicit return must match up with block's type.");
                let expr_span = expr.span();
                let expr = ty::TyExpression::type_check(handler, ctx, expr)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, expr_span, engines));

                let typed_expr = ty::TyExpression {
                    return_type: expr.return_type,
                    expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(expr)),
                    span,
                };
                Ok(typed_expr)
            }
            ExpressionKind::Return(expr) => {
                let function_type_annotation = ctx.function_type_annotation();
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(function_type_annotation)
                    .with_help_text(
                        "Return expression must return the declared function return type.",
                    );
                let expr_span = expr.span();
                let expr = ty::TyExpression::type_check(handler, ctx, expr)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, expr_span, engines));
                let typed_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Return(Box::new(expr)),
                    return_type: type_engine.id_of_never(),
                    span,
                };
                Ok(typed_expr)
            }
            ExpressionKind::Panic(expr) => {
                type_check_panic(handler, ctx.by_ref(), engines, expr, span)
            }
            ExpressionKind::Ref(RefExpression {
                to_mutable_value,
                value,
            }) => Self::type_check_ref(handler, ctx.by_ref(), *to_mutable_value, value, span),
            ExpressionKind::Deref(expr) => {
                Self::type_check_deref(handler, ctx.by_ref(), expr, span)
            }
        };
        let mut typed_expression = res?;

        // if the return type cannot be cast into the annotation type then it is a type error
        ctx.unify_with_type_annotation(handler, typed_expression.return_type, &expr_span);

        // The annotation may result in a cast, which is handled in the type engine.
        typed_expression.return_type = ctx
            .resolve_type(
                handler,
                typed_expression.return_type,
                &expr_span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        // Literals of type Numeric can now be resolved if typed_expression.return_type is
        // an UnsignedInteger or a Numeric
        if let ty::TyExpressionVariant::Literal(lit) = typed_expression.clone().expression {
            if let Literal::Numeric(_) = lit {
                match &*type_engine.get(typed_expression.return_type) {
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

        Ok(typed_expression)
    }

    fn type_check_literal(engines: &Engines, lit: Literal, span: Span) -> ty::TyExpression {
        let type_engine = engines.te();
        let return_type = match &lit {
            Literal::String(_) => type_engine.id_of_string_slice(),
            Literal::Numeric(_) => type_engine.new_numeric(),
            Literal::U8(_) => type_engine.id_of_u8(),
            Literal::U16(_) => type_engine.id_of_u16(),
            Literal::U32(_) => type_engine.id_of_u32(),
            Literal::U64(_) => type_engine.id_of_u64(),
            Literal::U256(_) => type_engine.id_of_u256(),
            Literal::Boolean(_) => type_engine.id_of_bool(),
            Literal::B256(_) => type_engine.id_of_b256(),
            Literal::Binary(_) => type_engine.id_of_raw_slice(),
        };
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(lit),
            return_type,
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

        let exp = match ctx.resolve_symbol(&Handler::default(), &name).ok() {
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
                            CallPath::from(decl_name.clone())
                                .to_fullpath(ctx.engines(), ctx.namespace()),
                        ),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. })) => {
                let const_decl = (*decl_engine.get_constant(&decl_id)).clone();
                let decl_name = const_decl.name().clone();
                ty::TyExpression {
                    return_type: const_decl.return_type,
                    expression: ty::TyExpressionVariant::ConstantExpression {
                        decl: Box::new(const_decl),
                        span: name.span(),
                        call_path: Some(
                            CallPath::from(decl_name).to_fullpath(ctx.engines(), ctx.namespace()),
                        ),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::ConfigurableDecl(ty::ConfigurableDecl { decl_id, .. })) => {
                let decl = (*decl_engine.get_configurable(&decl_id)).clone();
                let decl_name = decl.name().clone();
                ty::TyExpression {
                    return_type: decl.return_type,
                    expression: ty::TyExpressionVariant::ConfigurableExpression {
                        decl: Box::new(decl),
                        span: name.span(),
                        call_path: Some(
                            CallPath::from(decl_name).to_fullpath(ctx.engines(), ctx.namespace()),
                        ),
                    },
                    span,
                }
            }
            Some(ty::TyDecl::ConstGenericDecl(ty::ConstGenericDecl { decl_id })) => {
                let decl = (*decl_engine.get(&decl_id)).clone();
                ty::TyExpression {
                    return_type: decl.return_type,
                    expression: ty::TyExpressionVariant::ConstGenericExpression {
                        decl: Box::new(decl),
                        span: name.span(),
                        call_path: CallPath {
                            prefixes: vec![],
                            suffix: name.clone(),
                            callpath_type: CallPathType::Ambiguous,
                        },
                    },
                    span,
                }
            }
            Some(ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. })) => {
                let decl = decl_engine.get_abi(&decl_id);
                ty::TyExpression {
                    return_type: decl.create_type_id(engines),
                    expression: ty::TyExpressionVariant::AbiName(AbiName::Known(
                        decl.name.clone().into(),
                    )),
                    span,
                }
            }
            Some(a) => {
                let err = handler.emit_err(CompileError::NotAVariable {
                    name: name.clone(),
                    what_it_is: a.friendly_type_name_with_acronym(),
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
        arguments: &[Expression],
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
        lhs: &Expression,
        rhs: &Expression,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let mut ctx = ctx.with_help_text("");
        let engines = ctx.engines();
        let typed_lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs)
            .unwrap_or_else(|err| ty::TyExpression::error(err, lhs.span().clone(), engines));

        let typed_rhs = ty::TyExpression::type_check(handler, ctx.by_ref(), rhs)
            .unwrap_or_else(|err| ty::TyExpression::error(err, rhs.span().clone(), engines));

        let type_annotation = ctx.type_annotation();
        let exp = instantiate_lazy_operator(op, typed_lhs, typed_rhs, type_annotation, span);
        Ok(exp)
    }

    fn type_check_code_block(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        contents: &CodeBlock,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let (typed_block, block_return_type) =
            match ty::TyCodeBlock::type_check(handler, ctx.by_ref(), contents, false) {
                Ok(res) => {
                    let (block_type, _span) = TyCodeBlock::compute_return_type_and_span(&ctx, &res);
                    (res, block_type)
                }
                Err(_err) => (ty::TyCodeBlock::default(), type_engine.id_of_unit()),
            };

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(typed_block),
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
                .with_type_annotation(type_engine.id_of_bool());
            ty::TyExpression::type_check(handler, ctx, &condition)
                .unwrap_or_else(|err| ty::TyExpression::error(err, condition.span(), engines))
        };

        // The final type checking and unification, as well as other semantic requirement like the same type
        // in the `then` and `else` branch are done in the `instantiate_if_expression`.
        // However, if there is an expectation coming from the context via `ctx.type_annotation()` we need
        // to pass that contextual requirement to both branches in order to provide more specific contextual
        // information. E.g., that `Option<u8>` is expected.
        // But at the same time, we do not want to unify during type checking with that contextual information
        // at this stage, because the unification will be done in the `instantiate_if_expression`.
        // In order to pass the contextual information, but not to affect the original type with premature
        // unification, we create two copies of the `ctx.type_annotation()` type and pass them as the
        // expectation to both branches.
        let type_annotation = (*type_engine.get(ctx.type_annotation())).clone();

        let then = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(
                    engines,
                    type_annotation.clone(),
                    then.span().source_id(),
                ));
            ty::TyExpression::type_check(handler, ctx, &then)
                .unwrap_or_else(|err| ty::TyExpression::error(err, then.span(), engines))
        };

        let r#else = r#else.map(|expr| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(
                    engines,
                    type_annotation,
                    expr.span().source_id(),
                ));
            ty::TyExpression::type_check(handler, ctx, &expr)
                .unwrap_or_else(|err| ty::TyExpression::error(err, expr.span(), engines))
        });

        let exp = instantiate_if_expression(handler, ctx, condition, then.clone(), r#else, span)?;

        Ok(exp)
    }

    fn type_check_match_expression(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        value: &Expression,
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
                .with_type_annotation(type_engine.new_unknown());
            ty::TyExpression::type_check(handler, ctx, value)
                .unwrap_or_else(|err| ty::TyExpression::error(err, value.span().clone(), engines))
        };
        let type_id = typed_value.return_type;

        // check to make sure that the type of the value is something that can be matched upon
        type_engine
            .get(type_id)
            .expect_is_supported_in_match_expressions(handler, engines, &typed_value.span)?;

        // type check the match expression and create a ty::TyMatchExpression object
        let (typed_match_expression, typed_scrutinees) = ty::TyMatchExpression::type_check(
            handler,
            ctx.by_ref().with_help_text(""),
            typed_value,
            branches,
            span.clone(),
        )?;

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
                        match_value: value.span().clone(),
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
                value,
                &arms_reachability[..catch_all_arm_position],
            );
        }
        // if there are no interior catch-all arms and there is more than one arm
        else if let Some((last_arm_report, other_arms_reachability)) =
            arms_reachability.split_last()
        {
            // check reachable report for all the arms except the last one
            check_interior_non_catch_all_arms_for_reachability(
                handler,
                engines,
                type_id,
                value,
                other_arms_reachability,
            );

            // for the last one, give a different warning if it is an unreachable catch-all arm
            if !last_arm_report.reachable {
                handler.emit_warn(CompileWarning {
                    span: last_arm_report.scrutinee.span.clone(),
                    warning_content: Warning::MatchExpressionUnreachableArm {
                        match_value: value.span().clone(),
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

        if witness_report.has_witnesses() {
            return Err(
                handler.emit_err(CompileError::MatchExpressionNonExhaustive {
                    missing_patterns: format!("{witness_report}"),
                    span,
                }),
            );
        }

        // desugar the typed match expression to a typed if expression
        let desugared = typed_match_expression.desugar(handler, ctx)?;

        let match_exp = ty::TyExpression {
            span: desugared.span.clone(),
            return_type: desugared.return_type,
            expression: ty::TyExpressionVariant::MatchExp {
                desugared: Box::new(desugared),
                scrutinees: typed_scrutinees,
            },
        };

        return Ok(match_exp);

        /// Returns the position of the first match arm that is an "interior" arm, meaning:
        ///  - arm is a catch-all arm
        ///  - arm is not the last match arm
        ///
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

        if asm.is_empty() {
            handler.emit_warn(CompileWarning {
                span: span.clone(),
                warning_content: Warning::AsmBlockIsEmpty,
            });
        }

        // Various checks that we can catch early to check that the assembly is valid. For now,
        // this includes two checks:
        // 1. Check that no control flow opcodes are used.
        // 2. Check that initialized registers are not reassigned in the `asm` block.
        check_asm_block_validity(handler, &asm, &ctx)?;

        // Take the span of the returns register, or as a fallback, the span of the
        // whole ASM block.
        let asm_returns_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());

        let return_type = ctx
            .resolve_type(
                handler,
                type_engine.insert(
                    engines,
                    asm.return_type.clone(),
                    asm_returns_span.source_id(),
                ),
                &asm_returns_span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

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
                            .with_type_annotation(type_engine.new_unknown());

                        ty::TyExpression::type_check(handler, ctx, &initializer).unwrap_or_else(
                            |err| ty::TyExpression::error(err, initializer.span(), engines),
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
        Ok(exp)
    }

    fn type_check_subfield_expression(
        handler: &Handler,
        ctx: TypeCheckContext,
        prefix: &Expression,
        span: Span,
        field_to_access: Ident,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let mut ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.new_unknown());
        let parent = ty::TyExpression::type_check(handler, ctx.by_ref(), prefix)?;
        let exp = instantiate_struct_field_access(
            handler,
            engines,
            ctx.namespace(),
            parent,
            field_to_access,
            span,
        )?;
        Ok(exp)
    }

    fn type_check_tuple(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        fields: &[Expression],
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let t_arc = type_engine.get(ctx.type_annotation());
        let field_type_opt = match &*t_arc {
            TypeInfo::Tuple(field_type_ids) if field_type_ids.len() == fields.len() => {
                Some(field_type_ids)
            }
            _ => None,
        };
        let mut typed_field_types = Vec::with_capacity(fields.len());
        let mut typed_fields = Vec::with_capacity(fields.len());
        for (i, field) in fields.iter().enumerate() {
            let field_type = field_type_opt
                .as_ref()
                .map(|field_type_ids| field_type_ids[i].clone())
                .unwrap_or_else(|| {
                    let initial_type_id = type_engine.new_unknown();
                    GenericTypeArgument {
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
            typed_field_types.push(GenericTypeArgument {
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
            return_type: ctx.engines.te().insert_tuple(engines, typed_field_types),
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
        namespace_names: &[Ident],
        checkee: &[Ident],
        storage_keyword_span: Span,
        span: &Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        if !ctx
            .namespace()
            .current_module()
            .read(engines, |m| m.root_items().has_storage_declared())
        {
            return Err(handler.emit_err(CompileError::NoDeclaredStorage { span: span.clone() }));
        }

        let storage_fields = ctx.namespace().current_module().read(engines, |m| {
            m.root_items()
                .get_storage_field_descriptors(handler, decl_engine)
        })?;

        // Do all namespace checking here!
        let (storage_access, mut access_type) =
            ctx.namespace().current_module().read(engines, |m| {
                m.root_items().apply_storage_load(
                    handler,
                    ctx.engines,
                    ctx.namespace(),
                    namespace_names,
                    checkee,
                    &storage_fields,
                    storage_keyword_span.clone(),
                )
            })?;

        // The type of a storage access is `std::storage::storage_key::StorageKey`. This is
        // the path to it.
        let storage_key_mod_path = vec![
            Ident::new_with_override("std".into(), span.clone()),
            Ident::new_with_override("storage".into(), span.clone()),
            Ident::new_with_override("storage_key".into(), span.clone()),
        ];
        let storage_key_ident = Ident::new_with_override("StorageKey".into(), span.clone());

        // Search for the struct declaration with the call path above.
        let storage_key_decl = resolve_call_path(
            handler,
            engines,
            ctx.namespace(),
            &storage_key_mod_path,
            &storage_key_ident.into(),
            None,
            VisibilityCheck::No,
        )?;

        let storage_key_struct_decl_id = storage_key_decl
            .expect_typed()
            .to_struct_decl(handler, engines)?;
        let mut storage_key_struct_decl =
            (*decl_engine.get_struct(&storage_key_struct_decl_id)).clone();

        // Set the type arguments to `StorageKey` to the `access_type`, which is represents the
        // type of the data that the `StorageKey` "points" to.
        let mut type_arguments = vec![GenericArgument::Type(GenericTypeArgument {
            initial_type_id: access_type,
            type_id: access_type,
            span: span.clone(),
            call_path_tree: None,
        })];

        // Monomorphize the generic `StorageKey` type given the type argument specified above
        let mut ctx = ctx;
        ctx.monomorphize(
            handler,
            &mut storage_key_struct_decl,
            &mut type_arguments,
            BTreeMap::new(),
            EnforceTypeArguments::Yes,
            span,
        )?;

        // Update `access_type` to be the type of the monomorphized struct after inserting it
        // into the type engine
        let storage_key_struct_decl_ref = decl_engine.insert(
            storage_key_struct_decl,
            decl_engine
                .get_parsed_decl_id(&storage_key_struct_decl_id)
                .as_ref(),
        );
        access_type = type_engine.insert_struct(engines, *storage_key_struct_decl_ref.id());

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
            .with_type_annotation(type_engine.new_unknown());
        let parent = ty::TyExpression::type_check(handler, ctx, &prefix)?;
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
                    callpath_type,
                },
            type_arguments,
            span: path_span,
        }: TypeBinding<CallPath<AmbiguousSuffix>>,
        span: Span,
        args: &[Expression],
        qualified_path_root: Option<QualifiedPathType>,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = ctx.engines;
        let decl_engine = engines.de();

        if let Some(QualifiedPathType { ty, as_trait, .. }) = qualified_path_root.clone() {
            let method_name_binding = if !prefixes.is_empty() || before.is_some() {
                let mut prefixes_and_before = prefixes.clone();
                if let Some(before) = before {
                    prefixes_and_before.push(before.inner);
                }
                let prefixes_and_before_last =
                    prefixes_and_before.remove(prefixes_and_before.len() - 1);

                let qualified_call_path = QualifiedCallPath {
                    call_path: CallPath {
                        prefixes: prefixes_and_before.clone(),
                        suffix: prefixes_and_before_last.clone(),
                        callpath_type,
                    },
                    qualified_path_root: qualified_path_root.map(Box::new),
                };
                let type_info = TypeInfo::Custom {
                    qualified_call_path: qualified_call_path.clone(),
                    type_arguments: None,
                };

                TypeBinding {
                    inner: MethodName::FromType {
                        call_path_binding: TypeBinding {
                            span: qualified_call_path.call_path.span(),
                            type_arguments: type_arguments.clone(),
                            inner: CallPath {
                                prefixes,
                                suffix: (type_info, prefixes_and_before_last),
                                callpath_type,
                            },
                        },
                        method_name: suffix,
                    },
                    type_arguments,
                    span: path_span,
                }
            } else {
                TypeBinding {
                    inner: MethodName::FromQualifiedPathRoot {
                        ty,
                        as_trait,
                        method_name: suffix,
                    },
                    type_arguments,
                    span: path_span,
                }
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
            let call_path = CallPath {
                prefixes,
                suffix,
                callpath_type,
            }
            .to_fullpath(engines, ctx.namespace());

            if matches!(
                ctx.resolve_call_path(&Handler::default(), &call_path,),
                Ok(ty::TyDecl::EnumVariantDecl { .. })
            ) {
                // if it's a singleton it's either an enum variant or a function
                let call_path_binding = TypeBinding {
                    inner: QualifiedCallPath {
                        call_path,
                        qualified_path_root: None,
                    },
                    type_arguments,
                    span: path_span,
                };
                return Self::type_check_delineated_path(
                    handler,
                    ctx,
                    call_path_binding,
                    span,
                    Some(args),
                );
            } else {
                // if it's a singleton it's either an enum variant or a function
                let call_path_binding = TypeBinding {
                    inner: call_path,
                    type_arguments,
                    span: path_span,
                };
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

        let is_module = {
            let h = Handler::default();
            // The path may be relative to the current module,
            // or may be a full path to an external module
            ctx.namespace()
                .current_module()
                .read(engines, |m| m.lookup_submodule(&h, &path).is_ok())
                || (ctx.namespace().module_from_absolute_path(&path).is_some()
                    && ctx.namespace().module_is_external(&path))
        };

        // Not a module? Not a `Enum::Variant` either?
        // Type check as an associated function call instead.
        let is_associated_call = !is_module && {
            let probe_call_path = CallPath {
                prefixes: prefixes.clone(),
                suffix: before.inner.clone(),
                callpath_type,
            };
            ctx.resolve_call_path(&Handler::default(), &probe_call_path)
                .and_then(|decl| decl.to_enum_id(&Handler::default(), ctx.engines()))
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
                qualified_call_path: type_name.clone().into(),
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
                            callpath_type,
                        }
                        .to_fullpath(engines, ctx.namespace()),
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
                inner: QualifiedCallPath {
                    call_path: CallPath {
                        prefixes: path,
                        suffix,
                        callpath_type,
                    },
                    qualified_path_root: None,
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
        unknown_call_path_binding: TypeBinding<QualifiedCallPath>,
        span: Span,
        args: Option<&[Expression]>,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        // The first step is to determine if the call path refers to a module,
        // enum, function or constant.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        let mut is_module = false;
        let mut maybe_function: Option<(DeclRefFunction, _)> = None;
        let mut maybe_enum_variant_with_enum_name: Option<(DeclRefEnum, _, _, _)> = None;
        let mut maybe_enum_variant_without_enum_name: Option<(DeclRefEnum, _, _, _)> = None;

        let module_probe_handler = Handler::default();
        let function_probe_handler = Handler::default();
        let variant_with_enum_probe_handler = Handler::default();
        let variant_without_enum_probe_handler = Handler::default();
        let const_probe_handler = Handler::default();

        if unknown_call_path_binding
            .inner
            .qualified_path_root
            .is_none()
        {
            // Check if this could be a submodule of the current module or an external module
            is_module = {
                let call_path_binding = unknown_call_path_binding.clone();
                let lookup_path = [
                    call_path_binding.inner.call_path.prefixes.clone(),
                    vec![call_path_binding.inner.call_path.suffix.clone()],
                ]
                .concat();
                ctx.namespace().current_module().read(ctx.engines(), |m| {
                    m.lookup_submodule(&module_probe_handler, &lookup_path)
                        .ok()
                        .is_some()
                }) || ctx
                    .namespace()
                    .module_from_absolute_path(&lookup_path)
                    .is_some()
            };

            // Check if this could be a function
            maybe_function = {
                let call_path_binding = unknown_call_path_binding.clone();
                let mut call_path_binding = TypeBinding {
                    inner: call_path_binding.inner.call_path,
                    type_arguments: call_path_binding.type_arguments,
                    span: call_path_binding.span,
                };
                TypeBinding::type_check(
                    &mut call_path_binding,
                    &function_probe_handler,
                    ctx.by_ref(),
                )
                .ok()
                .map(|(fn_ref, _, _)| (fn_ref, call_path_binding))
            };

            // Check if this could be an enum variant preceded by its enum name.
            // For instance, the enum `Option` contains two variants, `None` and `Some`.
            // The full path for `None` would be current_mod_path::Option::None.
            maybe_enum_variant_with_enum_name = {
                let call_path_binding = unknown_call_path_binding.clone();
                let variant_name = call_path_binding.inner.call_path.suffix.clone();
                let enum_call_path = call_path_binding
                    .inner
                    .call_path
                    .to_fullpath(ctx.engines(), ctx.namespace())
                    .rshift();

                if enum_call_path.prefixes.is_empty() {
                    // If the path has no prefixes after the rshift, then the package name is the
                    // new suffix, and so the path is not an enum variant.
                    None
                } else {
                    let mut call_path_binding = TypeBinding {
                        inner: enum_call_path,
                        type_arguments: call_path_binding.type_arguments,
                        span: call_path_binding.span,
                    };
                    TypeBinding::type_check(
                        &mut call_path_binding,
                        &variant_with_enum_probe_handler,
                        ctx.by_ref(),
                    )
                    .ok()
                    .map(|(enum_ref, _, ty_decl)| {
                        (
                            enum_ref,
                            variant_name,
                            call_path_binding,
                            ty_decl.expect("type_check for TyEnumDecl should always return TyDecl"),
                        )
                    })
                }
            };

            // Check if this could be an enum variant without the enum name. This can happen when
            // the variants are imported using a star import
            // For instance, `use Option::*` binds the name `None` in the current module, so the
            // full path would be current_mod_path::None rather than current_mod_path::Option::None.
            maybe_enum_variant_without_enum_name = {
                if maybe_enum_variant_with_enum_name.is_some() {
                    // Corner case. This can happen if the path is just a single identifier
                    // referring to an enum variant name. In this case we use
                    // maybe_enum_variant_with_enum_name
                    None
                } else {
                    let call_path_binding = unknown_call_path_binding.clone();
                    let variant_name = call_path_binding.inner.call_path.suffix.clone();
                    let enum_call_path = call_path_binding
                        .inner
                        .call_path
                        .to_fullpath(ctx.engines(), ctx.namespace());

                    let mut call_path_binding = TypeBinding {
                        inner: enum_call_path,
                        type_arguments: call_path_binding.type_arguments,
                        span: call_path_binding.span,
                    };
                    TypeBinding::type_check(
                        &mut call_path_binding,
                        &variant_without_enum_probe_handler,
                        ctx.by_ref(),
                    )
                    .ok()
                    .map(|(enum_ref, _, ty_decl)| {
                        (
                            enum_ref,
                            variant_name,
                            call_path_binding,
                            ty_decl.expect("type_check for TyEnumDecl should always return TyDecl"),
                        )
                    })
                }
            };
        }

        // Check if this could be a constant
        let maybe_const =
            { Self::probe_const_decl(&unknown_call_path_binding, &mut ctx, &const_probe_handler) };

        // compare the results of the checks
        let exp = match (
            is_module,
            maybe_function,
            maybe_enum_variant_with_enum_name,
            maybe_enum_variant_without_enum_name,
            maybe_const,
        ) {
            (
                false,
                None,
                Some((enum_ref, variant_name, call_path_binding, call_path_decl)),
                None,
                None,
            ) => {
                handler.append(variant_with_enum_probe_handler);
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
            (
                false,
                None,
                None,
                Some((enum_ref, variant_name, call_path_binding, call_path_decl)),
                None,
            ) => {
                handler.append(variant_without_enum_probe_handler);
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
            (false, Some((fn_ref, call_path_binding)), None, None, None) => {
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
            (true, None, None, None, None) => {
                handler.append(module_probe_handler);
                return Err(handler.emit_err(CompileError::ModulePathIsNotAnExpression {
                    module_path: unknown_call_path_binding.inner.call_path.to_string(),
                    span,
                }));
            }
            (false, None, None, None, Some((const_ref, call_path_binding))) => {
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
            (false, None, None, None, None) => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: unknown_call_path_binding.inner.call_path.suffix.clone(),
                    span: unknown_call_path_binding.inner.call_path.suffix.span(),
                }));
            }
            _ => {
                return Err(handler.emit_err(CompileError::AmbiguousPath { span }));
            }
        };
        Ok(exp)
    }

    fn probe_const_decl(
        unknown_call_path_binding: &TypeBinding<QualifiedCallPath>,
        ctx: &mut TypeCheckContext,
        const_probe_handler: &Handler,
    ) -> Option<(DeclRefConstant, TypeBinding<CallPath>)> {
        let mut qualified_call_path_binding = unknown_call_path_binding.clone();

        let mut call_path_binding = TypeBinding {
            inner: qualified_call_path_binding.inner.call_path.clone(),
            type_arguments: qualified_call_path_binding.type_arguments.clone(),
            span: qualified_call_path_binding.span.clone(),
        };

        let type_info_opt = call_path_binding
            .clone()
            .inner
            .prefixes
            .last()
            .map(|type_name| {
                type_name_to_type_info_opt(type_name).unwrap_or(TypeInfo::Custom {
                    qualified_call_path: type_name.clone().into(),
                    type_arguments: None,
                })
            });

        if let Some(type_info) = type_info_opt {
            if TypeInfo::is_self_type(&type_info) {
                call_path_binding.strip_prefixes();
            }
        }

        let const_opt: Option<(DeclRefConstant, _)> =
            TypeBinding::type_check(&mut call_path_binding, &Handler::default(), ctx.by_ref())
                .ok()
                .map(|(const_ref, _, _)| (const_ref, call_path_binding.clone()));
        if const_opt.is_some() {
            return const_opt;
        }

        // If we didn't find a constant, check for the constant inside the impl.
        let const_decl_ref: DeclRefConstant =
            match TypeBinding::<QualifiedCallPath>::type_check_qualified(
                &mut qualified_call_path_binding,
                const_probe_handler,
                ctx,
            ) {
                Ok(val) => val,
                Err(_) => return None,
            };

        Some((const_decl_ref, call_path_binding.clone()))
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        abi_name: CallPath,
        address: &Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // TODO use lib-std's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span().clone();
        let address_expr = {
            let ctx = ctx
                .by_ref()
                .with_help_text("An address that is being ABI cast must be of type b256")
                .with_type_annotation(type_engine.id_of_b256());
            ty::TyExpression::type_check(handler, ctx, address)
                .unwrap_or_else(|err| ty::TyExpression::error(err, err_span, engines))
        };

        // look up the call path and get the declaration it references
        let abi = ctx.resolve_call_path(handler, &abi_name)?;
        let abi_ref = match abi {
            ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id }) => {
                let abi_decl = engines.de().get(&decl_id);
                DeclRef::new(abi_decl.name().clone(), decl_id, abi_decl.span.clone())
            }
            ty::TyDecl::VariableDecl(ref decl) => {
                let ty::TyVariableDecl { body: expr, .. } = &**decl;
                let ret_ty = type_engine.get(expr.return_type);
                let abi_name = match &*ret_ty {
                    TypeInfo::ContractCaller { abi_name, .. } => abi_name,
                    _ => {
                        return Err(handler.emit_err(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: abi.friendly_type_name_with_acronym(),
                        }));
                    }
                };
                match abi_name {
                    // look up the call path and get the declaration it references
                    AbiName::Known(abi_name) => {
                        let unknown_decl = ctx.resolve_call_path(handler, abi_name)?;
                        unknown_decl.to_abi_ref(handler, engines)?
                    }
                    AbiName::Deferred => {
                        return Ok(ty::TyExpression {
                            return_type: type_engine.new_contract_caller(
                                engines,
                                AbiName::Deferred,
                                None,
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
                    actually_is: a.friendly_type_name_with_acronym(),
                }));
            }
        };
        let abi_decl = decl_engine.get_abi(abi_ref.id());
        let ty::TyAbiDecl {
            interface_surface,
            items,
            supertraits,
            span,
            ..
        } = &*abi_decl;

        let return_type = type_engine.new_contract_caller(
            engines,
            AbiName::Known(abi_name.clone()),
            Some(Box::new(address_expr.clone())),
        );

        // Retrieve the interface surface for this abi.
        let mut abi_items = vec![];

        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let method = decl_engine.get_trait_fn(decl_ref);
                    abi_items.push(TyImplItem::Fn(
                        decl_engine
                            .insert(
                                method.to_dummy_func(
                                    AbiMode::ImplAbiFn(
                                        abi_name.suffix.clone(),
                                        Some(*abi_ref.id()),
                                    ),
                                    Some(return_type),
                                ),
                                None,
                            )
                            .with_parent(decl_engine, (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(decl_ref);
                    abi_items.push(TyImplItem::Constant(decl_engine.insert_arc(
                        const_decl,
                        decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                    )));
                }
                ty::TyTraitInterfaceItem::Type(decl_ref) => {
                    let type_decl = decl_engine.get_type(decl_ref);
                    abi_items.push(TyImplItem::Type(decl_engine.insert_arc(
                        type_decl,
                        decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                    )));
                }
            }
        }

        // Retrieve the items for this abi.
        abi_items.append(&mut items.to_vec());

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this abi cast.
        insert_supertraits_into_namespace(
            handler,
            ctx.by_ref(),
            return_type,
            supertraits,
            &SupertraitOf::Abi(span.clone()),
        )?;

        // Insert the abi methods into the namespace.
        ctx.insert_trait_implementation(
            handler,
            abi_name.clone(),
            vec![],
            return_type,
            vec![],
            &abi_items,
            span,
            Some(span.clone()),
            IsImplSelf::No,
            IsExtendingExistingImpl::No,
            IsImplInterfaceSurface::No,
        )?;

        let exp = ty::TyExpression {
            expression: ty::TyExpressionVariant::AbiCast {
                abi_name,
                address: Box::new(address_expr),
                span: span.clone(),
            },
            return_type,
            span: span.clone(),
        };
        Ok(exp)
    }

    fn type_check_array_repeat(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        value: &Expression,
        length: &Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // capture the expected array element type from context,
        // otherwise, fallback to Unknown.
        let elem_type = match &*ctx.engines().te().get(ctx.type_annotation()) {
            TypeInfo::Array(element_type, _) => {
                let element_type = (*ctx.engines().te().get(element_type.type_id)).clone();
                if matches!(element_type, TypeInfo::Never) {
                    TypeInfo::Unknown //Even if array element type is Never other elements may not be of type Never.
                } else {
                    element_type
                }
            }
            _ => TypeInfo::Unknown,
        };
        let elem_type = type_engine.insert(engines, elem_type, None);
        let elem_type_arg = GenericTypeArgument {
            type_id: elem_type,
            initial_type_id: elem_type,
            span: span.clone(),
            call_path_tree: None,
        };

        let value_ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(elem_type);
        let value = Self::type_check(handler, value_ctx, value)
            .unwrap_or_else(|err| ty::TyExpression::error(err, span.clone(), engines));

        let length_ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.id_of_u64());
        let length_expr = Self::type_check(handler, length_ctx, length)
            .unwrap_or_else(|err| ty::TyExpression::error(err, span.clone(), engines));
        let length = Length(ConstGenericExpr::from_ty_expression(handler, &length_expr)?);

        let return_type = type_engine.insert_array(engines, elem_type_arg, length);
        Ok(ty::TyExpression {
            expression: ty::TyExpressionVariant::ArrayRepeat {
                elem_type,
                value: Box::new(value),
                length: Box::new(length_expr),
            },
            return_type,
            span,
        })
    }

    fn type_check_array_explicit(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        contents: &[Expression],
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        if contents.is_empty() {
            let elem_type = type_engine.new_unknown();
            return Ok(ty::TyExpression {
                expression: ty::TyExpressionVariant::ArrayExplicit {
                    elem_type,
                    contents: Vec::new(),
                },
                return_type: type_engine.insert_array_without_annotations(engines, elem_type, 0),
                span,
            });
        };

        // capture the expected array element type from context,
        // otherwise, fallback to Unknown.
        let initial_type = match &*ctx.engines().te().get(ctx.type_annotation()) {
            TypeInfo::Array(element_type, _) => {
                let element_type = (*ctx.engines().te().get(element_type.type_id)).clone();
                if matches!(element_type, TypeInfo::Never) {
                    TypeInfo::Unknown //Even if array element type is Never other elements may not be of type Never.
                } else {
                    element_type
                }
            }
            _ => TypeInfo::Unknown,
        };

        let typed_contents: Vec<ty::TyExpression> = contents
            .iter()
            .map(|expr| {
                let span = expr.span();
                let ctx = ctx
                    .by_ref()
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(engines, initial_type.clone(), None));

                // type_check_analyze unification will give the final error
                let type_check_handler = Handler::default();
                let result = Self::type_check(&type_check_handler, ctx, expr)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, span, engines));

                if let TypeInfo::ErrorRecovery(_) = &*engines.te().get(result.return_type) {
                    handler.append(type_check_handler);
                }

                result
            })
            .collect();

        // if the element type is still unknown, and all elements are Never,
        // fallback to unit
        let initial_type = if matches!(initial_type, TypeInfo::Unknown) {
            let is_all_elements_never = typed_contents
                .iter()
                .all(|expr| matches!(&*engines.te().get(expr.return_type), TypeInfo::Never));
            if is_all_elements_never {
                TypeInfo::Tuple(vec![])
            } else {
                initial_type
            }
        } else {
            initial_type
        };

        let elem_type = type_engine.insert(engines, initial_type.clone(), None);
        let length = typed_contents.len();
        let expr = ty::TyExpression {
            expression: ty::TyExpressionVariant::ArrayExplicit {
                elem_type,
                contents: typed_contents,
            },
            return_type: type_engine.insert_array_without_annotations(engines, elem_type, length),
            span,
        };

        // type_check_analyze unification will give the final error
        let handler = Handler::default();
        expr.as_array_unify_elements(&handler, ctx.engines);

        Ok(expr)
    }

    fn type_check_array_index(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        prefix: &Expression,
        index: &Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let mut current_prefix_te = Box::new({
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.new_unknown());

            ty::TyExpression::type_check(handler, ctx, prefix)?
        });

        let mut current_type = type_engine.get_unaliased(current_prefix_te.return_type);

        let prefix_type_id = current_prefix_te.return_type;
        let prefix_span = current_prefix_te.span.clone();

        // Create the prefix part of the final array index expression.
        // This might be an expression that directly evaluates to an array type,
        // or an arbitrary number of dereferencing expressions where the last one
        // dereference to an array type.
        //
        // We will either hit an array at the end or return an error, so the
        // loop cannot be endless.
        while !current_type.is_array() {
            match &*current_type {
                TypeInfo::Ref {
                    referenced_type, ..
                } => {
                    let referenced_type_id = referenced_type.type_id;

                    current_prefix_te = Box::new(ty::TyExpression {
                        expression: ty::TyExpressionVariant::Deref(current_prefix_te),
                        return_type: referenced_type_id,
                        span: prefix_span.clone(),
                    });

                    current_type = type_engine.get_unaliased(referenced_type_id);
                }
                TypeInfo::ErrorRecovery(err) => return Err(*err),
                _ => {
                    return Err(handler.emit_err(CompileError::NotIndexable {
                        actually: engines.help_out(prefix_type_id).to_string(),
                        span: prefix_span,
                    }))
                }
            };
        }

        let TypeInfo::Array(array_type_argument, _) = &*current_type else {
            panic!("The current type must be an array.");
        };

        let index_te = {
            let ctx = ctx
                .with_help_text("Array index must be of type \"u64\".")
                .with_type_annotation(type_engine.id_of_u64());

            ty::TyExpression::type_check(handler, ctx, index)?
        };

        Ok(ty::TyExpression {
            expression: ty::TyExpressionVariant::ArrayIndex {
                prefix: current_prefix_te,
                index: Box::new(index_te),
            },
            return_type: array_type_argument.type_id,
            span,
        })
    }

    fn type_check_intrinsic_function(
        handler: &Handler,
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: &[Expression],
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
        condition: &Expression,
        body: &CodeBlock,
        is_desugared_for_loop: bool,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let typed_condition = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.id_of_bool())
                .with_help_text("A while loop's loop condition must be a boolean expression.");
            ty::TyExpression::type_check(handler, ctx, condition)?
        };

        let unit_ty = type_engine.id_of_unit();
        let mut ctx = ctx
            .with_type_annotation(unit_ty)
            .with_help_text(if is_desugared_for_loop {
                "A for loop's loop body cannot implicitly return a value. Try \
                 assigning it to a mutable variable declared outside of the loop \
                 instead."
            } else {
                "A while loop's loop body cannot implicitly return a value. Try \
                 assigning it to a mutable variable declared outside of the loop \
                 instead."
            });
        let typed_body = ty::TyCodeBlock::type_check(handler, ctx.by_ref(), body, false)?;

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

    fn type_check_for_loop(
        handler: &Handler,
        ctx: TypeCheckContext,
        desugared: &Expression,
    ) -> Result<Self, ErrorEmitted> {
        Self::type_check(handler, ctx, desugared)
    }

    fn type_check_reassignment(
        handler: &Handler,
        ctx: TypeCheckContext,
        lhs: ReassignmentTarget,
        rhs: &Expression,
        span: Span,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let mut ctx = ctx
            .with_type_annotation(type_engine.new_unknown())
            .with_help_text("");

        let (lhs, expected_rhs_type) = match lhs {
            ReassignmentTarget::Deref(dereference_exp) => {
                let internal_compiler_error = || {
                    Result::<Self, _>::Err(handler.emit_err(CompileError::Internal(
                        "Left-hand side of the reassignment must be dereferencing.",
                        dereference_exp.span.clone(),
                    )))
                };

                let Expression {
                    kind: ExpressionKind::Deref(reference_exp),
                    ..
                } = &*dereference_exp
                else {
                    return internal_compiler_error();
                };

                let reference_exp_span = reference_exp.span();
                let deref_exp = Self::type_check_deref(
                    handler,
                    ctx.by_ref(),
                    reference_exp,
                    reference_exp_span.clone(),
                )?;

                let TyExpression {
                    expression: TyExpressionVariant::Deref(reference_exp),
                    ..
                } = &deref_exp
                else {
                    return internal_compiler_error();
                };

                let TypeInfo::Ref {
                    to_mutable_value, ..
                } = *type_engine.get(reference_exp.return_type)
                else {
                    return internal_compiler_error();
                };

                if !to_mutable_value {
                    let (decl_reference_name, decl_reference_rhs, decl_reference_type) =
                        match &reference_exp.expression {
                            TyExpressionVariant::VariableExpression { name, .. } => {
                                let var_decl = ctx.resolve_symbol(handler, name)?;

                                let TyDecl::VariableDecl(var_decl) = var_decl else {
                                    return Err(handler.emit_err(CompileError::Internal(
                                        "Dereferenced expression must be a variable.",
                                        reference_exp_span,
                                    )));
                                };

                                let reference_type = engines
                                    .help_out(
                                        type_engine.get_unaliased_type_id(var_decl.return_type),
                                    )
                                    .to_string();

                                (
                                    Some(var_decl.name),
                                    Some(var_decl.body.span),
                                    reference_type,
                                )
                            }
                            _ => (
                                None,
                                None,
                                engines
                                    .help_out(
                                        type_engine
                                            .get_unaliased_type_id(reference_exp.return_type),
                                    )
                                    .to_string(),
                            ),
                        };

                    return Err(
                        handler.emit_err(CompileError::AssignmentViaNonMutableReference {
                            decl_reference_name,
                            decl_reference_rhs,
                            decl_reference_type,
                            span: reference_exp_span,
                        }),
                    );
                }

                let expected_rhs_type = deref_exp.return_type;
                (
                    TyReassignmentTarget::DerefAccess {
                        exp: Box::new(deref_exp),
                        indices: vec![],
                    },
                    expected_rhs_type,
                )
            }
            ReassignmentTarget::ElementAccess(path) => {
                let lhs_span = path.span.clone();
                let mut expr = path;
                let mut indices = Vec::new();

                // This variaable is used after the loop.
                #[allow(unused_assignments)]
                let mut base_deref_expr = None;
                // Loop through the LHS "backwards" starting from the outermost expression
                // (the whole LHS) and moving towards the first identifier that must
                // be a mutable variable.
                let (base_name, base_type) = loop {
                    base_deref_expr = None;
                    match expr.kind {
                        ExpressionKind::Variable(name) => {
                            // check that the reassigned name exists
                            let unknown_decl = ctx.resolve_symbol(handler, &name)?;

                            match unknown_decl {
                                TyDecl::VariableDecl(variable_decl) => {
                                    if !variable_decl.mutability.is_mutable() {
                                        return Err(handler.emit_err(
                                            CompileError::AssignmentToNonMutableVariable {
                                                decl_name: variable_decl.name.clone(),
                                                lhs_span,
                                            },
                                        ));
                                    }

                                    break (name, variable_decl.return_type);
                                }
                                TyDecl::ConstantDecl(constant_decl) => {
                                    let constant_decl =
                                        engines.de().get_constant(&constant_decl.decl_id);
                                    return Err(handler.emit_err(
                                        CompileError::AssignmentToConstantOrConfigurable {
                                            decl_name: constant_decl.name().clone(),
                                            is_configurable: false,
                                            lhs_span,
                                        },
                                    ));
                                }
                                TyDecl::ConfigurableDecl(decl) => {
                                    let decl = engines.de().get_configurable(&decl.decl_id);
                                    return Err(handler.emit_err(
                                        CompileError::AssignmentToConstantOrConfigurable {
                                            decl_name: decl.name().clone(),
                                            is_configurable: true,
                                            lhs_span,
                                        },
                                    ));
                                }
                                decl => {
                                    return Err(handler.emit_err(
                                        CompileError::DeclAssignmentTargetCannotBeAssignedTo {
                                            decl_name: decl.get_decl_ident(engines),
                                            decl_friendly_type_name: decl
                                                .friendly_type_name_with_acronym(),
                                            lhs_span,
                                        },
                                    ));
                                }
                            }
                        }
                        ExpressionKind::Subfield(SubfieldExpression {
                            prefix,
                            field_to_access: idx_name,
                            ..
                        }) => {
                            let prefix_expr = ty::TyExpression::type_check(
                                &Handler::default(),
                                ctx.by_ref(),
                                prefix.as_ref(),
                            )
                            .unwrap_or_else(|err| {
                                ty::TyExpression::error(err, span.clone(), engines)
                            });

                            let field_to_access = match &*engines.te().get(prefix_expr.return_type)
                            {
                                TypeInfo::Struct(decl_ref) => {
                                    let struct_decl = engines.de().get_struct(decl_ref);
                                    struct_decl.find_field(&idx_name).cloned()
                                }
                                _ => None,
                            };

                            indices.push(ty::ProjectionKind::StructField {
                                name: idx_name,
                                field_to_access: field_to_access.map(Box::new),
                            });
                            expr = prefix;
                        }
                        ExpressionKind::TupleIndex(TupleIndexExpression {
                            prefix,
                            index,
                            index_span,
                            ..
                        }) => {
                            indices.push(ty::ProjectionKind::TupleField { index, index_span });
                            expr = prefix;
                        }
                        ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                            let ctx = ctx
                                .by_ref()
                                .with_help_text("Array index must be of type \"u64\".")
                                .with_type_annotation(type_engine.id_of_u64());
                            let typed_index =
                                ty::TyExpression::type_check(handler, ctx, index.as_ref())
                                    .unwrap_or_else(|err| {
                                        ty::TyExpression::error(err, span.clone(), engines)
                                    });
                            indices.push(ty::ProjectionKind::ArrayIndex {
                                index: Box::new(typed_index),
                                index_span: index.span(),
                            });
                            expr = prefix;
                        }
                        ExpressionKind::Deref(reference_exp) => {
                            let reference_exp_span = reference_exp.span();
                            let deref_expr = Self::type_check_deref(
                                handler,
                                ctx.by_ref(),
                                &reference_exp,
                                reference_exp_span.clone(),
                            )?;
                            base_deref_expr = Some(deref_expr.clone());

                            break (BaseIdent::new(deref_expr.span), deref_expr.return_type);
                        }
                        _ => {
                            return Err(
                                handler.emit_err(CompileError::InvalidExpressionOnLhs { span })
                            );
                        }
                    }
                };

                let indices = indices.into_iter().rev().collect::<Vec<_>>();
                let (ty_of_field, _ty_of_parent) =
                    ctx.namespace().current_module().read(engines, |m| {
                        Self::find_subfield_type(
                            m,
                            handler,
                            ctx.engines(),
                            ctx.namespace(),
                            &base_name,
                            &base_deref_expr,
                            &indices,
                        )
                    })?;

                if let Some(base_deref_expr) = base_deref_expr {
                    (
                        TyReassignmentTarget::DerefAccess {
                            exp: Box::new(base_deref_expr),
                            indices,
                        },
                        ty_of_field,
                    )
                } else {
                    (
                        TyReassignmentTarget::ElementAccess {
                            base_name,
                            base_type,
                            indices,
                        },
                        ty_of_field,
                    )
                }
            }
        };

        let ctx = ctx
            .with_type_annotation(expected_rhs_type)
            .with_help_text("");
        let rhs_span = rhs.span();
        let rhs = ty::TyExpression::type_check(handler, ctx, rhs)
            .unwrap_or_else(|err| ty::TyExpression::error(err, rhs_span, engines));

        Ok(ty::TyExpression {
            expression: ty::TyExpressionVariant::Reassignment(Box::new(ty::TyReassignment {
                lhs,
                rhs,
            })),
            return_type: type_engine.id_of_unit(),
            span,
        })
    }

    pub fn find_subfield_type(
        module: &Module,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        base_name: &Ident,
        base_deref_expr: &Option<TyExpression>,
        projections: &[ty::ProjectionKind],
    ) -> Result<(TypeId, TypeId), ErrorEmitted> {
        let ret = module.walk_scope_chain_early_return(|lexical_scope| {
            Self::find_subfield_type_helper(
                lexical_scope,
                handler,
                engines,
                namespace,
                base_name,
                base_deref_expr,
                projections,
            )
        })?;

        if let Some(ret) = ret {
            Ok(ret)
        } else {
            // Symbol not found
            Err(handler.emit_err(CompileError::UnknownVariable {
                var_name: base_name.clone(),
                span: base_name.span(),
            }))
        }
    }

    /// Returns a tuple where the first element is the [TypeId] of the actual expression, and
    /// the second is the [TypeId] of its parent.
    fn find_subfield_type_helper(
        lexical_scope: &LexicalScope,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        base_name: &Ident,
        base_deref_expr: &Option<TyExpression>,
        projections: &[ty::ProjectionKind],
    ) -> Result<Option<(TypeId, TypeId)>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();

        let mut symbol = if let Some(ref base_deref_expr) = base_deref_expr {
            base_deref_expr.return_type
        } else {
            let symbol = match lexical_scope.items.symbols.get(base_name).cloned() {
                Some(s) => s,
                None => {
                    return Ok(None);
                }
            };
            match symbol {
                ResolvedDeclaration::Parsed(_) => unreachable!(),
                ResolvedDeclaration::Typed(ty_decl) => ty_decl.return_type(handler, engines)?,
            }
        };

        let mut symbol_span = base_name.span();
        let mut parent_rover = symbol;
        let mut full_span_for_error = base_name.span();
        for projection in projections {
            let resolved_type = match type_engine.to_typeinfo(symbol, &symbol_span) {
                Ok(resolved_type) => resolved_type,
                Err(error) => {
                    return Err(handler.emit_err(CompileError::TypeError(error)));
                }
            };
            match (resolved_type, projection) {
                (
                    TypeInfo::Struct(decl_ref),
                    ty::ProjectionKind::StructField {
                        name: field_name,
                        field_to_access: _,
                    },
                ) => {
                    let struct_decl = decl_engine.get_struct(&decl_ref);
                    let (struct_can_be_changed, is_public_struct_access) =
                        StructAccessInfo::get_info(engines, &struct_decl, namespace).into();

                    let field_type_id = match struct_decl.find_field(field_name) {
                        Some(struct_field) => {
                            if is_public_struct_access && struct_field.is_private() {
                                return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                                    field_name: field_name.into(),
                                    struct_name: struct_decl.call_path.suffix.clone(),
                                    field_decl_span: struct_field.name.span(),
                                    struct_can_be_changed,
                                    usage_context: StructFieldUsageContext::StructFieldAccess,
                                }));
                            }

                            struct_field.type_argument.type_id
                        }
                        None => {
                            return Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                                field_name: field_name.into(),
                                available_fields: struct_decl
                                    .accessible_fields_names(is_public_struct_access),
                                is_public_struct_access,
                                struct_name: struct_decl.call_path.suffix.clone(),
                                struct_decl_span: struct_decl.span(),
                                struct_is_empty: struct_decl.is_empty(),
                                usage_context: StructFieldUsageContext::StructFieldAccess,
                            }));
                        }
                    };
                    parent_rover = symbol;
                    symbol = field_type_id;
                    symbol_span = field_name.span().clone();
                    full_span_for_error = Span::join(full_span_for_error, &field_name.span());
                }
                (TypeInfo::Tuple(fields), ty::ProjectionKind::TupleField { index, index_span }) => {
                    let field_type_opt = {
                        fields
                            .get(*index)
                            .map(|GenericTypeArgument { type_id, .. }| type_id)
                    };
                    let field_type = match field_type_opt {
                        Some(field_type) => field_type,
                        None => {
                            return Err(handler.emit_err(CompileError::TupleIndexOutOfBounds {
                                index: *index,
                                count: fields.len(),
                                tuple_type: engines.help_out(symbol).to_string(),
                                span: index_span.clone(),
                                prefix_span: full_span_for_error.clone(),
                            }));
                        }
                    };
                    parent_rover = symbol;
                    symbol = *field_type;
                    symbol_span = index_span.clone();
                    full_span_for_error = Span::join(full_span_for_error, index_span);
                }
                (mut actually, ty::ProjectionKind::ArrayIndex { index, index_span }) => {
                    if let TypeInfo::Ref {
                        referenced_type, ..
                    } = actually
                    {
                        actually = (*engines.te().get(referenced_type.type_id)).clone();
                    }
                    match actually {
                        TypeInfo::Array(elem_ty, array_length)
                            if array_length.expr().as_literal_val().is_some() =>
                        {
                            parent_rover = symbol;
                            symbol = elem_ty.type_id;
                            symbol_span = index_span.clone();

                            if let Some(index_literal) = index
                                .expression
                                .as_literal()
                                .and_then(|x| x.cast_value_to_u64())
                            {
                                // SAFETY: safe by the guard above
                                let array_length = array_length
                                    .expr()
                                    .as_literal_val()
                                    .expect("unexpected non literal array length")
                                    as u64;
                                if index_literal >= array_length {
                                    return Err(handler.emit_err(CompileError::ArrayOutOfBounds {
                                        index: index_literal,
                                        count: array_length,
                                        span: index.span.clone(),
                                    }));
                                }
                            }

                            // `index_span` does not contain the enclosing square brackets.
                            // Which means, if this array index access is the last one before the
                            // erroneous expression, the `full_span_for_error` will be missing the
                            // closing `]`. We can live with this small glitch so far. To fix it,
                            // we would need to bring the full span of the index all the way from
                            // the parsing stage. An effort that doesn't pay off at the moment.
                            // TODO: Include the closing square bracket into the error span.
                            // https://github.com/FuelLabs/sway/issues/7023
                            full_span_for_error = Span::join(full_span_for_error, index_span);
                        }
                        _ => {
                            return Err(handler.emit_err(CompileError::NotIndexable {
                                actually: engines.help_out(actually).to_string(),
                                span: full_span_for_error,
                            }));
                        }
                    }
                }
                (
                    actually,
                    ty::ProjectionKind::StructField {
                        name,
                        field_to_access: _,
                    },
                ) => {
                    return Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                        actually: engines.help_out(actually).to_string(),
                        storage_variable: None,
                        field_name: name.into(),
                        span: full_span_for_error,
                    }));
                }
                (
                    actually,
                    ty::ProjectionKind::TupleField {
                        index, index_span, ..
                    },
                ) => {
                    return Err(
                        handler.emit_err(CompileError::TupleElementAccessOnNonTuple {
                            actually: engines.help_out(actually).to_string(),
                            span: full_span_for_error,
                            index: *index,
                            index_span: index_span.clone(),
                        }),
                    );
                }
            }
        }
        Ok(Some((symbol, parent_rover)))
    }

    fn type_check_ref(
        handler: &Handler,
        mut ctx: TypeCheckContext<'_>,
        to_mutable_value: bool,
        value: &Expression,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = ctx.engines();
        let type_engine = ctx.engines().te();

        // Get the type annotation.
        // If the type provided by the context is a reference, we expect the type of the `value`
        // to be the referenced type of that reference.
        // Otherwise, we have a wrong expectation coming from the context. So we will pass a new
        // `TypeInfo::Unknown` as the annotation, to allow the `value` to be evaluated
        // without any expectations. That value will at the end not unify with the type
        // annotation coming from the context and a type-mismatch error will be emitted.
        let type_annotation = match &*type_engine.get(ctx.type_annotation()) {
            TypeInfo::Ref {
                referenced_type, ..
            } => referenced_type.type_id,
            _ => type_engine.new_unknown(),
        };

        let ctx = ctx
            .by_ref()
            .with_type_annotation(type_annotation)
            .with_help_text("");

        let expr_span = value.span().clone();
        let expr = ty::TyExpression::type_check(handler, ctx, value)?;

        if to_mutable_value {
            if let Some(value) = Self::check_ref_mutability_mismatch(
                &expr.expression,
                handler,
                expr_span,
                span.clone(),
            ) {
                return value;
            }
        };

        let expr_return_type = expr.return_type;
        let typed_expr = ty::TyExpression {
            expression: ty::TyExpressionVariant::Ref(Box::new(expr)),
            return_type: type_engine.insert_ref_without_annotations(
                engines,
                to_mutable_value,
                expr_return_type,
            ),
            span,
        };

        Ok(typed_expr)
    }

    fn check_ref_mutability_mismatch(
        expr: &TyExpressionVariant,
        handler: &Handler,
        expr_span: Span,
        ref_span: Span,
    ) -> Option<Result<TyExpression, ErrorEmitted>> {
        match expr {
            ty::TyExpressionVariant::ConstantExpression { .. } => {
                return Some(Err(handler.emit_err(
                    CompileError::RefMutCannotReferenceConstant {
                        constant: expr_span.str(),
                        span: ref_span,
                    },
                )))
            }
            ty::TyExpressionVariant::VariableExpression {
                name: decl_name,
                mutability: VariableMutability::Immutable,
                ..
            } => {
                return Some(Err(handler.emit_err(
                    CompileError::RefMutCannotReferenceImmutableVariable {
                        decl_name: decl_name.clone(),
                        span: ref_span,
                    },
                )))
            }
            ty::TyExpressionVariant::StructFieldAccess { ref prefix, .. } => {
                return Self::check_ref_mutability_mismatch(
                    &prefix.expression,
                    handler,
                    expr_span,
                    ref_span,
                )
            }
            ty::TyExpressionVariant::TupleElemAccess { ref prefix, .. } => {
                return Self::check_ref_mutability_mismatch(
                    &prefix.expression,
                    handler,
                    expr_span,
                    ref_span,
                )
            }
            _ => (),
        }
        None
    }

    fn type_check_deref(
        handler: &Handler,
        mut ctx: TypeCheckContext<'_>,
        expr: &Expression,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = ctx.engines();
        let type_engine = ctx.engines().te();

        // Get the type annotation.
        // If there is an expectation coming from the context, i.e., if the context
        // type is not `TypeInfo::Unknown`, we expect the type of the `expr` to be a
        // reference to the expected type.
        // Otherwise, we pass a new `TypeInfo::Unknown` as the annotation, to allow the `expr`
        // to be evaluated without any expectations.
        // Since `&mut T` coerces into `&T` we always go with a lesser expectation, `&T`.
        // Thus, `to_mutable_vale` is set to false.
        let type_annotation = match &*type_engine.get(ctx.type_annotation()) {
            TypeInfo::Unknown => type_engine.new_unknown(),
            _ => type_engine.insert_ref_without_annotations(engines, false, ctx.type_annotation()),
        };

        let deref_ctx = ctx
            .by_ref()
            .with_type_annotation(type_annotation)
            .with_help_text("");

        let expr_span = expr.span().clone();
        let expr = ty::TyExpression::type_check(handler, deref_ctx, expr)
            .unwrap_or_else(|err| ty::TyExpression::error(err, expr_span.clone(), engines));

        let expr_type = type_engine.get(expr.return_type);
        let return_type = match *expr_type {
            TypeInfo::ErrorRecovery(_) => Ok(expr.return_type), // Just forward the error return type.
            TypeInfo::Ref {
                referenced_type: ref exp,
                ..
            } => Ok(exp.type_id), // Get the referenced type.
            _ => Err(
                handler.emit_err(CompileError::ExpressionCannotBeDereferenced {
                    expression_type: engines.help_out(expr.return_type).to_string(),
                    span: expr_span,
                }),
            ),
        }?;

        let typed_expr = ty::TyExpression {
            expression: ty::TyExpressionVariant::Deref(Box::new(expr)),
            return_type,
            span,
        };

        Ok(typed_expr)
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
            Literal::Numeric(num) => match &*type_engine.get(new_type) {
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
                    type_engine.new_numeric(),
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

fn type_check_panic(
    handler: &Handler,
    ctx: TypeCheckContext<'_>,
    engines: &Engines,
    expr: &Expression,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let mut ctx = ctx.with_type_annotation(engines.te().new_unknown());
    let expr_span = expr.span();
    let expr = ty::TyExpression::type_check(handler, ctx.by_ref(), expr)
        .unwrap_or_else(|err| ty::TyExpression::error(err, expr_span.clone(), engines));

    let expr_type_id = if ctx.experimental.new_encoding {
        // The type checked expression is either an `encode` call or an error.
        match &expr.expression {
            ty::TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                ..
            } => {
                if !(call_path.suffix.as_str() == "encode_allow_alias" && arguments.len() == 1) {
                    return Err(handler.emit_err(CompileError::Internal(
                        "In case of the new encoding, the `panic` expression argument must be a call to an \"encode\" function.",
                        expr_span
                    )));
                } else {
                    match &arguments[0].1.expression {
                        TyExpressionVariant::Ref(r) => r.return_type,
                        _ => todo!(),
                    }
                }
            }
            _ => expr.return_type, // Error. We just pass the type id through.
        }
    } else {
        expr.return_type
    };

    // TODO: (REFERENCES) Once we continue work on references, implement support for panicking on references
    //       of types that implement `std::marker::Error`.

    if !TraitMap::type_implements_trait(
        ctx.namespace().current_module(),
        engines,
        expr_type_id,
        |trait_entry| trait_entry.inner.is_std_marker_error_trait(),
    ) {
        return Err(
            handler.emit_err(CompileError::PanicExpressionArgumentIsNotError {
                argument_type: engines.help_out(expr_type_id).to_string(),
                span: expr.span.clone(),
            }),
        );
    }

    let typed_expr = ty::TyExpression {
        expression: ty::TyExpressionVariant::Panic(Box::new(expr)),
        return_type: engines.te().id_of_never(),
        span,
    };
    Ok(typed_expr)
}

fn check_asm_block_validity(
    handler: &Handler,
    asm: &AsmExpression,
    ctx: &TypeCheckContext,
) -> Result<(), ErrorEmitted> {
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
    for err in opcodes.iter().filter_map(|op| {
        if matches!(
            op.0,
            VirtualOp::JMP(..)
                | VirtualOp::JI(..)
                | VirtualOp::JNE(..)
                | VirtualOp::JNEI(..)
                | VirtualOp::JNZI(..)
                | VirtualOp::JMPB(..)
                | VirtualOp::JMPF(..)
                | VirtualOp::JNZB(..)
                | VirtualOp::JNZF(..)
                | VirtualOp::JNEB(..)
                | VirtualOp::JNEF(..)
                | VirtualOp::JAL(..)
                | VirtualOp::RET(..)
                | VirtualOp::RETD(..)
                | VirtualOp::RVRT(..)
        ) {
            Some(CompileError::DisallowedControlFlowInstruction {
                name: op.1.to_string(),
                span: op.2.clone(),
            })
        } else {
            None
        }
    }) {
        handler.emit_err(err);
    }

    // Check #2: Disallow initialized registers from being reassigned in the asm block
    //
    // 1. Collect all registers that have initializers in the list of arguments
    let initialized_registers = asm
        .registers
        .iter()
        .filter_map(|reg| {
            if reg.initializer.is_some() {
                Some(VirtualRegister::Virtual(reg.name.to_string()))
            } else {
                None
            }
        })
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
    for err in asm.registers.iter().filter_map(|reg| {
        if initialized_and_assigned_registers
            .contains(&VirtualRegister::Virtual(reg.name.to_string()))
        {
            Some(CompileError::InitializedRegisterReassignment {
                name: reg.name.to_string(),
                span: reg.name.span(),
            })
        } else {
            None
        }
    }) {
        handler.emit_err(err);
    }

    // Check #3: Check if there are uninitialized registers that are read before being written
    let mut uninitialized_registers = asm
        .registers
        .iter()
        .filter_map(|reg| {
            if reg.initializer.is_none() {
                let span = reg.name.span();

                // Emit warning if this register shadows a constant, or a configurable, or a variable.
                let temp_handler = Handler::default();
                let decl = ctx.resolve_call_path(
                    &temp_handler,
                    &CallPath {
                        prefixes: vec![],
                        suffix: sway_types::BaseIdent::new(span.clone()),
                        callpath_type: CallPathType::Ambiguous,
                    },
                );

                let shadowing_item = match decl {
                    Ok(ty::TyDecl::ConstantDecl(decl)) => {
                        let decl = ctx.engines.de().get_constant(&decl.decl_id);
                        Some((decl.name().into(), "Constant"))
                    }
                    Ok(ty::TyDecl::ConfigurableDecl(decl)) => {
                        let decl = ctx.engines.de().get_configurable(&decl.decl_id);
                        Some((decl.name().into(), "Configurable"))
                    }
                    Ok(ty::TyDecl::VariableDecl(decl)) => Some((decl.name.into(), "Variable")),
                    _ => None,
                };

                if let Some((item, item_kind)) = shadowing_item {
                    handler.emit_warn(CompileWarning {
                        span: span.clone(),
                        warning_content: Warning::UninitializedAsmRegShadowsItem {
                            constant_or_configurable_or_variable: item_kind,
                            item,
                        },
                    });
                }

                Some((VirtualRegister::Virtual(reg.name.to_string()), span))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    for (op, _, _) in opcodes.iter() {
        for being_read in op.use_registers() {
            if let Some(span) = uninitialized_registers.remove(being_read) {
                handler.emit_err(CompileError::UninitRegisterInAsmBlockBeingRead { span });
            }
        }

        for being_written in op.def_registers() {
            uninitialized_registers.remove(being_written);
        }
    }

    if let Some((reg, _)) = asm.returns.as_ref() {
        let reg = VirtualRegister::Virtual(reg.name.to_string());
        if let Some(span) = uninitialized_registers.remove(&reg) {
            handler.emit_err(CompileError::UninitRegisterInAsmBlockBeingRead { span });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Engines, ExperimentalFeatures};
    use sway_error::type_error::TypeError;
    use sway_types::ProgramId;
    use symbol_collection_context::SymbolCollectionContext;

    fn do_type_check(
        handler: &Handler,
        engines: &Engines,
        expr: &Expression,
        type_annotation: TypeId,
        experimental: ExperimentalFeatures,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let root_module_name = sway_types::Ident::new_no_span("do_type_check_test".to_string());
        let root_module = namespace::Package::new(root_module_name, None, ProgramId::new(0), false);
        let collection_ctx_ns = Namespace::new(handler, engines, root_module.clone(), true)?;
        let mut collection_ctx = SymbolCollectionContext::new(collection_ctx_ns);
        let mut namespace = Namespace::new(handler, engines, root_module, true)?;
        let ctx =
            TypeCheckContext::from_root(&mut namespace, &mut collection_ctx, engines, experimental)
                .with_type_annotation(type_annotation);
        ty::TyExpression::type_check(handler, ctx, expr)
    }

    fn do_type_check_for_boolx2(
        handler: &Handler,
        expr: &Expression,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let engines = Engines::default();
        let expr = do_type_check(
            handler,
            &engines,
            expr,
            engines
                .te()
                .insert_array_without_annotations(&engines, engines.te().id_of_bool(), 2),
            ExperimentalFeatures::default(),
        )?;
        expr.type_check_analyze(handler, &mut TypeCheckAnalysisContext::new(&engines))?;
        Ok(expr)
    }

    #[test]
    fn test_array_type_check_non_homogeneous_0() {
        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression::Explicit {
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
        let _comp_res = do_type_check_for_boolx2(&handler, &expr);
        let (errors, _warnings, _infos) = handler.consume();

        assert_eq!(errors.len(), 1);
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
            kind: ExpressionKind::Array(ArrayExpression::Explicit {
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
        let _comp_res = do_type_check_for_boolx2(&handler, &expr);
        let (errors, _warnings, _infos) = handler.consume();

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
    fn test_array_type_check_bad_count() {
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(ArrayExpression::Explicit {
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
        let _comp_res = do_type_check_for_boolx2(&handler, &expr);
        let (errors, _warnings, _infos) = handler.consume();
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
            kind: ExpressionKind::Array(ArrayExpression::Explicit {
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
            &expr,
            engines
                .te()
                .insert_array_without_annotations(&engines, engines.te().id_of_bool(), 0),
            ExperimentalFeatures::default(),
        );
        let (errors, warnings, infos) = handler.consume();
        assert!(comp_res.is_ok());
        assert!(infos.is_empty() && warnings.is_empty() && errors.is_empty());
    }
}
