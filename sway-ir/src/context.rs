//! The main handle to an IR instance.
//!
//! [`Context`] contains several
//! [slotmap](https://github.com/orlp/slotmap) collections to maintain the
//! IR ECS.
//!
//! It is passed around as a mutable reference to many of the Sway-IR APIs.

use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SlotMap};
use sway_types::SourceEngine;

use crate::{
    block::BlockContent, function::FunctionContent, local_var::LocalVarContent,
    metadata::Metadatum, module::Kind, module::ModuleContent, module::ModuleIterator,
    value::ValueContent, Type, TypeContent,
};

/// The main IR context handle.
///
/// Every module, function, block and value is stored here.  Some aggregate metadata is also
/// managed by the context.
pub struct Context<'eng> {
    pub source_engine: &'eng SourceEngine,

    pub(crate) modules: SlotMap<DefaultKey, ModuleContent>,
    pub(crate) functions: SlotMap<DefaultKey, FunctionContent>,
    pub(crate) blocks: SlotMap<DefaultKey, BlockContent>,
    pub(crate) values: SlotMap<DefaultKey, ValueContent>,
    pub(crate) local_vars: SlotMap<DefaultKey, LocalVarContent>,
    pub(crate) types: SlotMap<DefaultKey, TypeContent>,
    pub(crate) type_map: FxHashMap<TypeContent, Type>,
    pub(crate) metadata: SlotMap<DefaultKey, Metadatum>,

    pub program_kind: Kind,

    next_unique_sym_tag: u64,

    pub experimental: ExperimentalFlags,
}

#[derive(Copy, Clone)]
pub struct ExperimentalFlags {
    pub new_encoding: bool,
}

impl<'eng> Context<'eng> {
    pub fn new(source_engine: &'eng SourceEngine, experimental: ExperimentalFlags) -> Self {
        let mut def = Self {
            source_engine,
            modules: Default::default(),
            functions: Default::default(),
            blocks: Default::default(),
            values: Default::default(),
            local_vars: Default::default(),
            types: Default::default(),
            type_map: Default::default(),
            metadata: Default::default(),
            next_unique_sym_tag: Default::default(),
            program_kind: Kind::Contract,
            experimental,
        };
        Type::create_basic_types(&mut def);
        def
    }

    pub fn source_engine(&self) -> &'eng SourceEngine {
        self.source_engine
    }

    /// Return an iterator for every module in this context.
    pub fn module_iter(&self) -> ModuleIterator {
        ModuleIterator::new(self)
    }

    /// Get a globally unique symbol.
    ///
    /// The name will be in the form `"anon_N"`, where `N` is an incrementing decimal.
    pub fn get_unique_name(&mut self) -> String {
        format!("anon_{}", self.get_unique_id())
    }

    /// Get a globally unique symbol id.
    pub fn get_unique_id(&mut self) -> u64 {
        let sym = self.next_unique_sym_tag;
        self.next_unique_sym_tag += 1;
        sym
    }
}

use std::fmt::{Display, Error, Formatter};

impl Display for Context<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", crate::printer::to_string(self))
    }
}

impl From<Context<'_>> for String {
    fn from(context: Context) -> Self {
        crate::printer::to_string(&context)
    }
}
