use super::LexedModule;
use crate::language::parsed::TreeType;

/// A parsed, but not yet type-checked, Sway program.
///
/// Includes all modules in the form of a `ParseModule` tree accessed via the `root`.
#[derive(Debug, Clone)]
pub struct LexedProgram {
    pub kind: TreeType,
    pub root: LexedModule,
}

impl LexedProgram {
    pub fn new(kind: TreeType, root: LexedModule) -> LexedProgram {
        LexedProgram { kind, root }
    }
}
