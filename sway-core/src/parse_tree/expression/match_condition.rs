use super::scrutinee::Scrutinee;

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub(crate) struct CatchAll {
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition {
    CatchAll(CatchAll),
    Scrutinee(Scrutinee),
}

impl MatchCondition {
    pub(crate) fn span(&self) -> Span {
        match self {
            MatchCondition::CatchAll(catch_all) => catch_all.span.clone(),
            MatchCondition::Scrutinee(scrutinee) => scrutinee.span(),
        }
    }
}
