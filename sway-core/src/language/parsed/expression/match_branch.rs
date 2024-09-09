use crate::engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext};

use super::{Expression, Scrutinee};
use sway_types::span;

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee: Scrutinee,
    pub result: Expression,
    pub(crate) span: span::Span,
}

impl EqWithEngines for MatchBranch {}
impl PartialEqWithEngines for MatchBranch {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.scrutinee.eq(&other.scrutinee, ctx) && self.result.eq(&other.result, ctx)
    }
}
