//! A value representing a configurable. Every `configurable` field in the program
//! corresponds to a [Config].

use std::cell::Cell;

use crate::{context::Context, pretty::DebugWithContext, Constant, Function, MetadataIndex, Type};

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Config(#[in_context(configs)] pub slotmap::DefaultKey);

#[doc(hidden)]
#[derive(Clone, Debug, DebugWithContext)]
pub enum ConfigContent {
    V0 {
        name: String,
        ty: Type,
        ptr_ty: Type,
        constant: Constant,
        opt_metadata: Option<MetadataIndex>,
    },
    V1 {
        name: String,
        ty: Type,
        ptr_ty: Type,
        encoded_bytes: Vec<u8>,
        decode_fn: Cell<Option<Function>>, // `None` when the configurable's type is trivially decodable
        opt_metadata: Option<MetadataIndex>,
    },
}

impl Config {
    /// Insert a new configurable with the given `content` into the `context` and return its handle.
    pub fn new(context: &mut Context, content: ConfigContent) -> Self {
        Config(context.configs.insert(content))
    }

    /// Return the [ConfigContent] that this [Config] refers to.
    pub fn get_content<'a>(&self, context: &'a Context) -> &'a ConfigContent {
        &context.configs[self.0]
    }

    /// Return the configurable pointer type.
    pub fn get_type(&self, context: &Context) -> Type {
        match &context.configs[self.0] {
            ConfigContent::V0 { ptr_ty, .. } | ConfigContent::V1 { ptr_ty, .. } => *ptr_ty,
        }
    }

    /// Return the configurable name.
    pub fn get_name<'a>(&self, context: &'a Context) -> &'a str {
        match &context.configs[self.0] {
            ConfigContent::V0 { name, .. } | ConfigContent::V1 { name, .. } => name,
        }
    }
}
