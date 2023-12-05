//! Represents a 'basic block' of [`Instruction`]s in a control flow graph.
//!
//! [`Block`]s contain zero or more _non-terminating_ instructions and at most one _terminating_
//! instruction or _terminator_.  Terminators are either branches or a return instruction and are
//! the last instruction in the block.
//!
//! Blocks also contain a single 'phi' instruction at its start.  In
//! [SSA](https://en.wikipedia.org/wiki/Static_single_assignment_form) form 'phi' instructions are
//! used to merge values from preceding blocks.
//!
//! Every [`Function`] has at least one block, the first of which is usually labeled `entry`.

use rustc_hash::{FxHashMap, FxHashSet};
use sway_types::Ident;

use crate::{
    context::Context,
    error::IrError,
    function::Function,
    instruction::{FuelVmInstruction, InstOp},
    pretty::DebugWithContext,
    value::{Value, ValueDatum},
    AsmArg, AsmBlock, AsmInstruction, BinaryOpKind, BranchToWithArgs, Constant, Instruction,
    LocalVar, Predicate, Register, Type, UnaryOpKind,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Block(pub generational_arena::Index);

#[doc(hidden)]
pub struct BlockContent {
    /// Block label, useful for printing.
    pub label: Label,
    /// The function containing this block.
    pub function: Function,
    /// List of instructions in the block.
    instructions: Vec<Value>,
    /// Block arguments: Another form of SSA PHIs.
    pub args: Vec<Value>,
    /// CFG predecessors
    pub preds: FxHashSet<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DebugWithContext)]
pub struct BlockArgument {
    /// The block of which this is an argument.
    pub block: Block,
    /// idx'th argument of the block.
    pub idx: usize,
    pub ty: Type,
}

impl BlockArgument {
    /// Get the actual parameter passed to this block argument from `from_block`
    pub fn get_val_coming_from(&self, context: &Context, from_block: &Block) -> Option<Value> {
        for BranchToWithArgs {
            block: succ_block,
            args,
        } in from_block.successors(context)
        {
            if succ_block == self.block {
                return Some(args[self.idx]);
            }
        }
        None
    }
}

/// Each block may be explicitly named.  A [`Label`] is a simple `String` synonym.
pub type Label = String;

impl Block {
    /// Return a new block handle.
    ///
    /// Creates a new Block belonging to `function` in the context and returns its handle.  `label`
    /// is optional and is used only when printing the IR.
    pub fn new(context: &mut Context, function: Function, label: Option<String>) -> Block {
        let label = function.get_unique_label(context, label);
        let content = BlockContent {
            label,
            function,
            instructions: vec![],
            args: vec![],
            preds: FxHashSet::default(),
        };
        Block(context.blocks.insert(content))
    }

    /// Get the parent function for this block.
    pub fn get_function(&self, context: &Context) -> Function {
        context.blocks[self.0].function
    }

    /// Create a new [`InstructionInserter`] to more easily append instructions to this block.
    pub fn append<'a, 'eng>(
        &self,
        context: &'a mut Context<'eng>,
    ) -> InstructionInserter<'a, 'eng> {
        InstructionInserter::new(context, *self, crate::InsertionPosition::End)
    }

    /// Get the label of this block.  If it wasn't given one upon creation it will be a generated
    /// label.
    pub fn get_label(&self, context: &Context) -> String {
        context.blocks[self.0].label.clone()
    }

    /// Set the label of this block.  If the label isn't unique it will be made so.
    pub fn set_label(&self, context: &mut Context, new_label: Option<Label>) {
        let unique_label = self
            .get_function(context)
            .get_unique_label(context, new_label);
        context.blocks[self.0].label = unique_label;
    }

    /// Get the number of instructions in this block
    pub fn num_instructions(&self, context: &Context) -> usize {
        context.blocks[self.0].instructions.len()
    }

    /// Get the i'th block arg.
    pub fn get_arg(&self, context: &Context, index: usize) -> Option<Value> {
        context.blocks[self.0].args.get(index).cloned()
    }

    /// Get the number of predecessor blocks, i.e., blocks which branch to this one.
    pub fn num_predecessors(&self, context: &Context) -> usize {
        context.blocks[self.0].preds.len()
    }

