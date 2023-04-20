mod program;

use crate::language::ModName;
pub use program::LexedProgram;
use sway_ast::Module;

/// A module and its submodules in the form of a tree.
#[derive(Debug, Clone)]
pub struct LexedModule {
    /// The content of this module in the form of a [Module].
    pub tree: Module,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(ModName, LexedSubmodule)>,
}

/// A library module that was declared as a `mod` of another module.
///
/// Only submodules are guaranteed to be a `library`.
#[derive(Debug, Clone)]
pub struct LexedSubmodule {
    pub module: LexedModule,
}
