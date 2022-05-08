use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ordering {
    /// Reorder import and extern crate statements alphabetically.
    reorder_imports: bool,
    /// Reorder module statements alphabetically in group.
    reorder_modules: bool,
    /// Reorder `impl` items.
    reorder_impl_items: bool,
}
