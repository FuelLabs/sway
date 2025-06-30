use regex::Error as RegexError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MDPreProcessError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] RegexError),

    #[error("Missing include file: {0}")]
    MissingInclude(PathBuf),

    #[error("Cycle detected in includes!")]
    Cycle,

    #[error("Failed to canonicalize path: {0}")]
    Canonicalize(PathBuf),

    #[error("Other error: {0}")]
    Other(String),
}
