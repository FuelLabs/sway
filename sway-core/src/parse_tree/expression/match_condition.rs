use super::scrutinee::Scrutinee;

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub enum MatchCondition {
    CatchAll(Span),
    Scrutinee(Scrutinee),
}

impl MatchCondition {
    pub(crate) fn span(&self) -> Span {
        match self {
            MatchCondition::CatchAll(span) => span.clone(),
            MatchCondition::Scrutinee(scrutinee) => scrutinee.span(),
        }
    }
}
