use crate::{error::CompileError, warning::CompileWarning};
use core::cell::RefCell;

/// A handler with which you can emit diagnostics.
#[derive(Default, Debug, Clone)]
pub struct Handler {
    /// The inner handler.
    /// This construction is used to avoid `&mut` all over the compiler.
    inner: RefCell<HandlerInner>,
}

/// Contains the actual data for `Handler`.
/// Modelled this way to afford an API using interior mutability.
#[derive(Default, Debug, Clone)]
struct HandlerInner {
    /// The sink through which errors will be emitted.
    errors: Vec<CompileError>,
    /// The sink through which warnings will be emitted.
    warnings: Vec<CompileWarning>,
}

impl Handler {
    pub fn from_parts(errors: Vec<CompileError>, warnings: Vec<CompileWarning>) -> Self {
        Self {
            inner: RefCell::new(HandlerInner { errors, warnings }),
        }
    }

    /// Emit the error `err`.
    pub fn emit_err(&self, err: CompileError) -> ErrorEmitted {
        eprintln!("{err:?} {}", std::backtrace::Backtrace::force_capture());

        self.inner.borrow_mut().errors.push(err);
        ErrorEmitted { _priv: () }
    }

    // Compilation should be cancelled.
    pub fn cancel(&self) -> ErrorEmitted {
        ErrorEmitted { _priv: () }
    }

    /// Emit the warning `warn`.
    pub fn emit_warn(&self, warn: CompileWarning) {
        self.inner.borrow_mut().warnings.push(warn);
    }

    pub fn has_errors(&self) -> bool {
        !self.inner.borrow().errors.is_empty()
    }

    pub fn find_error(&self, f: impl FnMut(&&CompileError) -> bool) -> Option<CompileError> {
        self.inner.borrow().errors.iter().find(f).cloned()
    }

    pub fn has_warnings(&self) -> bool {
        !self.inner.borrow().warnings.is_empty()
    }

    pub fn scope<T>(
        &self,
        f: impl FnOnce(&Handler) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        let scoped_handler = Handler::default();
        let closure_res = f(&scoped_handler);

        match self.append(scoped_handler) {
            Some(err) => Err(err),
            None => closure_res,
        }
    }

    /// Extract all the warnings and errors from this handler.
    pub fn consume(self) -> (Vec<CompileError>, Vec<CompileWarning>) {
        let inner = self.inner.into_inner();
        (inner.errors, inner.warnings)
    }

    pub fn append(&self, other: Handler) -> Option<ErrorEmitted> {
        let other_has_errors = other.has_errors();

        let (errors, warnings) = other.consume();
        for warn in warnings {
            self.emit_warn(warn);
        }
        for err in errors {
            self.emit_err(err);
        }

        if other_has_errors {
            Some(ErrorEmitted { _priv: () })
        } else {
            None
        }
    }

    pub fn dedup(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.errors = dedup_unsorted(inner.errors.clone());
        inner.warnings = dedup_unsorted(inner.warnings.clone());
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    pub fn retain_err<F>(&self, f: F)
    where
        F: FnMut(&CompileError) -> bool,
    {
        self.inner.borrow_mut().errors.retain(f)
    }

    // Map all errors from `other` into this handler. If any mapping returns `None` it is ignored. This
    // method returns if any error was mapped or not.
    pub fn map_and_emit_errors_from(
        &self,
        other: Handler,
        mut f: impl FnMut(CompileError) -> Option<CompileError>,
    ) -> Result<(), ErrorEmitted> {
        let mut emitted = Ok(());

        let (errs, _) = other.consume();
        for err in errs {
            if let Some(err) = (f)(err) {
                emitted = Err(self.emit_err(err));
            }
        }

        emitted
    }
}

/// Proof that an error was emitted through a `Handler`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ErrorEmitted {
    _priv: (),
}

/// We want compile errors and warnings to retain their ordering, since typically
/// they are grouped by relevance. However, we want to deduplicate them.
/// Stdlib dedup in Rust assumes sorted data for efficiency, but we don't want that.
/// A hash set would also mess up the order, so this is just a brute force way of doing it
/// with a vector.
fn dedup_unsorted<T: PartialEq + std::hash::Hash + Clone + Eq>(mut data: Vec<T>) -> Vec<T> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    data.retain(|item| seen.insert(item.clone()));
    data
}
