use crate::{
    context::Context,
    function::Function,
    instruction::{Instruction, InstructionInserter, InstructionIterator},
    value::{Value, ValueContent},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Block(pub generational_arena::Index);

pub struct BlockContent {
    pub label: Label,
    pub function: Function,
    pub instructions: Vec<Value>,
}

pub type Label = String;

impl Block {
    pub fn new(context: &mut Context, function: Function, label: Option<String>) -> Block {
        let label = function.get_unique_label(context, label);
        let phi = Value::new_instruction(context, Instruction::Phi(Vec::new()));
        let content = BlockContent {
            label,
            function,
            instructions: vec![phi],
        };
        Block(context.blocks.insert(content))
    }

    pub fn get_function(&self, context: &Context) -> Function {
        context.blocks[self.0].function
    }

    pub fn ins<'a>(&self, context: &'a mut Context) -> InstructionInserter<'a> {
        InstructionInserter::new(context, *self)
    }

    pub fn get_label(&self, context: &Context) -> String {
        context.blocks[self.0].label.clone()
    }

    pub fn get_phi(&self, context: &Context) -> Value {
        context.blocks[self.0].instructions[0]
    }

    pub fn add_phi(&self, context: &mut Context, from_block: Block, phi_value: Value) {
        let phi_val = self.get_phi(context);
        match &mut context.values[phi_val.0] {
            ValueContent::Instruction(Instruction::Phi(list)) => {
                list.push((from_block, phi_value));
            }
            _ => unreachable!("First value in block instructions is not a phi."),
        }
    }

    pub fn get_phi_val_coming_from(&self, context: &Context, from_block: &Block) -> Option<Value> {
        let phi_val = self.get_phi(context);
        if let ValueContent::Instruction(Instruction::Phi(pairs)) = &context.values[phi_val.0] {
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

    pub fn update_phi_source_block(
        &self,
        context: &mut Context,
        old_source: Block,
        new_source: Block,
    ) {
        let phi_val = self.get_phi(context);
        if let ValueContent::Instruction(Instruction::Phi(ref mut pairs)) =
            &mut context.values[phi_val.0]
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

    pub fn get_term_inst<'a>(&self, context: &'a Context) -> Option<&'a Instruction> {
        context.blocks[self.0]
            .instructions
            .last()
            .map(|val| {
                // It's guaranteed to be an instruction value.
                if let ValueContent::Instruction(term_inst) = &context.values[val.0] {
                    Some(term_inst)
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn replace_value(&self, context: &mut Context, old_val: Value, new_val: Value) {
        for ins in context.blocks[self.0].instructions.clone() {
            ins.replace_instruction_value(context, old_val, new_val);
        }
    }

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

    pub fn instruction_iter(&self, context: &Context) -> InstructionIterator {
        InstructionIterator::new(context, self)
    }
}

pub struct BlockIterator {
    blocks: Vec<generational_arena::Index>,
    next: usize,
}

impl BlockIterator {
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
