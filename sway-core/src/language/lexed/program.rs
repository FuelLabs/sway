use super::LexedModule;
use crate::language::parsed::TreeType;

/// A lexed, but not yet parsed or type-checked, Sway program.
///
/// Includes all modules in the form of a [LexedModule] tree accessed via the `root`.
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
