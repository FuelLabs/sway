//! Configuration options related to re-ordering imports, modules and items.

#[derive(Debug)]
pub struct Ordering {
    /// Reorder import and extern crate statements alphabetically.
    pub reorder_imports: bool,
    /// Reorder module statements alphabetically in group.
    pub reorder_modules: bool,
    /// Reorder `impl` items.
    pub reorder_impl_items: bool,
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
