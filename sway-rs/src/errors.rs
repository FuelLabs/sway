use anyhow::anyhow;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// Invalid name
    #[error("Invalid name: {0}")]
    InvalidName(String),
    /// Invalid data
    #[error("Invalid data")]
    InvalidData,
    #[error("Missing data: {0}")]
    MissingData(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Invalid type: {0}")]
    InvalidType(String),
}
