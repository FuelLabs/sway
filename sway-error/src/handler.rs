use crate::{error::CompileError, warning::CompileWarning};
use std::collections::HashMap;

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
        let had_errors = scoped_handler.has_errors();

        self.append(scoped_handler);

        if had_errors {
            Err(ErrorEmitted { _priv: () })
        } else {
            closure_res
        }
    }

    /// Extract all the warnings and errors from this handler.
    pub fn consume(self) -> (Vec<CompileError>, Vec<CompileWarning>) {
        let inner = self.inner.into_inner();
        (inner.errors, inner.warnings)
    }

    pub fn append(&self, other: Handler) {
        let (errors, warnings) = other.consume();
        for warn in warnings {
            self.emit_warn(warn);
        }
        for err in errors {
            self.emit_err(err);
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
}

/// Proof that an error was emitted through a `Handler`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ErrorEmitted {
    _priv: (),
}

/// We want compile errors and warnings to retain their ordering, since typically
/// they are grouped by relevance. However, we want to deduplicate them.
/// Stdlib dedup in Rust assumes sorted data for efficiency, but we don't want that.
/// A hash set would also mess up the order, so this is just a brute force way of doing it
/// with a vector.
fn dedup_unsorted<T: PartialEq + std::hash::Hash>(mut data: Vec<T>) -> Vec<T> {
    // TODO(Centril): Consider using `IndexSet` instead for readability.
    use smallvec::SmallVec;
    use std::collections::hash_map::{DefaultHasher, Entry};
    use std::hash::Hasher;

    let mut write_index = 0;
    let mut indexes: HashMap<u64, SmallVec<[usize; 1]>> = HashMap::with_capacity(data.len());
    for read_index in 0..data.len() {
        let hash = {
            let mut hasher = DefaultHasher::new();
            data[read_index].hash(&mut hasher);
            hasher.finish()
        };
        let index_vec = match indexes.entry(hash) {
            Entry::Occupied(oe) => {
                if oe
                    .get()
                    .iter()
                    .any(|index| data[*index] == data[read_index])
                {
                    continue;
                }
                oe.into_mut()
            }
            Entry::Vacant(ve) => ve.insert(SmallVec::new()),
        };
        data.swap(write_index, read_index);
        index_vec.push(write_index);
        write_index += 1;
    }
    data.truncate(write_index);
    data
}
