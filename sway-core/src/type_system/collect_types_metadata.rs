//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    declaration_engine::DeclarationEngine, type_system::TypeId, CompileResult, TypeEngine,
};
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
pub struct CollectTypesMetadataContext<'cx> {
    // Consume this and update it via the methods implemented for CollectTypesMetadataContext to
    // obtain a unique ID for a given log instance.
    log_id_counter: usize,

    call_site_spans: Vec<Arc<Mutex<HashMap<TypeId, Span>>>>,
    pub(crate) type_engine: &'cx TypeEngine,
    pub(crate) declaration_engine: &'cx DeclarationEngine,
}

impl<'a> CollectTypesMetadataContext<'a> {
    pub fn log_id_counter(&self) -> usize {
        self.log_id_counter
    }

    pub fn log_id_counter_mut(&mut self) -> &mut usize {
        &mut self.log_id_counter
    }

    pub fn call_site_push(&mut self) {
        self.call_site_spans
            .push(Arc::new(Mutex::new(HashMap::new())));
    }

    pub fn call_site_pop(&mut self) {
        self.call_site_spans.pop();
    }

    pub fn call_site_insert(&mut self, type_id: TypeId, span: Span) {
        self.call_site_spans
            .last()
            .and_then(|h| h.lock().ok())
            .and_then(|mut l| l.insert(type_id, span));
    }

    pub fn call_site_get(&mut self, type_id: &TypeId) -> Option<Span> {
        for lock in self.call_site_spans.iter() {
            if let Ok(hash_map) = lock.lock() {
                let opt = hash_map.get(type_id).cloned();
                if opt.is_some() {
                    return opt;
                }
            }
        }
        None
    }

    pub fn new(type_engine: &'a TypeEngine, declaration_engine: &'a DeclarationEngine) -> Self {
        let mut ctx = Self {
            type_engine,
            declaration_engine,
            log_id_counter: 0,
            call_site_spans: vec![],
        };
        ctx.call_site_push();
        ctx
    }
}

pub(crate) trait CollectTypesMetadata {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>>;
}
