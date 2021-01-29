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
}
