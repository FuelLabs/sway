use super::scrutinee::Scrutinee;

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub enum MatchCondition {
    CatchAll(CatchAll),
    Scrutinee(Scrutinee),
}

#[derive(Debug, Clone)]
pub struct CatchAll {
    pub span: Span,
}
