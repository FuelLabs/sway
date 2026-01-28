//! A typical function data type.
//!
//! [`Function`] is named, takes zero or more arguments and has an optional return value.  It
//! contains a collection of [`Block`]s.
//!
//! It also maintains a collection of local values which can be typically regarded as variables
//! existing in the function scope.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    block::{Block, BlockIterator, Label},
    context::Context,
    error::IrError,
    irtype::Type,
    metadata::MetadataIndex,
    module::Module,
    value::{Value, ValueDatum},
    variable::{LocalVar, LocalVarContent},
    BlockArgument, BranchToWithArgs,
};
use crate::{Constant, InstOp};

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Function(pub slotmap::DefaultKey);

#[doc(hidden)]
pub struct FunctionContent {
    pub name: String,
    /// Display string representing the function in the ABI errors
    /// related context (in "errorCodes" and "panickingCalls" sections).
    // TODO: Explore how and if we should lazy evaluate `abi_errors_display`,
    //       only for functions that are actually used in ABI errors context.
    //       Having it precomputed for every function is a simple design.
    //       Lazy evaluation might be much more complex to implement and
    //       a premature optimization, considering that even for large
    //       project we compile <1500 functions.
    pub abi_errors_display: String,
    pub arguments: Vec<(String, Value)>,
    pub return_type: Type,
    pub blocks: Vec<Block>,
    pub module: Module,
    pub is_public: bool,
    pub is_entry: bool,
    /// True if the function was an entry, before getting wrapped
    /// by the `__entry` function. E.g, a script `main` function.
    pub is_original_entry: bool,
    pub is_fallback: bool,
    pub selector: Option<[u8; 4]>,
    pub metadata: Option<MetadataIndex>,

