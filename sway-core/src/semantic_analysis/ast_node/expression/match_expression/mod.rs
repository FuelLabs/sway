mod typed;
mod analysis;

pub(crate) use typed::TypedMatchExpression;
pub(crate) use analysis::check_match_expression_usefulness;
