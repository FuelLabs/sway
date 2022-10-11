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

/// The name used within a module to refer to one of its submodules.
///
/// If an alias was given to the `dep`, this will be the alias. If not, this is the submodule's
/// library name.
pub type DepName = Ident;

/// A library module that was declared as a `dep` of another module.
///
/// Only submodules are guaranteed to be a `library` and have a `library_name`.
#[derive(Debug)]
pub struct ParseSubmodule {
    /// The name of a submodule, parsed from the `library` declaration within the module itself.
    pub library_name: Ident,
    pub module: ParseModule,
}
