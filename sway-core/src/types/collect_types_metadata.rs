//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    abi_generation::abi_str::AbiStrContext,
    language::ty::{TyExpression, TyExpressionVariant},
    type_system::TypeId,
    Engines,
};
use sha2::{Digest, Sha256};
use sway_error::{
    error::CompileError,
    formatting,
    handler::{ErrorEmitted, Handler},
};
use sway_features::ExperimentalFeatures;
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

impl TypeMetadata {
    /// Returns the [TypeId] of the actual type being logged.
    /// When the "new_encoding" is off, this is the actual type of
    /// the `logged_expr`.  E.g., when calling `__log(<logged_expr>)`,
    /// or `panic <logged_expr>;` it will be the type of the `__log`
    /// or `panic` argument, respectively. When "new_encoding" is on,
    /// it is actually the type of the argument passed to the `encode`
    /// function. In this case, when `is_new_encoding` is true, `logged_expr`
    /// must represent a [TyExpressionVariant::FunctionApplication] call to `encode`.
    pub(crate) fn get_logged_type_id(
        logged_expr: &TyExpression,
        is_new_encoding: bool,
    ) -> Result<TypeId, CompileError> {
        Self::get_logged_expression(logged_expr, is_new_encoding)
            .map(|logged_expr| logged_expr.return_type)
    }

    /// Returns the [TyExpression] that is actually being logged.
    /// When the "new_encoding" is off, this is the `logged_expr` itself.
    /// E.g., when calling `__log(<logged_expr>)`, or `panic <logged_expr>;`
    /// it will be the expression of the `__log` or `panic` argument,
    /// respectively. When "new_encoding" is on, it is the expression
    /// of the argument passed to the `encode` function.
    /// In this case, when `is_new_encoding` is true, `logged_expr`
    /// must represent a [TyExpressionVariant::FunctionApplication] call to `encode`.
    pub(crate) fn get_logged_expression(
        logged_expr: &TyExpression,
        is_new_encoding: bool,
    ) -> Result<&TyExpression, CompileError> {
        if is_new_encoding {
            match &logged_expr.expression {
                TyExpressionVariant::FunctionApplication {
                    call_path,
                    arguments,
                    ..
                } if call_path.suffix.as_str() == "encode_allow_alias" => {
                    if arguments.len() != 1 {
                        Err(CompileError::InternalOwned(
                            format!("The \"encode\" function must have exactly one argument but it had {}.", formatting::num_to_str(arguments.len())), 
                            logged_expr.span.clone(),
                        ))
                    } else {
                        match &arguments[0].1.expression {
                            TyExpressionVariant::Ref(r) => {
                                Ok(r.as_ref())
                            }
                            _ => todo!(),
                        }
                    }
                }
                _ => Err(CompileError::Internal(
                        "In case of the new encoding, the \"logged_expr\" must be a call to an \"encode_allow_alias\" function.",
                        logged_expr.span.clone()
                    ))
            }
        } else {
            Ok(logged_expr)
        }
    }

    pub(crate) fn new_logged_type(
        handler: &Handler,
        engines: &Engines,
        type_id: TypeId,
        program_name: String,
    ) -> Result<Self, ErrorEmitted> {
        Ok(TypeMetadata::LoggedType(
            LogId::new(type_id.get_abi_type_str(
                handler,
                &AbiStrContext {
                    program_name,
                    abi_with_callpaths: true,
                    abi_with_fully_specified_types: true,
                    abi_root_type_without_generic_type_parameters: false,
                },
                engines,
                type_id,
            )?),
            type_id,
        ))
    }
}

// A simple context that only contains a single counter for now but may expand in the future.
pub struct CollectTypesMetadataContext<'cx> {
    // Consume this and update it via the methods implemented for CollectTypesMetadataContext to
    // obtain a unique ID for a given smo instance.
    message_id_counter: usize,

    call_site_spans: Vec<Arc<Mutex<HashMap<TypeId, Span>>>>,
    pub(crate) engines: &'cx Engines,

    pub(crate) program_name: String,

    pub experimental: ExperimentalFeatures,
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
        experimental: ExperimentalFeatures,
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
