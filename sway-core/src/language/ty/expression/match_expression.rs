use sway_types::Span;

use crate::{language::ty::*, semantic_analysis::MatchReqMap, type_system::*};

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    pub(crate) branches: Vec<TyMatchBranch>,
    pub(crate) return_type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    pub(crate) conditions: MatchReqMap,
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    pub(crate) span: Span,
}
