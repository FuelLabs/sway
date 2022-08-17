//! Configuration options related to re-ordering imports, modules and items.
use crate::config::user_opts::OrderingOptions;

#[derive(Debug, Clone)]
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

impl Ordering {
    pub fn from_opts(opts: &OrderingOptions) -> Self {
        let default = Self::default();
        Self {
            reorder_imports: opts.reorder_imports.unwrap_or(default.reorder_imports),
            reorder_modules: opts.reorder_modules.unwrap_or(default.reorder_modules),
            reorder_impl_items: opts
                .reorder_impl_items
                .unwrap_or(default.reorder_impl_items),
        }
    }
}
