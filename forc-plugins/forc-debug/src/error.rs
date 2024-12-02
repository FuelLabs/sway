use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForcDebugError {
    #[error("Command argument error: {0}")]
    ArgumentError(#[from] ArgumentError),

    #[error("VM error: {0}")]
    VMError(String),

    #[error("Fuel Client error: {0}")]
    FuelClientError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ArgumentError {
    #[error("Invalid argument: {0}")]
    Invalid(String),

    #[error("Not enough arguments, expected {expected} but got {got}")]
    NotEnough { expected: usize, got: usize },

    #[error("Too many arguments, expected {expected} but got {got}")]
    TooMany { expected: usize, got: usize },

    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

pub type Result<T> = std::result::Result<T, ForcDebugError>;

impl ArgumentError {
    /// Ensures argument count falls within [min, max] range.
    pub fn ensure_arg_count(
        args: &[String],
        min: usize,
        max: usize,
    ) -> std::result::Result<(), ArgumentError> {
        let count = args.len();
        if count < min {
            Err(ArgumentError::NotEnough {
                expected: min,
                got: count,
            })
        } else if count > max {
            Err(ArgumentError::TooMany {
                expected: max,
                got: count,
            })
        } else {
            Ok(())
        }
    }
}
