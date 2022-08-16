use sway_types::Span;

use crate::{
    semantic_analysis::{IsConstant, TypedExpressionVariant},
    type_system::TypeId,
    LazyOp,
};

use super::TypedExpression;

pub(crate) fn instantiate_lazy_operator(
    op: LazyOp,
    lhs: TypedExpression,
    rhs: TypedExpression,
    return_type: TypeId,
    span: Span,
) -> TypedExpression {
    TypedExpression {
        expression: TypedExpressionVariant::LazyOperator {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        return_type,
        is_constant: IsConstant::No,
        span,
    }
}
