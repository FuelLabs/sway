use super::{Expression, Scrutinee};
use sway_types::span;

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee: Scrutinee,
    pub result: Expression,
    pub(crate) span: span::Span,
}