    /// Add a new block argument of type `ty`. Returns its index.
    pub fn new_arg(&self, context: &mut Context, ty: Type) -> usize {
        let idx = context.blocks[self.0].args.len();
        let arg_val = Value::new_argument(
            context,
            BlockArgument {
                block: *self,
                idx,
                ty,
            },
        );
        context.blocks[self.0].args.push(arg_val);
        idx
    }

    pub fn set_arg(&self, context: &mut Context, arg: Value) {
        match context.values[arg.0].value {
            ValueDatum::Argument(BlockArgument { block, idx, ty: _ })
                if block == *self && idx < context.blocks[self.0].args.len() =>
            {
                context.blocks[self.0].args[idx] = arg;
            }
            _ => panic!("Inconsistent block argument being set"),
        }
    }

    /// Add a block argument, asserts that `arg` is suitable here.
    pub fn add_arg(&self, context: &mut Context, arg: Value) {
        match context.values[arg.0].value {
            ValueDatum::Argument(BlockArgument { block, idx, ty: _ })
                if block == *self && idx == context.blocks[self.0].args.len() =>
            {
                context.blocks[self.0].args.push(arg);
            }
            _ => panic!("Inconsistent block argument being added"),
        }
    }

    /// Get an iterator over this block's args.
    pub fn arg_iter<'a>(&'a self, context: &'a Context) -> impl Iterator<Item = &Value> {
        context.blocks[self.0].args.iter()
    }

    /// How many args does this block have?
    pub fn num_args(&self, context: &Context) -> usize {
        context.blocks[self.0].args.len()
    }

    /// Get an iterator over this block's predecessor blocks.
    pub fn pred_iter<'a>(&'a self, context: &'a Context) -> impl Iterator<Item = &Block> {
        context.blocks[self.0].preds.iter()
    }

    /// Add `from_block` to the set of predecessors of this block.
    pub fn add_pred(&self, context: &mut Context, from_block: &Block) {
        context.blocks[self.0].preds.insert(*from_block);
    }

    /// Remove `from_block` from the set of predecessors of this block.
    pub fn remove_pred(&self, context: &mut Context, from_block: &Block) {
        context.blocks[self.0].preds.remove(from_block);
    }

    /// Replace a `old_source` with `new_source` as a predecessor.
    pub fn replace_pred(&self, context: &mut Context, old_source: &Block, new_source: &Block) {
        self.remove_pred(context, old_source);
        self.add_pred(context, new_source);
    }

    /// Get instruction at position `pos`.
    ///
    /// Returns `None` if block is empty.
    pub fn get_instruction_at(&self, context: &Context, pos: usize) -> Option<Value> {
        context.blocks[self.0].instructions.get(pos).cloned()
    }

    /// Get a reference to the block terminator.
    ///
    /// Returns `None` if block is empty.
    pub fn get_terminator<'a>(&self, context: &'a Context) -> Option<&'a Instruction> {
        context.blocks[self.0].instructions.last().and_then(|val| {
            // It's guaranteed to be an instruction value.
            if let ValueDatum::Instruction(term_inst) = &context.values[val.0].value {
                Some(term_inst)
            } else {
                None
            }
        })
    }

    /// Get a mut reference to the block terminator.
    ///
    /// Returns `None` if block is empty.
    pub fn get_terminator_mut<'a>(&self, context: &'a mut Context) -> Option<&'a mut Instruction> {
        context.blocks[self.0].instructions.last().and_then(|val| {
            // It's guaranteed to be an instruction value.
            if let ValueDatum::Instruction(term_inst) = &mut context.values[val.0].value {
                Some(term_inst)
            } else {
                None
            }
        })
    }

    /// Get the CFG successors (and the parameters passed to them) of this block.
    pub(super) fn successors<'a>(&'a self, context: &'a Context) -> Vec<BranchToWithArgs> {
        match self.get_terminator(context) {
            Some(Instruction {
                op:
                    InstOp::ConditionalBranch {
                        true_block,
                        false_block,
                        ..
                    },
                ..
            }) => vec![true_block.clone(), false_block.clone()],

            Some(Instruction {
                op: InstOp::Branch(block),
                ..
            }) => vec![block.clone()],

            _otherwise => Vec::new(),
        }
    }

    /// For a particular successor (if it indeed is one), get the arguments passed.
    pub fn get_succ_params(&self, context: &Context, succ: &Block) -> Vec<Value> {
        self.successors(context)
            .iter()
            .find(|branch| &branch.block == succ)
            .map_or(vec![], |branch| branch.args.clone())
    }

    /// For a particular successor (if it indeed is one), get a mut ref to parameters passed.
    pub fn get_succ_params_mut<'a>(
        &'a self,
        context: &'a mut Context,
        succ: &Block,
    ) -> Option<&'a mut Vec<Value>> {
        match self.get_terminator_mut(context) {
            Some(Instruction {
                op:
                    InstOp::ConditionalBranch {
                        true_block,
                        false_block,
                        ..
                    },
                ..
            }) => {
                if true_block.block == *succ {
                    Some(&mut true_block.args)
                } else if false_block.block == *succ {
                    Some(&mut false_block.args)
                } else {
                    None
                }
            }
            Some(Instruction {
                op: InstOp::Branch(block),
                ..
            }) if block.block == *succ => Some(&mut block.args),
            _ => None,
        }
    }

    /// Replace successor `old_succ` with `new_succ`.
    /// Updates `preds` of both `old_succ` and `new_succ`.
    pub(super) fn replace_successor(
        &self,
        context: &mut Context,
        old_succ: Block,
        new_succ: Block,
        new_params: Vec<Value>,
    ) {
        let mut modified = false;
        if let Some(term) = self.get_terminator_mut(context) {
            match term {
                Instruction {
                    op:
                        InstOp::ConditionalBranch {
                            true_block:
                                BranchToWithArgs {
                                    block: true_block,
                                    args: true_opds,
                                },
                            false_block:
                                BranchToWithArgs {
                                    block: false_block,
                                    args: false_opds,
                                },
                            cond_value: _,
                        },
                    ..
                } => {
                    if old_succ == *true_block {
                        modified = true;
                        *true_block = new_succ;
                        *true_opds = new_params.clone();
                    }
                    if old_succ == *false_block {
                        modified = true;
                        *false_block = new_succ;
                        *false_opds = new_params
                    }
                }

                Instruction {
                    op: InstOp::Branch(BranchToWithArgs { block, args }),
                    ..
                } if *block == old_succ => {
                    *block = new_succ;
                    *args = new_params;
                    modified = true;
                }
                _ => (),
            }
        }
        if modified {
            old_succ.remove_pred(context, self);
            new_succ.add_pred(context, self);
        }
    }

    /// Return whether this block is already terminated.  Checks if the final instruction, if it
    /// exists, is a terminator.
    pub fn is_terminated(&self, context: &Context) -> bool {
        context.blocks[self.0]
            .instructions
            .last()
            .map_or(false, |val| val.is_terminator(context))
    }

    /// Return whether this block is already terminated specifically by a Ret instruction.
    pub fn is_terminated_by_ret_or_revert(&self, context: &Context) -> bool {
        self.get_terminator(context).map_or(false, |i| {
            matches!(
                i,
                Instruction {
                    op: InstOp::Ret(..) | InstOp::FuelVm(FuelVmInstruction::Revert(..)),
                    ..
                }
            )
        })
    }

    /// Replace a value within this block.
    ///
    /// For every instruction within the block, any reference to `old_val` is replaced with
    /// `new_val`.
    pub fn replace_values(&self, context: &mut Context, replace_map: &FxHashMap<Value, Value>) {
        for ins_idx in 0..context.blocks[self.0].instructions.len() {
            let ins = context.blocks[self.0].instructions[ins_idx];
            ins.replace_instruction_values(context, replace_map);
        }
    }

    /// Remove an instruction from this block.
    ///
    /// **NOTE:** We must be very careful!  We mustn't remove the phi or the terminator.  Some
    /// extra checks should probably be performed here to avoid corruption! Ideally we use get a
    /// user/uses system implemented.  Using `Vec::remove()` is also O(n) which we may want to
    /// avoid someday.
    pub fn remove_instruction(&self, context: &mut Context, instr_val: Value) {
        let ins = &mut context.blocks[self.0].instructions;
        if let Some(pos) = ins.iter().position(|iv| *iv == instr_val) {
            ins.remove(pos);
        }
    }

    /// Remove an instruction at position `pos` from this block.
    ///
    /// **NOTE:** We must be very careful!  We mustn't remove the phi or the terminator.  Some
    /// extra checks should probably be performed here to avoid corruption! Ideally we use get a
    /// user/uses system implemented.  Using `Vec::remove()` is also O(n) which we may want to
    /// avoid someday.
    pub fn remove_instruction_at(&self, context: &mut Context, pos: usize) {
        context.blocks[self.0].instructions.remove(pos);
    }

    /// Remove instructions from block that satisfy a given predicate.
    pub fn remove_instructions<T: Fn(Value) -> bool>(&self, context: &mut Context, pred: T) {
        let ins = &mut context.blocks[self.0].instructions;
        ins.retain(|value| !pred(*value));
    }

    /// Clear the current instruction list and take the one provided.
    pub fn take_body(&self, context: &mut Context, new_insts: Vec<Value>) {
        let _ = std::mem::replace(&mut (context.blocks[self.0].instructions), new_insts);
        for inst in &context.blocks[self.0].instructions {
            let ValueDatum::Instruction(inst) = &mut context.values[inst.0].value else {
                continue;
            };
            inst.parent = *self;
        }
    }

    /// Insert instruction(s) at the beginning of the block.
    pub fn prepend_instructions(&self, context: &mut Context, mut insts: Vec<Value>) {
        let block_ins = &mut context.blocks[self.0].instructions;
        insts.append(block_ins);
        context.blocks[self.0].instructions = insts;
    }

    /// Replace an instruction in this block with another.  Will return a ValueNotFound on error.
    /// Any use of the old instruction value will also be replaced by the new value throughout the
    /// owning function if `replace_uses` is set.
    pub fn replace_instruction(
        &self,
        context: &mut Context,
        old_instr_val: Value,
        new_instr_val: Value,
        replace_uses: bool,
    ) -> Result<(), IrError> {
        match context.blocks[self.0]
            .instructions
            .iter_mut()
            .find(|instr_val| *instr_val == &old_instr_val)
        {
            None => Err(IrError::ValueNotFound(
                "Attempting to replace instruction.".to_owned(),
            )),
            Some(instr_val) => {
                *instr_val = new_instr_val;
                if replace_uses {
                    self.get_function(context).replace_value(
                        context,
                        old_instr_val,
                        new_instr_val,
                        Some(*self),
                    );
                }
                Ok(())
            }
        }
    }

    /// Split the block into two.
    ///
    /// This will create a new block and move the instructions at and following `split_idx` to it.
    /// Returns both blocks.
    pub fn split_at(&self, context: &mut Context, split_idx: usize) -> (Block, Block) {
        let function = context.blocks[self.0].function;
        if split_idx == 0 {
            // We can just create a new empty block and put it before this one.  We know that it
            // will succeed because self is definitely in the function, so we can unwrap().
            //
            // If self is the entry block then for now we need to rename it from 'entry' and call
            // our new block 'entry'.
            let new_block_name = (*self == self.get_function(context).get_entry_block(context))
                .then(|| {
                    self.set_label(context, None);
                    "entry".to_owned()
                });
            let new_block = function
                .create_block_before(context, self, new_block_name)
                .unwrap();

            // Move the block arguments to the new block. We collect because we want to mutate next.
            #[allow(clippy::needless_collect)]
            let args: Vec<_> = self.arg_iter(context).copied().collect();
            for arg in args.into_iter() {
                match &mut context.values[arg.0].value {
                    ValueDatum::Argument(BlockArgument {
                        block,
                        idx: _,
                        ty: _,
                    }) => {
                        // We modify the Value in place to be a BlockArgument for the new block.
                        *block = new_block;
                    }
                    _ => unreachable!("Block arg value inconsistent"),
                }
                new_block.add_arg(context, arg);
            }
            context.blocks[self.0].args.clear();

            (new_block, *self)
        } else {
            // Again, we know that it will succeed because self is definitely in the function, and
            // so we can unwrap().
            let new_block = function.create_block_after(context, self, None).unwrap();

            // Split the instructions at the index and append them to the new block.
            let mut tail_instructions = context.blocks[self.0].instructions.split_off(split_idx);
            // Update the parent of tail_instructions.
            for instr in &tail_instructions {
                instr.get_instruction_mut(context).unwrap().parent = new_block;
            }
            context.blocks[new_block.0]
                .instructions
                .append(&mut tail_instructions);

            // If the terminator of the old block (now the new block) was a branch then we need to
            // update the destination block's preds.
            //
            // Copying the candidate blocks and putting them in a vector to avoid borrowing context
            // as immutable and then mutable in the loop body.
            for to_block in match new_block.get_terminator(context) {
                Some(Instruction {
                    op: InstOp::Branch(to_block),
                    ..
                }) => {
                    vec![to_block.block]
                }
                Some(Instruction {
                    op:
                        InstOp::ConditionalBranch {
                            true_block,
                            false_block,
                            ..
                        },
                    ..
                }) => {
                    vec![true_block.block, false_block.block]
                }

                _ => Vec::new(),
            } {
                to_block.replace_pred(context, self, &new_block);
            }

            (*self, new_block)
        }
    }

    /// Return an instruction iterator for each instruction in this block.
    pub fn instruction_iter(&self, context: &Context) -> InstructionIterator {
        InstructionIterator::new(context, self)
    }
}

