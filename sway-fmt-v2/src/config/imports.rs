//! Configuration options related to formatting imports.
use crate::config::{lists::ListTactic, user_opts::ImportsOptions, whitespace::IndentStyle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub struct Imports {
    /// Controls the strategy for how imports are grouped together.
    pub group_imports: GroupImports,
    /// Merge or split imports to the provided granularity.
    pub imports_granularity: ImportGranularity,
    /// Indent of imports.
    pub imports_indent: IndentStyle,
    /// Item layout inside a import block.
    pub imports_layout: ListTactic,
}

impl Default for Imports {
    fn default() -> Self {
        Self {
            group_imports: GroupImports::Preserve,
            imports_granularity: ImportGranularity::Preserve,
            imports_indent: IndentStyle::Block,
            imports_layout: ListTactic::Mixed,
        }
    }
}

impl Imports {
    pub fn from_opts(opts: &ImportsOptions) -> Self {
        let default = Self::default();
        Self {
            group_imports: opts.group_imports.unwrap_or(default.group_imports),
            imports_granularity: opts
                .imports_granularity
                .unwrap_or(default.imports_granularity),
            imports_indent: opts.imports_indent.unwrap_or(default.imports_indent),
            imports_layout: opts.imports_layout.unwrap_or(default.imports_layout),
        }
    }
}

/// Configuration for import groups, i.e. sets of imports separated by newlines.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum GroupImports {
    /// Keep groups as they are.
    Preserve,
    /// Discard existing groups, and create new groups for
    ///  1. `std` / `core` / `alloc` imports
    ///  2. other imports
    ///  3. `self` / `crate` / `super` imports
    StdExternalCrate,
    /// Discard existing groups, and create a single group for everything
    One,
}

/// How to merge imports.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ImportGranularity {
    /// Do not merge imports.
    Preserve,
    /// Use one `use` statement per crate.
    Crate,
    /// Use one `use` statement per module.
    Module,
    /// Use one `use` statement per imported item.
    Item,
    /// Use one `use` statement including all items.
    One,
}
