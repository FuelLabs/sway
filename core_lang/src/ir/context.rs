use std::collections::HashMap;

use generational_arena::Arena;

use super::*;

pub(crate) struct Context {
    pub(crate) modules: Arena<ModuleContent>,
    pub(crate) functions: Arena<FunctionContent>,
    pub(crate) blocks: Arena<BlockContent>,
    pub(crate) values: Arena<ValueContent>,
    pub(crate) pointers: Arena<PointerContent>,
    pub(crate) aggregates: Arena<AggregateContent>,
    pub(crate) asm_blocks: Arena<AsmBlockContent>,

    pub(super) aggregate_names: HashMap<String, Aggregate>,
    aggregate_symbols: HashMap<Aggregate, HashMap<String, u64>>,

    next_unique_sym_tag: u64,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context {
            modules: Arena::new(),
            functions: Arena::new(),
            blocks: Arena::new(),
            values: Arena::new(),
            pointers: Arena::new(),
            aggregates: Arena::new(),
            asm_blocks: Arena::new(),

            aggregate_names: HashMap::new(),
            aggregate_symbols: HashMap::new(),

            next_unique_sym_tag: 0,
        }
    }

    pub(crate) fn module_iter(&self) -> ModuleIterator {
        ModuleIterator::new(self)
    }

    pub(crate) fn add_aggregate_symbols(
        &mut self,
        aggregate: Aggregate,
        symbols: HashMap<String, u64>,
    ) -> Result<(), String> {
        match self.aggregate_symbols.insert(aggregate, symbols) {
            None => Ok(()),
            Some(_) => Err("Aggregate symbols were overwritten/shadowed.".into()),
        }
    }

    pub(crate) fn get_aggregate_by_name(&self, name: &str) -> Option<Aggregate> {
        self.aggregate_names.get(name).copied()
    }

    pub(crate) fn get_aggregate_index(
        &self,
        aggregate: &Aggregate,
        field_name: &str,
    ) -> Option<u64> {
        self.aggregate_symbols
            .get(aggregate)
            .map(|idx_map| idx_map.get(field_name).copied())
            .flatten()
    }

    pub(crate) fn get_unique_name(&mut self) -> String {
        let sym = format!("anon_{}", self.next_unique_sym_tag);
        self.next_unique_sym_tag += 1;
        sym
    }
}