/// An iterator over each block in a [`Function`].
pub struct BlockIterator {
    blocks: Vec<generational_arena::Index>,
    next: usize,
}

impl BlockIterator {
    /// Return a new iterator for each block in `function`.
    pub fn new(context: &Context, function: &Function) -> Self {
        // Copy all the current block indices, so they may be modified in the context during
        // iteration.
        BlockIterator {
            blocks: context.functions[function.0]
                .blocks
                .iter()
                .map(|block| block.0)
                .collect(),
            next: 0,
        }
    }
}

impl Iterator for BlockIterator {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        if self.next < self.blocks.len() {
            let idx = self.next;
            self.next += 1;
            Some(Block(self.blocks[idx]))
        } else {
            None
        }
    }
}

/// Iterate over all [`Instruction`]s in a specific [`Block`].
pub struct InstructionIterator {
    instructions: Vec<generational_arena::Index>,
    next: usize,
    next_back: isize,
}

impl InstructionIterator {
    pub fn new(context: &Context, block: &Block) -> Self {
        // Copy all the current instruction indices, so they may be modified in the context during
        // iteration.
        InstructionIterator {
            instructions: context.blocks[block.0]
                .instructions
                .iter()
                .map(|val| val.0)
                .collect(),
            next: 0,
            next_back: context.blocks[block.0].instructions.len() as isize - 1,
        }
    }
}

