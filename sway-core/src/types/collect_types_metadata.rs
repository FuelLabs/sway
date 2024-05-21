//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{type_system::TypeId, Engines, ExperimentalFlags};
use sha2::{Digest, Sha256};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span};

/// If any types contained by this node are unresolved or have yet to be inferred, throw an
/// error to signal to the user that more type information is needed.

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct LogId {
    pub hash_id: u64,
}

impl LogId {
    pub fn new(string: String) -> LogId {
        let mut hasher = Sha256::new();
        hasher.update(string);
        let result = hasher.finalize();
        let hash_id = u64::from_be_bytes(result[0..8].try_into().unwrap());
        LogId { hash_id }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct MessageId(usize);

impl std::ops::Deref for MessageId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageId {
    pub fn new(index: usize) -> MessageId {
        MessageId(index)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum TypeMetadata {
    // UnresolvedType receives the Ident of the type and a call site span.
    UnresolvedType(Ident, Option<Span>),
    // A log with a unique log ID and the type ID of the type of the value being logged
    LoggedType(LogId, TypeId),
    // An smo with a unique message ID and the type ID of the type of the message data being sent
    MessageType(MessageId, TypeId),
}

// A simple context that only contains a single counter for now but may expand in the future.
pub struct CollectTypesMetadataContext<'cx> {
    // Consume this and update it via the methods implemented for CollectTypesMetadataContext to
    // obtain a unique ID for a given smo instance.
    message_id_counter: usize,

    call_site_spans: Vec<Arc<Mutex<HashMap<TypeId, Span>>>>,
    pub(crate) engines: &'cx Engines,

    pub(crate) program_name: String,

    pub experimental: ExperimentalFlags,
}

impl<'cx> CollectTypesMetadataContext<'cx> {
    pub fn message_id_counter(&self) -> usize {
        self.message_id_counter
    }

    pub fn message_id_counter_mut(&mut self) -> &mut usize {
        &mut self.message_id_counter
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
        for lock in &self.call_site_spans {
            if let Ok(hash_map) = lock.lock() {
                let opt = hash_map.get(type_id).cloned();
                if opt.is_some() {
                    return opt;
                }
            }
        }
        None
    }

    pub fn new(
        engines: &'cx Engines,
        experimental: ExperimentalFlags,
        program_name: String,
    ) -> Self {
        let mut ctx = Self {
            engines,
            message_id_counter: 0,
            call_site_spans: vec![],
            experimental,
            program_name,
        };
        ctx.call_site_push();
        ctx
    }
}

pub(crate) trait CollectTypesMetadata {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted>;
}
