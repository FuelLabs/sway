

use sway_types::{span};


use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub(crate) condition: MatchCondition,
    pub(crate) result: Expression,
    pub(crate) span: span::Span,
}
