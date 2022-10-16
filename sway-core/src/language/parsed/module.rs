use crate::language::DepName;

use super::ParseTree;
use sway_types::Ident;

/// A module and its submodules in the form of a tree.
#[derive(Debug)]
pub struct ParseModule {
    /// The content of this module in the form of a `ParseTree`.
    pub tree: ParseTree,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(DepName, ParseSubmodule)>,
}

/// A library module that was declared as a `dep` of another module.
///
/// Only submodules are guaranteed to be a `library` and have a `library_name`.
#[derive(Debug)]
pub struct ParseSubmodule {
    /// The name of a submodule, parsed from the `library` declaration within the module itself.
    pub library_name: Ident,
    pub module: ParseModule,
}
