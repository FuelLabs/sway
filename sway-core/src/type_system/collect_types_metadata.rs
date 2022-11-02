//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use crate::{type_system::TypeId, CompileResult};
use sway_types::{Ident, Span};

/// If any types contained by this node are unresolved or have yet to be inferred, throw an
/// error to signal to the user that more type information is needed.

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct LogId(usize);

impl std::ops::Deref for LogId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl LogId {
    pub fn new(index: usize) -> LogId {
        LogId(index)
    }
}

pub enum TypeMetadata {
    // UnresolvedType receives the Ident of the type and a call site span.
    UnresolvedType(Ident, Option<Span>),
    // A log with a unique log ID and the type ID of the type of the value being logged
    LoggedType(LogId, TypeId),
}

// A simple context that only contains a single counter for now but may expand in the future.
pub struct CollectTypesMetadataContext {
    // Consume this and update it via the methods implemented for CollectTypesMetadataContext to
    // obtain a unique ID for a given log instance.
    log_id_counter: usize,

    call_site_span: Option<Span>,
}

impl CollectTypesMetadataContext {
    pub fn log_id_counter(&self) -> usize {
        self.log_id_counter
    }

    pub fn log_id_counter_mut(&mut self) -> &mut usize {
        &mut self.log_id_counter
    }

    pub fn call_site_span(&self) -> Option<Span> {
        self.call_site_span.clone()
    }

    pub fn set_call_site_span(&mut self, span: Option<Span>) {
        self.call_site_span = span;
    }

    pub fn new() -> Self {
        Self {
            log_id_counter: 0,
            call_site_span: None,
        }
    }
}

pub(crate) trait CollectTypesMetadata {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>>;
}
