//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use crate::error::*;

/// If any types contained by this node are unresolved or have yet to be inferred, throw an
/// error to signal to the user that more type information is needed.
pub(crate) trait UnresolvedTypeCheck {
    fn check_for_unresolved_types(&self) -> Vec<CompileError>;
}
