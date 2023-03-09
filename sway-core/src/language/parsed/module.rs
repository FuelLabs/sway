use crate::{language::ModName, transform};

use super::ParseTree;
use sway_types::Span;

/// A module and its submodules in the form of a tree.
#[derive(Debug, Clone)]
pub struct ParseModule {
    /// The content of this module in the form of a `ParseTree`.
    pub tree: ParseTree,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(ModName, ParseSubmodule)>,
    pub attributes: transform::AttributesMap,
    /// an empty span at the beginning of the file containing the module
    pub span: Span,
}

/// A library module that was declared as a `mod` of another module.
///
/// Only submodules are guaranteed to be a `library`.
#[derive(Debug, Clone)]
pub struct ParseSubmodule {
    pub module: ParseModule,
    pub mod_name_span: Span,
}
