use crate::Span;

use super::scrutinee::Scrutinee;

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition<'sc> {
    CatchAll(CatchAll<'sc>),
    Scrutinee(Scrutinee<'sc>),
}

#[derive(Debug, Clone)]
pub struct CatchAll<'sc> {
    pub span: Span<'sc>
}