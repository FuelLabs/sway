use sway_types::Span;
use crate::Engines;
use crate::type_system::TypeInfo;

use crate::{
    language::{ty, LazyOp},
    type_system::TypeId,
};

pub(crate) fn instantiate_lazy_operator(
    op: LazyOp,
    lhs: ty::TyExpression,
    rhs: ty::TyExpression,
    return_type: TypeId,
    span: Span,
) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::LazyOperator {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        return_type,
        span,
    }
}

/// Instantiates a [LazyOp::Or] expression of the form `<lhs> || <rhs>`
/// whose span is the joined span of `lhs` and `rhs` spans.
pub(crate) fn instantiate_lazy_or(engines: &Engines, lhs: ty::TyExpression, rhs: ty::TyExpression) -> ty::TyExpression {
    let type_engine = engines.te();
    let span = Span::join(lhs.span.clone(), rhs.span.clone());

    instantiate_lazy_operator(
        LazyOp::Or,
        lhs,
        rhs,
        type_engine.insert(engines, TypeInfo::Boolean),
        span
    )
}

/// Instantiates a [LazyOp::And] expression of the form `<lhs> && <rhs>`
/// whose span is the joined span of `lhs` and `rhs` spans.
pub(crate) fn instantiate_lazy_and(engines: &Engines, lhs: ty::TyExpression, rhs: ty::TyExpression) -> ty::TyExpression {
    let type_engine = engines.te();
    let span = Span::join(lhs.span.clone(), rhs.span.clone());

    instantiate_lazy_operator(
        LazyOp::And,
        lhs,
        rhs,
        type_engine.insert(engines, TypeInfo::Boolean),
        span
    )
}