    pub local_storage: BTreeMap<String, LocalVar>, // BTree rather than Hash for deterministic ordering.

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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: &mut Context,
        module: Module,
        name: String,
        abi_errors_display: String,
        args: Vec<(String, Type, Option<MetadataIndex>)>,
        return_type: Type,
        selector: Option<[u8; 4]>,
        is_public: bool,
        is_entry: bool,
        is_original_entry: bool,
        is_fallback: bool,
        metadata: Option<MetadataIndex>,
    ) -> Function {
        let content = FunctionContent {
            name,
            abi_errors_display,
            // Arguments to a function are the arguments to its entry block.
            // We set it up after creating the entry block below.
            arguments: Vec::new(),
            return_type,
            blocks: Vec::new(),
            module,
            is_public,
            is_entry,
            is_original_entry,
            is_fallback,
            selector,
            metadata,
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

        // Setup the arguments.
        let arguments: Vec<_> = args
            .into_iter()
            .enumerate()
            .map(|(idx, (name, ty, arg_metadata))| {
                (
                    name,
                    Value::new_argument(
                        context,
                        BlockArgument {
                            block: entry_block,
                            idx,
                            ty,
                            is_immutable: false,
                        },
                    )
                    .add_metadatum(context, arg_metadata),
                )
            })
            .collect();
        context
            .functions
            .get_mut(func.0)
            .unwrap()
            .arguments
            .clone_from(&arguments);
        let (_, arg_vals): (Vec<_>, Vec<_>) = arguments.iter().cloned().unzip();
        context.blocks.get_mut(entry_block.0).unwrap().args = arg_vals;

        func
    }

    pub fn is_leaf_fn(&self, context: &Context) -> bool {
        let any_call = self
            .instruction_iter(context)
            .filter_map(|(_, i)| i.get_instruction(context).map(|i| i.is_call()))
            .any(|x| x);
        !any_call
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
        let block_idx = context.functions[self.0]
            .blocks
            .iter()
            .position(|block| block == other)
            .ok_or_else(|| {
                let label = &context.blocks[other.0].label;
                IrError::MissingBlock(label.clone())
            })?;

        let new_block = Block::new(context, *self, label);
        context.functions[self.0]
            .blocks
            .insert(block_idx, new_block);
        Ok(new_block)
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

    /// Remove a [`Block`] from this function.
    ///
    /// > Care must be taken to ensure the block has no predecessors otherwise the function will be
    /// > made invalid.
    pub fn remove_block(&self, context: &mut Context, block: &Block) -> Result<(), IrError> {
        let label = block.get_label(context);
        let func = context.functions.get_mut(self.0).unwrap();
        let block_idx = func
            .blocks
            .iter()
            .position(|b| b == block)
            .ok_or(IrError::RemoveMissingBlock(label))?;
        func.blocks.remove(block_idx);
        Ok(())
    }

    /// Remove instructions from function that satisfy a given predicate.
    pub fn remove_instructions<T: Fn(Value) -> bool>(&self, context: &mut Context, pred: T) {
        for block in context.functions[self.0].blocks.clone() {
            block.remove_instructions(context, &pred);
        }
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
                    self.get_unique_label(context, Some(format!("{hint}{idx}")))
                } else {
                    hint
                }
            }
            None => {
                let idx = self.get_next_label_idx(context);
                self.get_unique_label(context, Some(format!("block{idx}")))
            }
        }
    }

    fn get_next_label_idx(&self, context: &mut Context) -> u64 {
        let func = context.functions.get_mut(self.0).unwrap();
        let idx = func.next_label_idx;
        func.next_label_idx += 1;
        idx
    }

    /// Return the number of blocks in this function.
    pub fn num_blocks(&self, context: &Context) -> usize {
        context.functions[self.0].blocks.len()
    }

    /// Return the number of instructions in this function.
    ///
    /// The [crate::InstOp::AsmBlock] is counted as a single instruction,
    /// regardless of the number of [crate::asm::AsmInstruction]s in the ASM block.
    /// E.g., even if the ASM block is empty and contains no instructions, it
    /// will still be counted as a single instruction.
    ///
    /// If you want to count every ASM instruction as an instruction, use
    /// `num_instructions_incl_asm_instructions` instead.
    pub fn num_instructions(&self, context: &Context) -> usize {
        self.block_iter(context)
            .map(|block| block.num_instructions(context))
            .sum()
    }

    /// Return the number of instructions in this function, including
    /// the [crate::asm::AsmInstruction]s found in [crate::InstOp::AsmBlock]s.
    ///
    /// Every [crate::asm::AsmInstruction] encountered in any of the ASM blocks
    /// will be counted as an instruction. The [crate::InstOp::AsmBlock] itself
    /// is not counted but rather replaced with the number of ASM instructions
    /// found in the block. In other words, empty ASM blocks do not count as
    /// instructions.
    ///
    /// If you want to count [crate::InstOp::AsmBlock]s as single instructions, use
    /// `num_instructions` instead.
    pub fn num_instructions_incl_asm_instructions(&self, context: &Context) -> usize {
        self.instruction_iter(context).fold(0, |num, (_, value)| {
            match &value
                .get_instruction(context)
                .expect("We are iterating through the instructions.")
                .op
            {
                InstOp::AsmBlock(asm, _) => num + asm.body.len(),
                _ => num + 1,
            }
        })
    }

    /// Return the function name.
    pub fn get_name<'a>(&self, context: &'a Context) -> &'a str {
        &context.functions[self.0].name
    }

    /// Return the display string representing the function in the ABI errors
    /// related context, in the "errorCodes" and "panickingCalls" sections.
    pub fn get_abi_errors_display(&self, context: &Context) -> String {
        context.functions[self.0].abi_errors_display.clone()
    }

    /// Return the module that this function belongs to.
    pub fn get_module(&self, context: &Context) -> Module {
        context.functions[self.0].module
    }

    /// Return the function entry (i.e., the first) block.
    pub fn get_entry_block(&self, context: &Context) -> Block {
        context.functions[self.0].blocks[0]
    }

    /// Return the attached metadata.
    pub fn get_metadata(&self, context: &Context) -> Option<MetadataIndex> {
        context.functions[self.0].metadata
    }

    /// Whether this function has a valid selector.
    pub fn has_selector(&self, context: &Context) -> bool {
        context.functions[self.0].selector.is_some()
    }

    /// Return the function selector, if it has one.
    pub fn get_selector(&self, context: &Context) -> Option<[u8; 4]> {
        context.functions[self.0].selector
    }

    /// Whether or not the function is a program entry point, i.e. `main`, `#[test]` fns or abi
    /// methods.
    pub fn is_entry(&self, context: &Context) -> bool {
        context.functions[self.0].is_entry
    }

    /// Whether or not the function was a program entry point, i.e. `main`, `#[test]` fns or abi
    /// methods, before it got wrapped within the `__entry` function.
    pub fn is_original_entry(&self, context: &Context) -> bool {
        context.functions[self.0].is_original_entry
    }

    /// Whether or not this function is a contract fallback function
    pub fn is_fallback(&self, context: &Context) -> bool {
        context.functions[self.0].is_fallback
    }

    // Get the function return type.
    pub fn get_return_type(&self, context: &Context) -> Type {
        context.functions[self.0].return_type
    }

    // Set a new function return type.
    pub fn set_return_type(&self, context: &mut Context, new_ret_type: Type) {
        context.functions.get_mut(self.0).unwrap().return_type = new_ret_type
    }

    /// Get the number of args.
    pub fn num_args(&self, context: &Context) -> usize {
        context.functions[self.0].arguments.len()
    }

    /// Get an arg value by name, if found.
    pub fn get_arg(&self, context: &Context, name: &str) -> Option<Value> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(arg_name, val)| (arg_name == name).then_some(val))
            .copied()
    }

    /// Append an extra argument to the function signature.
    ///
    /// NOTE: `arg` must be a `BlockArgument` value with the correct index otherwise `add_arg` will
    /// panic.
    pub fn add_arg<S: Into<String>>(&self, context: &mut Context, name: S, arg: Value) {
        match context.values[arg.0].value {
            ValueDatum::Argument(BlockArgument { idx, .. })
                if idx == context.functions[self.0].arguments.len() =>
            {
                context.functions[self.0].arguments.push((name.into(), arg));
            }
            _ => panic!("Inconsistent function argument being added"),
        }
    }

    /// Find the name of an arg by value.
    pub fn lookup_arg_name<'a>(&self, context: &'a Context, value: &Value) -> Option<&'a String> {
        context.functions[self.0]
            .arguments
            .iter()
            .find_map(|(name, arg_val)| (arg_val == value).then_some(name))
    }

    /// Return an iterator for each of the function arguments.
    pub fn args_iter<'a>(&self, context: &'a Context) -> impl Iterator<Item = &'a (String, Value)> {
        context.functions[self.0].arguments.iter()
    }

    /// Is argument `i` marked immutable?
    pub fn is_arg_immutable(&self, context: &Context, i: usize) -> bool {
        if let Some((_, val)) = context.functions[self.0].arguments.get(i) {
            if let ValueDatum::Argument(arg) = &context.values[val.0].value {
                return arg.is_immutable;
            }
        }
        false
    }

    /// Get a pointer to a local value by name, if found.
    pub fn get_local_var(&self, context: &Context, name: &str) -> Option<LocalVar> {
        context.functions[self.0].local_storage.get(name).copied()
    }

    /// Find the name of a local value by pointer.
    pub fn lookup_local_name<'a>(
        &self,
        context: &'a Context,
        var: &LocalVar,
    ) -> Option<&'a String> {
        context.functions[self.0]
            .local_storage
            .iter()
            .find_map(|(name, local_var)| if local_var == var { Some(name) } else { None })
    }

    /// Add a value to the function local storage.
    ///
    /// The name must be unique to this function else an error is returned.
    pub fn new_local_var(
        &self,
        context: &mut Context,
        name: String,
        local_type: Type,
        initializer: Option<Constant>,
        mutable: bool,
    ) -> Result<LocalVar, IrError> {
        let var = LocalVar::new(context, local_type, initializer, mutable);
        let func = context.functions.get_mut(self.0).unwrap();
        func.local_storage
            .insert(name.clone(), var)
            .map(|_| Err(IrError::FunctionLocalClobbered(func.name.clone(), name)))
            .unwrap_or(Ok(var))
    }

    /// Add a value to the function local storage, by forcing the name to be unique if needed.
    ///
    /// Will use the provided name as a hint and rename to guarantee insertion.
    pub fn new_unique_local_var(
        &self,
        context: &mut Context,
        name: String,
        local_type: Type,
        initializer: Option<Constant>,
        mutable: bool,
    ) -> LocalVar {
        let func = &context.functions[self.0];
        let new_name = if func.local_storage.contains_key(&name) {
            // Assuming that we'll eventually find a unique name by appending numbers to the old
            // one...
            (0..)
                .find_map(|n| {
                    let candidate = format!("{name}{n}");
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
        self.new_local_var(context, new_name, local_type, initializer, mutable)
            .unwrap()
    }

    /// Return an iterator to all of the values in this function's local storage.
    pub fn locals_iter<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = (&'a String, &'a LocalVar)> {
        context.functions[self.0].local_storage.iter()
    }

    /// Remove given list of locals
    pub fn remove_locals(&self, context: &mut Context, removals: &Vec<String>) {
        for remove in removals {
            if let Some(local) = context.functions[self.0].local_storage.remove(remove) {
                context.local_vars.remove(local.0);
            }
        }
    }

    /// Merge values from another [`Function`] into this one.
    ///
    /// The names of the merged values are guaranteed to be unique via the use of
    /// [`Function::new_unique_local_var`].
    ///
    /// Returns a map from the original pointers to the newly merged pointers.
    pub fn merge_locals_from(
        &self,
        context: &mut Context,
        other: Function,
    ) -> HashMap<LocalVar, LocalVar> {
        let mut var_map = HashMap::new();
        let old_vars: Vec<(String, LocalVar, LocalVarContent)> = context.functions[other.0]
            .local_storage
            .iter()
            .map(|(name, var)| (name.clone(), *var, context.local_vars[var.0].clone()))
            .collect();
        for (name, old_var, old_var_content) in old_vars {
            let old_ty = old_var_content
                .ptr_ty
                .get_pointee_type(context)
                .expect("LocalVar types are always pointers.");
            let new_var = self.new_unique_local_var(
                context,
                name.clone(),
                old_ty,
                old_var_content.initializer,
                old_var_content.mutable,
            );
            var_map.insert(old_var, new_var);
        }
        var_map
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
                block
                    .instruction_iter(context)
                    .map(move |ins_val| (*block, ins_val))
            })
    }

    /// Return a reverse iterator to each instruction in each block in this function.
    ///
    /// Blocks and their instructions are both traversed in reverse order.
    ///
    /// This is a convenience method for when all instructions in a function need to be inspected
    /// in reverse order.
    /// The instruction value is returned from the iterator along with the block it belongs to.
    pub fn instruction_iter_rev<'a>(
        &self,
        context: &'a Context,
    ) -> impl Iterator<Item = (Block, Value)> + 'a {
        context.functions[self.0]
            .blocks
            .iter()
            .rev()
            .flat_map(move |block| {
                block
                    .instruction_iter(context)
                    .rev()
                    .map(move |ins_val| (*block, ins_val))
            })
    }

    /// Replace a value with another within this function.
    ///
    /// This is a convenience method which iterates over this function's blocks and calls
    /// [`Block::replace_values`] in turn.
    ///
    /// `starting_block` is an optimisation for when the first possible reference to `old_val` is
    /// known.
    pub fn replace_values(
        &self,
        context: &mut Context,
        replace_map: &FxHashMap<Value, Value>,
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
            block.replace_values(context, replace_map);
        }
    }

    pub fn replace_value(
        &self,
        context: &mut Context,
        old_val: Value,
        new_val: Value,
        starting_block: Option<Block>,
    ) {
        let mut map = FxHashMap::<Value, Value>::default();
        map.insert(old_val, new_val);
        self.replace_values(context, &map, starting_block);
    }

    /// A graphviz dot graph of the control-flow-graph.
    pub fn dot_cfg(&self, context: &Context) -> String {
        let mut worklist = Vec::<Block>::new();
        let mut visited = FxHashSet::<Block>::default();
        let entry = self.get_entry_block(context);
        let mut res = format!("digraph {} {{\n", self.get_name(context));

        worklist.push(entry);
        while let Some(n) = worklist.pop() {
            visited.insert(n);
            for BranchToWithArgs { block: n_succ, .. } in n.successors(context) {
                let _ = writeln!(
                    res,
                    "\t{} -> {}\n",
                    n.get_label(context),
                    n_succ.get_label(context)
                );
                if !visited.contains(&n_succ) {
                    worklist.push(n_succ);
                }
            }
        }

        res += "}\n";
        res
    }
}

/// An iterator over each [`Function`] in a [`Module`].
pub struct FunctionIterator {
    functions: Vec<slotmap::DefaultKey>,
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
