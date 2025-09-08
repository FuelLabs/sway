use crate::types::Instruction;
use dap::requests::Command;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ArgumentError(#[from] ArgumentError),

    #[error(transparent)]
    AdapterError(#[from] AdapterError),

    #[error("VM error: {0}")]
    VMError(String),

    #[error("Fuel Client error: {0}")]
    FuelClientError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("ABI error: {0}")]
    AbiError(String),

    #[error("Json error")]
    JsonError(#[from] serde_json::Error),

    #[error("Server error: {0}")]
    DapServerError(#[from] dap::errors::ServerError),

    #[error("Readline error: {0}")]
    Readline(#[from] rustyline::error::ReadlineError),
}

#[derive(Debug, thiserror::Error)]
pub enum ArgumentError {
    #[error("Invalid argument: {0}")]
    Invalid(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Not enough arguments, expected {expected} but got {got}")]
    NotEnough { expected: usize, got: usize },

    #[error("Too many arguments, expected {expected} but got {got}")]
    TooMany { expected: usize, got: usize },

    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Unhandled command")]
    UnhandledCommand { command: Command },

    #[error("Missing command")]
    MissingCommand,

    #[error("Missing configuration")]
    MissingConfiguration,

    #[error("Missing source path argument")]
    MissingSourcePathArgument,

    #[error("Missing breakpoint location")]
    MissingBreakpointLocation,

    #[error("Missing source map")]
    MissingSourceMap { pc: Instruction },

    #[error("Unknown breakpoint")]
    UnknownBreakpoint { pc: Instruction },

    #[error("Build failed")]
    BuildFailed { reason: String },

    #[error("No active test executor")]
    NoActiveTestExecutor,

    #[error("Test execution failed")]
    TestExecutionFailed {
        #[from]
        source: anyhow::Error,
    },
}

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