impl Iterator for InstructionIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        if self.next < self.instructions.len() {
            let idx = self.next;
            self.next += 1;
            Some(Value(self.instructions[idx]))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for InstructionIterator {
    fn next_back(&mut self) -> Option<Value> {
        if self.next_back >= 0 {
            let idx = self.next_back;
            self.next_back -= 1;
            Some(Value(self.instructions[idx as usize]))
        } else {
            None
        }
    }
}

/// Where to insert new instructions in the block.
pub enum InsertionPosition {
    // Insert at the start of the basic block.
    Start,
    // Insert at the end of the basic block (append).
    End,
    // Insert after instruction.
    After(Value),
    // Insert before instruction.
    Before(Value),
    // Insert at position / index.
    At(usize),
}

/// Provide a context for inserting new [`Instruction`]s to a [`Block`].
pub struct InstructionInserter<'a, 'eng> {
    context: &'a mut Context<'eng>,
    block: Block,
    position: InsertionPosition,
}

macro_rules! insert_instruction {
    ($self: ident, $ctor: expr) => {{
        let instruction_val = Value::new_instruction($self.context, $self.block, $ctor);
        let pos = $self.get_position_index();
        let instructions = &mut $self.context.blocks[$self.block.0].instructions;
        instructions.insert(pos, instruction_val);
        instruction_val
    }};
}

impl<'a, 'eng> InstructionInserter<'a, 'eng> {
    /// Return a new [`InstructionInserter`] context for `block`.
    pub fn new(
        context: &'a mut Context<'eng>,
        block: Block,
        position: InsertionPosition,
    ) -> InstructionInserter<'a, 'eng> {
        InstructionInserter {
            context,
            block,
            position,
        }
    }

