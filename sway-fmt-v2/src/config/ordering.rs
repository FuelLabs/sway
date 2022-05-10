//! Configuration options related to re-ordering imports, modules and items.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Ordering {
    /// Reorder import and extern crate statements alphabetically.
    reorder_imports: bool,
    /// Reorder module statements alphabetically in group.
    reorder_modules: bool,
    /// Reorder `impl` items.
    reorder_impl_items: bool,
}

impl Default for Ordering {
    fn default() -> Self {
        Self {
            reorder_imports: true,
            reorder_modules: true,
            reorder_impl_items: false,
        }
    }
}
