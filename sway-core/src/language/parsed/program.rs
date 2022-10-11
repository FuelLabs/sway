use super::ParseModule;
use sway_types::Ident;

/// A parsed, but not yet type-checked, Sway program.
///
/// Includes all modules in the form of a `ParseModule` tree accessed via the `root`.
#[derive(Debug)]
pub struct ParseProgram {
    pub kind: TreeType,
    pub root: ParseModule,
}

/// A Sway program can be either a contract, script, predicate, or a library.
///
/// All submodules declared with `dep` should be `Library`s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeType {
    Predicate,
    Script,
    Contract,
    Library { name: Ident },
}

impl std::fmt::Display for TreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Predicate => "predicate",
                Self::Script => "script",
                Self::Contract => "contract",
                Self::Library { .. } => "library",
            }
        )
    }
}