    // Recomputes the index in the instruction vec. O(n) in the worst case.
    fn get_position_index(&self) -> usize {
        let instructions = &self.context.blocks[self.block.0].instructions;
        match self.position {
            InsertionPosition::Start => 0,
            InsertionPosition::End => instructions.len(),
            InsertionPosition::After(inst) => {
                instructions
                    .iter()
                    .position(|val| *val == inst)
                    .expect("Provided position for insertion does not exist")
                    + 1
            }
            InsertionPosition::Before(inst) => instructions
                .iter()
                .position(|val| *val == inst)
                .expect("Provided position for insertion does not exist"),
            InsertionPosition::At(pos) => pos,
        }
    }

    // Insert a slice of instructions.
    pub fn insert_slice(&mut self, slice: &[Value]) {
        let pos = self.get_position_index();
        self.context.blocks[self.block.0]
            .instructions
            .splice(pos..pos, slice.iter().cloned());
    }

    // Insert a single instruction.
    pub fn insert(&mut self, inst: Value) {
        let pos = self.get_position_index();
        self.context.blocks[self.block.0]
            .instructions
            .insert(pos, inst);
    }

    //
    // XXX Maybe these should return result, in case they get bad args?
    //

    /// Append a new [`Instruction::AsmBlock`] from `args` and a `body`.
    pub fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_type: Type,
        return_name: Option<Ident>,
    ) -> Value {
        let asm = AsmBlock::new(
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
            return_type,
            return_name,
        );
        self.asm_block_from_asm(asm, args)
    }

