use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error("Error parsing file: {0}")]
    ParseFileError(#[from] sway_parse::ParseFileError),
}
