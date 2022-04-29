use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}