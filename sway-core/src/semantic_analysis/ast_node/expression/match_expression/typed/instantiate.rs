use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{integer_bits::IntegerBits, Ident, Span};

use crate::{
    language::{ty, LazyOp, Literal},
    semantic_analysis::{
        typed_expression::{instantiate_lazy_operator, instantiate_tuple_index_access},
        TypeCheckContext,
    },
    Engines, TypeId, TypeInfo,
};

/// Simplifies instantiation of desugared code in the match expression and match arms.
pub(super) struct Instantiate {
    /// Both dummy span for instantiation of desugared elements
    /// and error span for internal compiler errors.
    span: Span,
    u64_type: TypeId,
    boolean_type: TypeId,
    revert_type: TypeId,
}

impl Instantiate {
    pub(super) fn new(engines: &Engines, span: Span) -> Self {
        let type_engine = engines.te();
        let u64_type = type_engine.insert(
            engines,
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            None,
        );
        let boolean_type = type_engine.insert(engines, TypeInfo::Boolean, None);
        let revert_type = type_engine.insert(engines, TypeInfo::Never, None);

        Self {
            span,
            u64_type,
            boolean_type,
            revert_type,
        }
    }

    pub(super) fn dummy_span(&self) -> Span {
        self.span.clone()
    }

    pub(super) fn error_span(&self) -> Span {
        self.span.clone()
    }

    pub(super) fn u64_type(&self) -> TypeId {
        self.u64_type
    }

