use crate::parser::Rule;
use pest::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError<'sc> {
    #[error("Error parsing input: {0:?}")]
    ParseFailure(#[from] pest::error::Error<Rule>),
    #[error("Invalid top-level item: {0:?}")]
    InvalidTopLevelItem(Rule, Span<'sc>),
}
