mod analysis;
mod typed;

pub(crate) use analysis::check_match_expression_usefulness;
pub(crate) use typed::{MatchReqMap, TypedMatchExpression};
