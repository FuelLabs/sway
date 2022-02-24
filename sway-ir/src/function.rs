//! A typical function data type.
//!
//! [`Function`] is named, takes zero or more arguments and has an optional return value.  It
//! contains a collection of [`Block`]s.
//!
//! It also maintains a collection of local values which can be typically regarded as variables
//! existing in the function scope.

use std::collections::{BTreeMap, HashMap};

use crate::{
    block::{Block, BlockIterator, Label},
    constant::Constant,
    context::Context,
    error::IrError,
    irtype::Type,
    metadata::MetadataIndex,
    module::Module,
    pointer::{Pointer, PointerContent},
    value::Value,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Function(pub generational_arena::Index);

#[doc(hidden)]
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
    /// Return a new [`Function`] handle.
    ///
    /// Creates a [`Function`] in the `context` within `module` and returns a handle.
    ///
    /// `name`, `args`, `return_type` and `is_public` are the usual suspects.  `selector` is a
    /// special value used for Sway contract calls; much like `name` is unique and not particularly
    /// used elsewhere in the IR.
    pub fn new(
        context: &mut Context,
        module: Module,
        name: String,
        args: Vec<(String, Type, Option<MetadataIndex>)>,
        return_type: Type,
        selector: Option<[u8; 4]>,
        is_public: bool,
    ) -> Function {
        let arguments = args
            .into_iter()
            .map(|(name, ty, span_md_idx)| (name, Value::new_argument(context, ty, span_md_idx)))
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

    /// Create and append a new [`Block`] to this function.
    pub fn create_block(&self, context: &mut Context, label: Option<Label>) -> Block {
        let block = Block::new(context, *self, label);
        let func = context.functions.get_mut(self.0).unwrap();
        func.blocks.push(block);
        block
    }

    /// Create and insert a new [`Block`] into this function.
    ///
    /// The new block is inserted before `other`.
    pub fn create_block_before(
        &self,
        context: &mut Context,
        other: &Block,
        label: Option<Label>,
    ) -> Result<Block, IrError> {
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
            .ok_or_else(|| {
                let label = &context.blocks[other.0].label;
                IrError::MissingBlock(label.clone())
            })
    }

    /// Create and insert a new [`Block`] into this function.
    ///
    /// The new block is inserted after `other`.
    pub fn create_block_after(
        &self,
        context: &mut Context,
        other: &Block,
        label: Option<Label>,
    ) -> Result<Block, IrError> {
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
            .ok_or_else(|| {
                let label = &context.blocks[other.0].label;
                IrError::MissingBlock(label.clone())
            })
    }

    /// Get a new unique block label.
    ///
    /// If `hint` is `None` then the label will be in the form `"blockN"` where N is an
    /// incrementing decimal.
    ///
    /// Otherwise if the hint is already unique to this function it will be returned.  If not
    /// already unique it will have N appended to it until it is unique.
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

    /// Return the function name.
    pub fn get_name<'a>(&self, context: &'a Context) -> &'a str {
        &context.functions[self.0].name
    }

    /// Return the function entry (i.e., the first) block.
    pub fn get_entry_block(&self, context: &Context) -> Block {
        context.functions[self.0].blocks[0]
    }

    /// Whether this function has a valid selector.
    pub fn has_selector(&self, context: &Context) -> bool {
        context.functions[self.0].selector.is_some()
    }

    /// Return the function selector, if it has one.
    pub fn get_selector(&self, context: &Context) -> Option<[u8; 4]> {
        context.functions[self.0].selector
    }

    /// Get an arg value by name, if found.
    pub fn get_arg(&self, context: &Context, name: &str) -> Option<Value> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(arg_name, val)| if arg_name == name { Some(val) } else { None })
            .copied()
    }

    /// Find the name of an arg by value.
    pub fn lookup_arg_name<'a>(&self, context: &'a Context, value: &Value) -> Option<&'a String> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(name, arg_val)| if arg_val == value { Some(name) } else { None })
    }

    /// Return an iterator for each of the function arguments.
    pub fn args_iter<'a>(&self, context: &'a Context) -> impl Iterator<Item = &'a (String, Value)> {
        context.functions[self.0].arguments.iter()
    }

    /// Get a pointer to a local value by name, if found.
    pub fn get_local_ptr(&self, context: &Context, name: &str) -> Option<Pointer> {
        context.functions[self.0].local_storage.get(name).copied()
    }

    /// Find the name of a local value by pointer.
    pub fn lookup_local_name<'a>(&self, context: &'a Context, ptr: &Pointer) -> Option<&'a String> {
        context.functions[self.0]
            .local_storage
            .iter()
            .find_map(|(name, local_ptr)| if local_ptr == ptr { Some(name) } else { None })
    }

    /// Add a value to the function local storage.
    ///
    /// The name must be unique to this function else an error is returned.
    pub fn new_local_ptr(
        &self,
        context: &mut Context,
        name: String,
        local_type: Type,
        is_mutable: bool,
        initializer: Option<Constant>,
    ) -> Result<Pointer, IrError> {
        let ptr = Pointer::new(context, local_type, is_mutable, initializer);
        let func = context.functions.get_mut(self.0).unwrap();
        func.local_storage
            .insert(name.clone(), ptr)
            .map(|_| Err(IrError::FunctionLocalClobbered(func.name.clone(), name)))
            .unwrap_or(Ok(ptr))
    }

    /// Add a value to the function local storage, by forcing the name to be unique if needed.
    ///
    /// Will use the provided name as a hint and rename to guarantee insertion.
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

    /// Return an iterator to all of the values in this function's local storage.
    pub fn locals_iter<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = (&'a String, &'a Pointer)> {
        context.functions[self.0].local_storage.iter()
    }

    /// Merge values from another [`Function`] into this one.
    ///
    /// The names of the merged values are guaranteed to be unique via the use of
    /// [`Function::new_unique_local_ptr`].
    ///
    /// Returns a map from the original pointers to the newly merged pointers.
    ///
    /// XXX This function returns a Result but can't actually fail?
    pub fn merge_locals_from(
        &self,
        context: &mut Context,
        other: Function,
    ) -> Result<HashMap<Pointer, Pointer>, IrError> {
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

    /// Return an iterator to each block in this function.
    pub fn block_iter(&self, context: &Context) -> BlockIterator {
        BlockIterator::new(context, self)
    }

    /// Return an iterator to each instruction in each block in this function.
    ///
    /// This is a convenience method for when all instructions in a function need to be inspected.
    /// The instruction value is returned from the iterator along with the block it belongs to.
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

    /// Replace a value with another within this function.
    ///
    /// This is a convenience method which iterates over this function's blocks and calls
    /// [`Block::replace_value`] in turn.
    ///
    /// `starting_block` is an optimisation for when the first possible reference to `old_val` is
    /// known.
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

/// An iterator over each [`Function`] in a [`Module`].
pub struct FunctionIterator {
    functions: Vec<generational_arena::Index>,
    next: usize,
}

impl FunctionIterator {
    /// Return a new iterator for the functions in `module`.
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
