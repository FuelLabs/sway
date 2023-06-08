pub(crate) mod dispatch;
pub(crate) mod global_state;
pub(crate) mod main_loop;
pub(crate) mod task_pool;

use serde::de::DeserializeOwned;
use std::{
    fmt,
    panic::{self, UnwindSafe},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn from_json<T: DeserializeOwned>(what: &'static str, json: &serde_json::Value) -> Result<T> {
    let res = serde_json::from_value(json.clone())
        .map_err(|e| format!("Failed to deserialize {what}: {e}; {json}"))?;
    Ok(res)
}

#[derive(Debug)]
struct LspError {
    code: i32,
    message: String,
}

impl LspError {
    fn new(code: i32, message: String) -> LspError {
        LspError { code, message }
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Language Server request failed with {}. ({})",
            self.code, self.message
        )
    }
}

impl std::error::Error for LspError {}

/// A panic payload indicating that a salsa revision was cancelled.
#[derive(Debug)]
#[non_exhaustive]
pub struct Cancelled;

impl Cancelled {
    fn throw() -> ! {
        // We use resume and not panic here to avoid running the panic
        // hook (that is, to avoid collecting and printing backtrace).
        std::panic::resume_unwind(Box::new(Self));
    }

    /// Runs `f`, and catches any salsa cancellation.
    pub fn catch<F, T>(f: F) -> Result<T, Cancelled>
    where
        F: FnOnce() -> T + UnwindSafe,
    {
        match panic::catch_unwind(f) {
            Ok(t) => Ok(t),
            Err(payload) => match payload.downcast() {
                Ok(cancelled) => Err(*cancelled),
                Err(payload) => panic::resume_unwind(payload),
            },
        }
    }
}

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cancelled")
    }
}

impl std::error::Error for Cancelled {}
