use std::{io, path::PathBuf};
use sway_error::error::CompileError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("Unable to parse: {}", self.0.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"))]
pub struct ParseFileError(pub Vec<CompileError>);

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error("Error parsing file: {0}")]
    ParseFileError(#[from] ParseFileError),
    #[error("Error formatting a message into a stream: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Error while adding comments")]
    CommentError,
    #[error("Error while formatting newline sequences")]
    NewlineSequenceError,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse config: {err}")]
    Deserialize { err: toml::de::Error },
    #[error("failed to read config at {:?}: {err}", path)]
    ReadConfig { path: PathBuf, err: io::Error },
    #[error("could not find a `swayfmt.toml` in the given directory or its parents")]
    NotFound,
}
