mod program;

use crate::language::DepName;
pub use program::LexedProgram;
use sway_ast::Module;
use sway_types::Ident;

/// A module and its submodules in the form of a tree.
#[derive(Debug, Clone)]
pub struct LexedModule {
    /// The content of this module in the form of a `ParseTree`.
    pub module: Module,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(DepName, LexedSubmodule)>,
}

/// A library module that was declared as a `dep` of another module.
///
/// Only submodules are guaranteed to be a `library` and have a `library_name`.
#[derive(Debug, Clone)]
pub struct LexedSubmodule {
    /// The name of a submodule, parsed from the `library` declaration within the module itself.
    pub library_name: Ident,
    pub module: LexedModule,
}
