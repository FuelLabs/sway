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

use crate::{
    context::Context,
    error::IrError,
    function::Function,
    instruction::{Instruction, InstructionInserter, InstructionIterator},
    value::{Value, ValueDatum},
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Block(pub generational_arena::Index);

#[doc(hidden)]
pub struct BlockContent {
    pub label: Label,
    pub function: Function,
    pub instructions: Vec<Value>,
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
        let phi = Value::new_instruction(context, Instruction::Phi(Vec::new()), None);
        let content = BlockContent {
            label,
            function,
            instructions: vec![phi],
        };
        Block(context.blocks.insert(content))
    }

    /// Get the parent function for this block.
    pub fn get_function(&self, context: &Context) -> Function {
        context.blocks[self.0].function
    }

    /// Create a new [`InstructionIterator`] to more easily append instructions to this block.
    pub fn ins<'a>(&self, context: &'a mut Context) -> InstructionInserter<'a> {
        InstructionInserter::new(context, *self)
    }

    /// Get the label of this block.  If it wasn't given one upon creation it will be a generated
    /// label.
    pub fn get_label(&self, context: &Context) -> String {
        context.blocks[self.0].label.clone()
    }

    /// Get the phi instruction for this block.
    pub fn get_phi(&self, context: &Context) -> Value {
        context.blocks[self.0].instructions[0]
    }

    /// Add a new phi entry to this block.
    ///
    /// This indicates that if control flow comes from `from_block` then the phi instruction should
    /// use `phi_value`.
    pub fn add_phi(&self, context: &mut Context, from_block: Block, phi_value: Value) {
        let phi_val = self.get_phi(context);
        match &mut context.values[phi_val.0].value {
            ValueDatum::Instruction(Instruction::Phi(list)) => {
                list.push((from_block, phi_value));
            }
            _ => unreachable!("First value in block instructions is not a phi."),
        }
    }

    /// Get the value from the phi instruction which correlates to `from_block`.
    ///
    /// Returns `None` if `from_block` isn't found.
    pub fn get_phi_val_coming_from(&self, context: &Context, from_block: &Block) -> Option<Value> {
        let phi_val = self.get_phi(context);
        if let ValueDatum::Instruction(Instruction::Phi(pairs)) = &context.values[phi_val.0].value {
            pairs.iter().find_map(|(block, value)| {
                if block == from_block {
                    Some(*value)
                } else {
                    None
                }
            })
        } else {
            unreachable!("Phi value must be a PHI instruction.");
        }
    }

    /// Replace a block reference in the phi instruction.
    ///
    /// Any reference to `old_source` will be replace with `new_source` in the list of phi values.
    pub fn update_phi_source_block(
        &self,
        context: &mut Context,
        old_source: Block,
        new_source: Block,
    ) {
        let phi_val = self.get_phi(context);
        if let ValueDatum::Instruction(Instruction::Phi(ref mut pairs)) =
            &mut context.values[phi_val.0].value
        {
            for (block, _) in pairs {
                if *block == old_source {
                    *block = new_source;
                }
            }
        } else {
            unreachable!("Phi value must be a PHI instruction.");
        }
    }

    /// Get a reference to the block terminator.
    ///
    /// Returns `None` if block is empty.
    pub fn get_term_inst<'a>(&self, context: &'a Context) -> Option<&'a Instruction> {
        context.blocks[self.0].instructions.last().and_then(|val| {
            // It's guaranteed to be an instruction value.
            if let ValueDatum::Instruction(term_inst) = &context.values[val.0].value {
                Some(term_inst)
            } else {
                None
            }
        })
    }

    /// Replace a value within this block.
    ///
    /// For every instruction within the block, any reference to `old_val` is replaced with
    /// `new_val`.
    pub fn replace_value(&self, context: &mut Context, old_val: Value, new_val: Value) {
        for ins in context.blocks[self.0].instructions.clone() {
            ins.replace_instruction_value(context, old_val, new_val);
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

    /// Replace an instruction in this block with another.  Will return a ValueNotFound on error.
    /// Any use of the old instruction value will also be replaced by the new value throughout the
    /// owning function.
    pub fn replace_instruction(
        &self,
        context: &mut Context,
        old_instr_val: Value,
        new_instr_val: Value,
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
                self.get_function(context).replace_value(
                    context,
                    old_instr_val,
                    new_instr_val,
                    Some(*self),
                );
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
            let new_block = function.create_block_before(context, self, None).unwrap();
            (new_block, *self)
        } else {
            // Again, we know that it will succeed because self is definitely in the function, and
            // so we can unwrap().
            let new_block = function.create_block_after(context, self, None).unwrap();

            // Split the instructions at the index and append them to the new block.
            let mut tail_instructions = context.blocks[self.0].instructions.split_off(split_idx);
            context.blocks[new_block.0]
                .instructions
                .append(&mut tail_instructions);

            // If the terminator of the old block (now the new block) was a branch then we need to
            // update the destination PHI.
            //
            // Copying the candidate blocks and putting them in a vector to avoid borrowing context
            // as immutable and then mutable in the loop body.
            for to_block in match new_block.get_term_inst(context) {
                Some(Instruction::Branch(to_block)) => {
                    vec![*to_block]
                }
                Some(Instruction::ConditionalBranch {
                    true_block,
                    false_block,
                    ..
                }) => {
                    vec![*true_block, *false_block]
                }

                _ => Vec::new(),
            } {
                to_block.update_phi_source_block(context, *self, new_block);
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
