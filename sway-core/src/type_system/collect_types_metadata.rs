//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use crate::{type_system::TypeId, CompileResult};
use sway_types::Ident;

/// If any types contained by this node are unresolved or have yet to be inferred, throw an
/// error to signal to the user that more type information is needed.

pub enum TypeMetadata {
    UnresolvedType(Ident),
    LoggedType(TypeId),
}

pub(crate) trait CollectTypesMetadata {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>>;
}
