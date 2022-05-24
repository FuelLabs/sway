use sway_types::span;

use super::{Expression, Scrutinee};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub(crate) condition: Scrutinee,
    pub(crate) result: Expression,
    pub(crate) span: span::Span,
}