    pub fn asm_block_from_asm(self, asm: AsmBlock, args: Vec<AsmArg>) -> Value {
        insert_instruction!(self, InstOp::AsmBlock(asm, args))
    }

    pub fn bitcast(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::BitCast(value, ty))
    }

    pub fn unary_op(self, op: UnaryOpKind, arg: Value) -> Value {
        insert_instruction!(self, InstOp::UnaryOp { op, arg })
    }

    pub fn wide_unary_op(self, op: UnaryOpKind, arg: Value, result: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { op, arg, result })
        )
    }

    pub fn wide_binary_op(
        self,
        op: BinaryOpKind,
        arg1: Value,
        arg2: Value,
        result: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideBinaryOp {
                op,
                arg1,
                arg2,
                result
            })
        )
    }

    pub fn wide_modular_op(
        self,
        op: BinaryOpKind,
        result: Value,
        arg1: Value,
        arg2: Value,
        arg3: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideModularOp {
                op,
                result,
                arg1,
                arg2,
                arg3,
            })
        )
    }

    pub fn wide_cmp_op(self, op: Predicate, arg1: Value, arg2: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideCmpOp { op, arg1, arg2 })
        )
    }

    pub fn binary_op(self, op: BinaryOpKind, arg1: Value, arg2: Value) -> Value {
        insert_instruction!(self, InstOp::BinaryOp { op, arg1, arg2 })
    }

    pub fn branch(self, to_block: Block, dest_params: Vec<Value>) -> Value {
        let br_val = Value::new_instruction(
            self.context,
            self.block,
            InstOp::Branch(BranchToWithArgs {
                block: to_block,
                args: dest_params,
            }),
        );
        to_block.add_pred(self.context, &self.block);
        self.context.blocks[self.block.0].instructions.push(br_val);
        br_val
    }

    pub fn call(self, function: Function, args: &[Value]) -> Value {
        insert_instruction!(self, InstOp::Call(function, args.to_vec()))
    }

    pub fn cast_ptr(self, val: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::CastPtr(val, ty))
    }

    pub fn cmp(self, pred: Predicate, lhs_value: Value, rhs_value: Value) -> Value {
        insert_instruction!(self, InstOp::Cmp(pred, lhs_value, rhs_value))
    }

    pub fn conditional_branch(
        self,
        cond_value: Value,
        true_block: Block,
        false_block: Block,
        true_dest_params: Vec<Value>,
        false_dest_params: Vec<Value>,
    ) -> Value {
        let cbr_val = Value::new_instruction(
            self.context,
            self.block,
            InstOp::ConditionalBranch {
                cond_value,
                true_block: BranchToWithArgs {
                    block: true_block,
                    args: true_dest_params,
                },
                false_block: BranchToWithArgs {
                    block: false_block,
                    args: false_dest_params,
                },
            },
        );
        true_block.add_pred(self.context, &self.block);
        false_block.add_pred(self.context, &self.block);
        self.context.blocks[self.block.0].instructions.push(cbr_val);
        cbr_val
    }

    pub fn contract_call(
        self,
        return_type: Type,
        name: String,
        params: Value,
        coins: Value,    // amount of coins to forward
        asset_id: Value, // b256 asset ID of the coint being forwarded
        gas: Value,      // amount of gas to forward
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            }
        )
    }

    pub fn gtf(self, index: Value, tx_field_id: u64) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Gtf { index, tx_field_id })
        )
    }

    // get_elem_ptr() and get_elem_ptr_*() all take the element type and will store the pointer to
    // that type in the instruction, which is later returned by Instruction::get_type().
    pub fn get_elem_ptr(self, base: Value, elem_ty: Type, indices: Vec<Value>) -> Value {
        let elem_ptr_ty = Type::new_ptr(self.context, elem_ty);
        insert_instruction!(
            self,
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty,
                indices
            }
        )
    }

    pub fn get_elem_ptr_with_idx(self, base: Value, elem_ty: Type, index: u64) -> Value {
        let idx_val = Constant::get_uint(self.context, 64, index);
        self.get_elem_ptr(base, elem_ty, vec![idx_val])
    }

    pub fn get_elem_ptr_with_idcs(self, base: Value, elem_ty: Type, indices: &[u64]) -> Value {
        let idx_vals = indices
            .iter()
            .map(|idx| Constant::get_uint(self.context, 64, *idx))
            .collect();
        self.get_elem_ptr(base, elem_ty, idx_vals)
    }

    pub fn get_local(self, local_var: LocalVar) -> Value {
        insert_instruction!(self, InstOp::GetLocal(local_var))
    }

    pub fn int_to_ptr(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::IntToPtr(value, ty))
    }

    pub fn load(self, src_val: Value) -> Value {
        insert_instruction!(self, InstOp::Load(src_val))
    }

    pub fn log(self, log_val: Value, log_ty: Type, log_id: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Log {
                log_val,
                log_ty,
                log_id
            })
        )
    }

    pub fn mem_copy_bytes(self, dst_val_ptr: Value, src_val_ptr: Value, byte_len: u64) -> Value {
        insert_instruction!(
            self,
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len
            }
        )
    }

    pub fn mem_copy_val(self, dst_val_ptr: Value, src_val_ptr: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            }
        )
    }

    pub fn nop(self) -> Value {
        insert_instruction!(self, InstOp::Nop)
    }

    pub fn ptr_to_int(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::PtrToInt(value, ty))
    }

    pub fn read_register(self, reg: Register) -> Value {
        insert_instruction!(self, InstOp::FuelVm(FuelVmInstruction::ReadRegister(reg)))
    }

    pub fn ret(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::Ret(value, ty))
    }

    pub fn revert(self, value: Value) -> Value {
        let revert_val = Value::new_instruction(
            self.context,
            self.block,
            InstOp::FuelVm(FuelVmInstruction::Revert(value)),
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(revert_val);
        revert_val
    }

    pub fn smo(self, recipient: Value, message: Value, message_size: Value, coins: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Smo {
                recipient,
                message,
                message_size,
                coins,
            })
        )
    }

    pub fn state_clear(self, key: Value, number_of_slots: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateClear {
                key,
                number_of_slots
            })
        )
    }

    pub fn state_load_quad_word(
        self,
        load_val: Value,
        key: Value,
        number_of_slots: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord {
                load_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_load_word(self, key: Value) -> Value {
        insert_instruction!(self, InstOp::FuelVm(FuelVmInstruction::StateLoadWord(key)))
    }

    pub fn state_store_quad_word(
        self,
        stored_val: Value,
        key: Value,
        number_of_slots: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord {
                stored_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_store_word(self, stored_val: Value, key: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateStoreWord { stored_val, key })
        )
    }

    pub fn store(self, dst_val_ptr: Value, stored_val: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            }
        )
    }
}
