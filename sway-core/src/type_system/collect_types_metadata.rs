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
    LoggedType(TypeId, usize),
}

pub struct CollectTypesMetadataContext {
    log_id: usize,
}

impl CollectTypesMetadataContext {
    pub fn log_id(&self) -> usize {
        self.log_id
    }

    pub fn log_id_mut(&mut self) -> &mut usize {
        &mut self.log_id
    }

    pub fn new() -> Self {
        Self { log_id: 0 }
    }
}

pub(crate) trait CollectTypesMetadata {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>>;
}
