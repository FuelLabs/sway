use sway_types::span;

use super::{Expression, Scrutinee};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee:   Scrutinee,
    pub result:      Expression,
    pub(crate) span: span::Span,
}
