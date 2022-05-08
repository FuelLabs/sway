use serde::{Deserialize, Serialize};

use super::{fundamentals::IndentStyle, lists::ListTactic};

#[derive(Serialize, Deserialize, Debug)]
pub struct Imports {
    /// Controls the strategy for how imports are grouped together.
    pub group_imports: GroupImportsTactic,
    /// Merge or split imports to the provided granularity.
    pub imports_granularity: ImportGranularity,
    /// Indent of imports.
    pub imports_indent: IndentStyle,
    /// Item layout inside a import block.
    pub imports_layout: ListTactic,
}

/// Configuration for import groups, i.e. sets of imports separated by newlines.
#[derive(Serialize, Deserialize, Debug)]
pub enum GroupImportsTactic {
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
#[derive(Serialize, Deserialize, Debug)]
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
