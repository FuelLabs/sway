use sway_types::Span;

use crate::TypeId;

use super::TyExpression;

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    branches: Vec<TyMatchBranch>,
    return_type_id: TypeId,
    #[allow(dead_code)]
    span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    pub(crate) conditions: MatchReqMap,
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    span: Span,
}