    /// Instantiates a [ty::TyDecl::VariableDecl] for an immutable variable of the form `let <name> = <body>;`.
    pub(super) fn var_decl(&self, name: Ident, body: ty::TyExpression) -> ty::TyDecl {
        let return_type = body.return_type;
        let type_ascription = body.return_type.into();

        ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
            name,
            body,
            mutability: ty::VariableMutability::Immutable,
            return_type,
            type_ascription,
        }))
    }

    /// Instantiates a [ty::TyExpressionVariant::VariableExpression] for accessing an immutable variable
    /// `name` of the type `type_id`.
    pub(super) fn var_exp(&self, name: Ident, type_id: TypeId) -> ty::TyExpression {
        ty::TyExpression {
            expression: ty::TyExpressionVariant::VariableExpression {
                name,
                span: self.dummy_span(),
                mutability: ty::VariableMutability::Immutable,
                call_path: None,
            },
            return_type: type_id,
            span: self.dummy_span(),
        }
    }

    /// Instantiates a [ty::TyExpressionVariant::Literal] that represents a `u64` `value`.
    pub(super) fn u64_literal(&self, value: u64) -> ty::TyExpression {
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::U64(value)),
            return_type: self.u64_type,
            span: self.dummy_span(),
        }
    }

    /// Instantiates a [ty::TyExpressionVariant::Literal] that represents a `boolean` `value`.
    pub(super) fn boolean_literal(&self, value: bool) -> ty::TyExpression {
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::Boolean(value)),
            return_type: self.boolean_type,
            span: self.dummy_span(),
        }
    }

    /// Instantiates an [Ident] with overridden `name`.
    pub(super) fn ident(&self, name: String) -> Ident {
        Ident::new_with_override(name, self.dummy_span())
    }

    /// Instantiates a [ty::TyExpressionVariant::CodeBlock] with a single
    /// [ty::TyAstNodeContent::ImplicitReturnExpression] that returns the `value`.
    pub(super) fn code_block_with_implicit_return_u64(&self, value: u64) -> ty::TyExpression {
        let ret_expr = ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::U64(value)),
            return_type: self.u64_type,
            span: self.dummy_span(),
        };
        ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                whole_block_span: self.dummy_span(),
                contents: vec![ty::TyAstNode {
                    content: ty::TyAstNodeContent::Expression(ty::TyExpression {
                        return_type: ret_expr.return_type,
                        span: ret_expr.span.clone(),
                        expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(ret_expr)),
                    }),
                    span: self.dummy_span(),
                }],
            }),
            return_type: self.u64_type,
            span: self.dummy_span(),
        }
    }

    /// Instantiates a [ty::TyExpressionVariant::CodeBlock] with a single
    /// [ty::TyAstNodeContent::ImplicitReturnExpression] that returns calls `__revert(revert_code)`.
    pub(super) fn code_block_with_implicit_return_revert(
        &self,
        revert_code: u64,
    ) -> ty::TyExpression {
        let ret_expr = ty::TyExpression {
            expression: ty::TyExpressionVariant::IntrinsicFunction(ty::TyIntrinsicFunctionKind {
                kind: sway_ast::Intrinsic::Revert,
                arguments: vec![ty::TyExpression {
                    expression: ty::TyExpressionVariant::Literal(Literal::U64(revert_code)),
                    return_type: self.u64_type,
                    span: self.dummy_span(),
                }],
                type_arguments: vec![],
                span: self.dummy_span(),
            }),
            return_type: self.revert_type,
            span: self.dummy_span(),
        };
        ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                whole_block_span: self.dummy_span(),
                contents: vec![ty::TyAstNode {
                    content: ty::TyAstNodeContent::Expression(ty::TyExpression {
                        return_type: ret_expr.return_type,
                        span: ret_expr.span.clone(),
                        expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(ret_expr)),
                    }),
                    span: self.dummy_span(),
                }],
            }),
            return_type: self.revert_type,
            span: self.dummy_span(),
        }
    }

    /// Instantiates an expression equivalent to `<lhs> == <rhs>`.
    pub(super) fn eq_result(
        &self,
        handler: &Handler,
        ctx: TypeCheckContext,
        lhs: ty::TyExpression,
        rhs: ty::TyExpression,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        ty::TyExpression::core_ops_eq(handler, ctx, vec![lhs, rhs], self.dummy_span())
    }

    /// Instantiates an expression equivalent to `<lhs> != <rhs>`.
    pub(super) fn neq_result(
        &self,
        handler: &Handler,
        ctx: TypeCheckContext,
        lhs: ty::TyExpression,
        rhs: ty::TyExpression,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        ty::TyExpression::core_ops_neq(handler, ctx, vec![lhs, rhs], self.dummy_span())
    }

    /// Instantiates an expression equivalent to `<lhs> == <rhs>`. The method expects that
    /// the expression can be instantiated and panics if that's not the case.
    pub(super) fn eq(
        &self,
        ctx: TypeCheckContext,
        lhs: ty::TyExpression,
        rhs: ty::TyExpression,
    ) -> ty::TyExpression {
        ty::TyExpression::core_ops_eq(&Handler::default(), ctx, vec![lhs, rhs], self.dummy_span())
            .expect("Instantiating `core::ops::eq` is expected to always work.")
    }

    /// Instantiates a [ty::TyExpressionVariant::TupleElemAccess] `<tuple_variable>.<index>`. The method expects that
    /// the expression can be instantiated and panics if that's not the case.
    pub(super) fn tuple_elem_access(
        &self,
        engines: &Engines,
        tuple_variable: ty::TyExpression,
        index: usize,
    ) -> ty::TyExpression {
        instantiate_tuple_index_access(
            &Handler::default(),
            engines,
            tuple_variable,
            index,
            self.dummy_span(),
            self.dummy_span(),
        )
        .expect("Instantiating tuple element access expression is expected to always work.")
    }

    /// Instantiates a [LazyOp::And] expression of the form `<lhs> && <rhs>`.
    pub(super) fn lazy_and(
        &self,
        lhs: ty::TyExpression,
        rhs: ty::TyExpression,
    ) -> ty::TyExpression {
        instantiate_lazy_operator(LazyOp::And, lhs, rhs, self.boolean_type, self.dummy_span())
    }

    /// Instantiates a [LazyOp::Or] expression of the form `<lhs> || <rhs>`.
    pub(super) fn lazy_or(&self, lhs: ty::TyExpression, rhs: ty::TyExpression) -> ty::TyExpression {
        instantiate_lazy_operator(LazyOp::Or, lhs, rhs, self.boolean_type, self.dummy_span())
    }
}
