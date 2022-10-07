use sway_types::Span;

use crate::{
    language::LazyOp,
    semantic_analysis::{IsConstant, TyExpressionVariant},
    type_system::TypeId,
};

use super::TyExpression;

pub(crate) fn instantiate_lazy_operator(
    op: LazyOp,
    lhs: TyExpression,
    rhs: TyExpression,
    return_type: TypeId,
    span: Span,
) -> TyExpression {
    TyExpression {
        expression: TyExpressionVariant::LazyOperator {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        return_type,
        is_constant: IsConstant::No,
        span,
    }
}
