//! Common types shared between forc crates.

use std::{fmt::Display, process::Termination};

pub use ansiterm;
pub use paste;

pub const DEFAULT_OUTPUT_DIRECTORY: &str = "out";
pub const DEFAULT_ERROR_EXIT_CODE: u8 = 1;
pub const DEFAULT_SUCCESS_EXIT_CODE: u8 = 0;

/// A result type for forc operations. This shouldn't be returned from entry points, instead return
/// `ForcCliResult` to exit with correct exit code.
pub type ForcResult<T, E = ForcError> = Result<T, E>;

/// A wrapper around `ForcResult`. Designed to be returned from entry points as it handles
/// error reporting and exits with correct exit code.
#[derive(Debug)]
pub struct ForcCliResult<T> {
    result: ForcResult<T>,
}

/// A forc error type which is a wrapper around `anyhow::Error`. It enables propagation of custom
/// exit code alongside the original error.
#[derive(Debug)]
pub struct ForcError {
    error: anyhow::Error,
    exit_code: u8,
}

impl ForcError {
    pub fn new(error: anyhow::Error, exit_code: u8) -> Self {
        Self { error, exit_code }
    }

    /// Returns a `ForcError` with provided exit_code.
    pub fn exit_code(self, exit_code: u8) -> Self {
        Self {
            error: self.error,
            exit_code,
        }
    }
}

impl AsRef<anyhow::Error> for ForcError {
    fn as_ref(&self) -> &anyhow::Error {
        &self.error
    }
}

impl From<&str> for ForcError {
    fn from(value: &str) -> Self {
        Self {
            error: anyhow::anyhow!("{value}"),
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl From<anyhow::Error> for ForcError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            error: value,
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl From<std::io::Error> for ForcError {
    fn from(value: std::io::Error) -> Self {
        Self {
            error: value.into(),
            exit_code: DEFAULT_ERROR_EXIT_CODE,
        }
    }
}

impl Display for ForcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T> Termination for ForcCliResult<T> {
    fn report(self) -> std::process::ExitCode {
        match self.result {
            Ok(_) => DEFAULT_SUCCESS_EXIT_CODE.into(),
            Err(e) => {
                forc_diagnostic::println_error(&format!("{e}"));
                e.exit_code.into()
            }
        }
    }
}

impl<T> From<ForcResult<T>> for ForcCliResult<T> {
    fn from(value: ForcResult<T>) -> Self {
        Self { result: value }
    }
}

#[macro_export]
macro_rules! forc_result_bail {
    ($msg:literal $(,)?) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($msg).into())
    };
    ($err:expr $(,)?) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($err).into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::ForcResult::Err(anyhow::anyhow!($fmt, $($arg)*).into())
    };
}

#[macro_use]
pub mod cli;
