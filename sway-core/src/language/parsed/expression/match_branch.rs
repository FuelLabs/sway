use sway_types::{span, Span};

use super::{Expression, Literal, Scrutinee};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub scrutinee: Scrutinee,
    pub result: Expression,
    pub(crate) span: span::Span,
}

impl MatchBranch {
    pub fn catch_all(result: Expression) -> Self {
        MatchBranch {
            scrutinee: Scrutinee::CatchAll {
                span: Span::dummy(),
            },
            result,
            span: Span::dummy(),
        }
    }

    pub fn literal(value: Literal, result: Expression) -> Self {
        MatchBranch {
            scrutinee: Scrutinee::Literal {
                value,
                span: Span::dummy(),
            },
            result,
            span: Span::dummy(),
        }
    }
}
