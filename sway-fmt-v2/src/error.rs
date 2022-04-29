use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

impl std::convert::From<sway_parse::ParseFileError> for FormatterError {}