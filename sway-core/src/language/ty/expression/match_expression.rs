use sway_types::Span;

use crate::{language::ty::*, semantic_analysis::ReqDeclTree, type_system::*};

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    pub(crate) value_type_id: TypeId,
    pub(crate) branches: Vec<TyMatchBranch>,
    pub(crate) return_type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    pub(crate) req_decl_tree: ReqDeclTree, // TODO-IG: Remove. Replace with new Req struct.
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    pub(crate) span: Span,
}
