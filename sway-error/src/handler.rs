use crate::error::CompileError;

use core::cell::RefCell;

/// A handler with which you can emit errors.
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
    sink: Vec<CompileError>,
}

impl Handler {
    /// Emit the error `err`.
    pub fn emit_err(&self, err: CompileError) {
        self.inner.borrow_mut().sink.push(err);
    }

    /// Extract all the errors from this handler.
    pub fn into_errors(self) -> Vec<CompileError> {
        self.inner.into_inner().sink
    }
}
