use crate::parser::Rule;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Error parsing input: {0:?}")]
    ParseFailure(#[from] pest::error::Error<Rule>),
}
