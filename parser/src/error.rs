use crate::parser::Rule;
use pest::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError<'sc> {
    #[error("Error parsing input: {0:?}")]
    ParseFailure(#[from] pest::error::Error<Rule>),
    #[error("Invalid top-level item: {0:?}")]
    InvalidTopLevelItem(Rule, Span<'sc>),
    #[error("Internal compiler error: {0}. Please file an issue on the repository and include the code that triggered this error.")]
    Internal(&'static str, Span<'sc>),
    #[error("Unimplemented feature: {0:?}")]
    Unimplemented(Rule, Span<'sc>),
}

impl<'sc> CompileError<'sc> {
    pub fn span(&self) -> (usize, usize) {
        use CompileError::*;
        match self {
            ParseFailure(err) => match err.location{
                pest::error::InputLocation::Pos(num) => (num, num + 1),
                pest::error::InputLocation::Span((start, end)) => (start, end),
            },
            InvalidTopLevelItem(_, sp) => (sp.start(), sp.end()),
            Internal(_, sp) => (sp.start(), sp.end()),
            Unimplemented(_, sp) => (sp.start(), sp.end()),
        }
    }
}
