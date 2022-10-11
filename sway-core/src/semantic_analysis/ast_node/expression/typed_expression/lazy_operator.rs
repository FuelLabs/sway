use sway_types::Span;

use crate::{
    language::{ty, LazyOp},
    semantic_analysis::IsConstant,
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
        is_constant: IsConstant::No,
        span,
    }
}
