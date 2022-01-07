use std::collections::{BTreeMap, HashMap};

use crate::{
    block::{Block, BlockIterator, Label},
    constant::Constant,
    context::Context,
    irtype::Type,
    module::Module,
    pointer::{Pointer, PointerContent},
    value::Value,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Function(pub generational_arena::Index);

pub struct FunctionContent {
    pub name: String,
    pub arguments: Vec<(String, Value)>,
    pub return_type: Type,
    pub blocks: Vec<Block>,
    pub is_public: bool,
    pub selector: Option<[u8; 4]>,

    pub local_storage: BTreeMap<String, Pointer>, // BTree rather than Hash for deterministic ordering.

    next_label_idx: u64,
}

impl Function {
    pub fn new(
        context: &mut Context,
        module: Module,
        name: String,
        args: Vec<(String, Type)>,
        return_type: Type,
        selector: Option<[u8; 4]>,
        is_public: bool,
    ) -> Function {
        let arguments = args
            .into_iter()
            .map(|(name, ty)| (name, Value::new_argument(context, ty)))
            .collect();
        let content = FunctionContent {
            name,
            arguments,
            return_type,
            blocks: Vec::new(),
            is_public,
            selector,
            local_storage: BTreeMap::new(),
            next_label_idx: 0,
        };
        let func = Function(context.functions.insert(content));

        context.modules[module.0].functions.push(func);

        let entry_block = Block::new(context, func, Some("entry".to_owned()));
        context
            .functions
            .get_mut(func.0)
            .unwrap()
            .blocks
            .push(entry_block);

        func
    }

    pub fn create_block(&self, context: &mut Context, label: Option<Label>) -> Block {
        let block = Block::new(context, *self, label);
        let func = context.functions.get_mut(self.0).unwrap();
        func.blocks.push(block);
        block
    }

    pub fn create_block_before(
        &self,
        context: &mut Context,
        other: &Block,
        label: Option<Label>,
    ) -> Result<Block, String> {
        // We need to create the new block first (even though we may not use it on Err below) since
        // we can't borrow context mutably twice.
        let new_block = Block::new(context, *self, label);
        let func = context.functions.get_mut(self.0).unwrap();
        func.blocks
            .iter()
            .position(|block| block == other)
            .map(|idx| {
                func.blocks.insert(idx, new_block);
                new_block
            })
            .ok_or_else(|| "Cannot insert block before other, not found in function.".into())
    }

    pub fn create_block_after(
        &self,
        context: &mut Context,
        other: &Block,
        label: Option<Label>,
    ) -> Result<Block, String> {
        // We need to create the new block first (even though we may not use it on Err below) since
        // we can't borrow context mutably twice.
        let new_block = Block::new(context, *self, label);
        let func = context.functions.get_mut(self.0).unwrap();
        func.blocks
            .iter()
            .position(|block| block == other)
            .map(|idx| {
                func.blocks.insert(idx + 1, new_block);
                new_block
            })
            .ok_or_else(|| "Cannot insert block after other, not found in function.".into())
    }

    pub fn get_unique_label(&self, context: &mut Context, hint: Option<String>) -> String {
        match hint {
            Some(hint) => {
                if context.functions[self.0]
                    .blocks
                    .iter()
                    .any(|block| context.blocks[block.0].label == hint)
                {
                    let idx = self.get_next_label_idx(context);
                    self.get_unique_label(context, Some(format!("{}{}", hint, idx)))
                } else {
                    hint
                }
            }
            None => {
                let idx = self.get_next_label_idx(context);
                self.get_unique_label(context, Some(format!("block{}", idx)))
            }
        }
    }

    fn get_next_label_idx(&self, context: &mut Context) -> u64 {
        let func = context.functions.get_mut(self.0).unwrap();
        let idx = func.next_label_idx;
        func.next_label_idx += 1;
        idx
    }

    pub fn get_name<'a>(&self, context: &'a Context) -> &'a str {
        &context.functions[self.0].name
    }

    pub fn get_entry_block(&self, context: &Context) -> Block {
        context.functions[self.0].blocks[0]
    }

    pub fn has_selector(&self, context: &Context) -> bool {
        context.functions[self.0].selector.is_some()
    }

    pub fn get_selector(&self, context: &Context) -> Option<[u8; 4]> {
        context.functions[self.0].selector
    }

    pub fn get_arg(&self, context: &Context, name: &str) -> Option<Value> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(arg_name, val)| if arg_name == name { Some(val) } else { None })
            .copied()
    }

    pub fn lookup_arg_name<'a>(&self, context: &'a Context, value: &Value) -> Option<&'a String> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(name, arg_val)| if arg_val == value { Some(name) } else { None })
    }

    pub fn args_iter<'a>(&self, context: &'a Context) -> impl Iterator<Item = &'a (String, Value)> {
        context.functions[self.0].arguments.iter()
    }

    pub fn get_local_ptr(&self, context: &Context, name: &str) -> Option<Pointer> {
        context.functions[self.0].local_storage.get(name).copied()
    }

    pub fn lookup_local_name<'a>(&self, context: &'a Context, ptr: &Pointer) -> Option<&'a String> {
        context.functions[self.0]
            .local_storage
            .iter()
            .find_map(|(name, local_ptr)| if local_ptr == ptr { Some(name) } else { None })
    }

    pub fn new_local_ptr(
        &self,
        context: &mut Context,
        name: String,
        local_type: Type,
        is_mutable: bool,
        initializer: Option<Constant>,
    ) -> Result<Pointer, String> {
        let ptr = Pointer::new(context, local_type, is_mutable, initializer);
        let func = context.functions.get_mut(self.0).unwrap();
        if func.local_storage.insert(name.clone(), ptr).is_some() {
            Err(format!(
                "Local storage for function {} already has entry for {}.",
                func.name, name
            ))
        } else {
            Ok(ptr)
        }
    }

    // Will use the provided name as a hint and rename to guarantee insertion.
    pub fn new_unique_local_ptr(
        &self,
        context: &mut Context,
        name: String,
        local_type: Type,
        is_mutable: bool,
        initializer: Option<Constant>,
    ) -> Pointer {
        let func = &context.functions[self.0];
        let new_name = if func.local_storage.contains_key(&name) {
            // Assuming that we'll eventually find a unique name by appending numbers to the old
            // one...
            (0..)
                .find_map(|n| {
                    let candidate = format!("{}{}", name, n);
                    if func.local_storage.contains_key(&candidate) {
                        None
                    } else {
                        Some(candidate)
                    }
                })
                .unwrap()
        } else {
            name
        };
        self.new_local_ptr(context, new_name, local_type, is_mutable, initializer)
            .unwrap()
    }

    pub fn locals_iter<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = (&'a String, &'a Pointer)> {
        context.functions[self.0].local_storage.iter()
    }

    pub fn merge_locals_from(
        &self,
        context: &mut Context,
        other: Function,
    ) -> Result<HashMap<Pointer, Pointer>, String> {
        let mut ptr_map = HashMap::new();
        let old_ptrs: Vec<(String, Pointer, PointerContent)> = context.functions[other.0]
            .local_storage
            .iter()
            .map(|(name, ptr)| (name.clone(), *ptr, context.pointers[ptr.0].clone()))
            .collect();
        for (name, old_ptr, old_ptr_content) in old_ptrs {
            let new_ptr = self.new_unique_local_ptr(
                context,
                name.clone(),
                old_ptr_content.ty,
                old_ptr_content.is_mutable,
                old_ptr_content.initializer,
            );
            ptr_map.insert(old_ptr, new_ptr);
        }
        Ok(ptr_map)
    }

    pub fn block_iter(&self, context: &Context) -> BlockIterator {
        BlockIterator::new(context, self)
    }

    pub fn instruction_iter<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = (Block, Value)> + 'a {
        context.functions[self.0]
            .blocks
            .iter()
            .flat_map(move |block| {
                context.blocks[block.0]
                    .instructions
                    .iter()
                    .map(move |ins_val| (*block, *ins_val))
            })
    }

    pub fn replace_value(
        &self,
        context: &mut Context,
        old_val: Value,
        new_val: Value,
        starting_block: Option<Block>,
    ) {
        let mut block_iter = self.block_iter(context).peekable();

        if let Some(ref starting_block) = starting_block {
            // Skip blocks until we hit the starting block.
            while block_iter
                .next_if(|block| block != starting_block)
                .is_some()
            {}
        }

        for block in block_iter {
            block.replace_value(context, old_val, new_val);
        }
    }
}

pub struct FunctionIterator {
    functions: Vec<generational_arena::Index>,
    next: usize,
}

impl FunctionIterator {
    pub fn new(context: &Context, module: &Module) -> FunctionIterator {
        // Copy all the current modules indices, so they may be modified in the context during
        // iteration.
        FunctionIterator {
            functions: context.modules[module.0]
                .functions
                .iter()
                .map(|func| func.0)
                .collect(),
            next: 0,
        }
    }
}

impl Iterator for FunctionIterator {
    type Item = Function;

    fn next(&mut self) -> Option<Function> {
        if self.next < self.functions.len() {
            let idx = self.next;
            self.next += 1;
            Some(Function(self.functions[idx]))
        } else {
            None
        }
    }
}
