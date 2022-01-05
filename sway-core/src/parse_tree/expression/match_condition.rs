use crate::Span;

use super::scrutinee::Scrutinee;

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition {
    CatchAll(CatchAll),
    Scrutinee(Scrutinee),
}

#[derive(Debug, Clone)]
pub struct CatchAll {
    pub span: Span,
}
