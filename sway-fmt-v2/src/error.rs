use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error("Error parsing file: {0}")]
    ParseFileError(#[from] sway_parse::ParseFileError),
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
