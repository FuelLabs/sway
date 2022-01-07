use std::collections::HashMap;

use generational_arena::Arena;

use crate::{
    asm::AsmBlockContent,
    block::BlockContent,
    function::FunctionContent,
    irtype::{AbiInstanceContent, Aggregate, AggregateContent},
    module::ModuleContent,
    module::ModuleIterator,
    pointer::PointerContent,
    value::ValueContent,
};

#[derive(Default)]
pub struct Context {
    pub modules: Arena<ModuleContent>,
    pub functions: Arena<FunctionContent>,
    pub blocks: Arena<BlockContent>,
    pub values: Arena<ValueContent>,
    pub pointers: Arena<PointerContent>,
    pub aggregates: Arena<AggregateContent>,
    pub abi_instances: Arena<AbiInstanceContent>,
    pub asm_blocks: Arena<AsmBlockContent>,

    pub(super) aggregate_names: HashMap<String, Aggregate>,
    aggregate_symbols: HashMap<Aggregate, HashMap<String, u64>>,

    next_unique_sym_tag: u64,
}

impl Context {
    pub fn module_iter(&self) -> ModuleIterator {
        ModuleIterator::new(self)
    }

    pub fn add_aggregate_symbols(
        &mut self,
        aggregate: Aggregate,
        symbols: HashMap<String, u64>,
    ) -> Result<(), String> {
        match self.aggregate_symbols.insert(aggregate, symbols) {
            None => Ok(()),
            Some(_) => Err("Aggregate symbols were overwritten/shadowed.".into()),
        }
    }

    pub fn get_aggregate_by_name(&self, name: &str) -> Option<Aggregate> {
        self.aggregate_names.get(name).copied()
    }

    pub fn get_aggregate_index(&self, aggregate: &Aggregate, field_name: &str) -> Option<u64> {
        self.aggregate_symbols
            .get(aggregate)
            .map(|idx_map| idx_map.get(field_name).copied())
            .flatten()
    }

    pub fn get_unique_name(&mut self) -> String {
        let sym = format!("anon_{}", self.next_unique_sym_tag);
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
