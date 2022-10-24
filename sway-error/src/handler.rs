use crate::{error::CompileError, warning::CompileWarning};

use core::cell::RefCell;

/// A handler with which you can emit diagnostics.
#[derive(Default)]
pub struct Handler {
    /// The inner handler.
    /// This construction is used to avoid `&mut` all over the compiler.
    inner: RefCell<HandlerInner>,
}

/// Contains the actual data for `Handler`.
/// Modelled this way to afford an API using interior mutability.
#[derive(Default)]
struct HandlerInner {
    /// The sink through which errors will be emitted.
    errors:   Vec<CompileError>,
    /// The sink through which warnings will be emitted.
    warnings: Vec<CompileWarning>,
}

impl Handler {
    /// Emit the error `err`.
    pub fn emit_err(&self, err: CompileError) -> ErrorEmitted {
        self.inner.borrow_mut().errors.push(err);
        ErrorEmitted { _priv: () }
    }

    /// Emit the warning `warn`.
    pub fn emit_warn(&self, warn: CompileWarning) {
        self.inner.borrow_mut().warnings.push(warn);
    }

    /// Extract all the errors from this handler.
    pub fn consume(self) -> (Vec<CompileError>, Vec<CompileWarning>) {
        let inner = self.inner.into_inner();
        (inner.errors, inner.warnings)
    }
}

/// Proof that an error was emitted through a `Handler`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorEmitted {
    _priv: (),
}
