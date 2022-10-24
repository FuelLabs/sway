//! The main handle to an IR instance.
//!
//! [`Context`] contains several
//! [generational_arena](https://github.com/fitzgen/generational-arena) collections to maintain the
//! IR ECS.
//!
//! It is passed around as a mutable reference to many of the Sway-IR APIs.

use generational_arena::Arena;

use crate::{
    asm::AsmBlockContent,
    block::BlockContent,
    function::FunctionContent,
    irtype::AggregateContent,
    metadata::Metadatum,
    module::{ModuleContent, ModuleIterator},
    pointer::PointerContent,
    value::ValueContent,
};

/// The main IR context handle.
///
/// Every module, function, block and value is stored here.  Some aggregate metadata is also
/// managed by the context.
#[derive(Default)]
pub struct Context {
    pub(crate) modules:    Arena<ModuleContent>,
    pub(crate) functions:  Arena<FunctionContent>,
    pub(crate) blocks:     Arena<BlockContent>,
    pub(crate) values:     Arena<ValueContent>,
    pub(crate) pointers:   Arena<PointerContent>,
    pub(crate) aggregates: Arena<AggregateContent>,
    pub(crate) asm_blocks: Arena<AsmBlockContent>,
    pub(crate) metadata:   Arena<Metadatum>,

    next_unique_sym_tag: u64,
}

impl Context {
    /// Return an interator for every module in this context.
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

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", crate::printer::to_string(self))
    }
}

impl From<Context> for String {
    fn from(context: Context) -> Self {
        crate::printer::to_string(&context)
    }
}
