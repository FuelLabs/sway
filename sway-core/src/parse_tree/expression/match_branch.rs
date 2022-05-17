use sway_types::Span;

use super::{scrutinee::Scrutinee, Expression};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee: Scrutinee,
    pub result: Expression,
    pub(crate) span: Span,
}
