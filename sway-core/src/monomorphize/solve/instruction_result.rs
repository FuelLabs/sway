use crate::monomorphize::priv_prelude::*;

pub(crate) enum InstructionResult {
    NewInstructions(Vec<Instruction>),
    NoInstruction,
    RedoConstraint,
}

impl InstructionResult {
    pub(crate) fn from_instructions(instructions: Vec<Instruction>) -> InstructionResult {
        if instructions.is_empty() {
            InstructionResult::NoInstruction
        } else {
            InstructionResult::NewInstructions(instructions)
        }
    }
}

impl FromIterator<InstructionResult> for InstructionResult {
    fn from_iter<I: IntoIterator<Item = InstructionResult>>(iter: I) -> Self {
        let mut instructions = vec![];

        for elem in iter {
            match elem {
                InstructionResult::NewInstructions(new_instructions) => {
                    instructions.extend(new_instructions);
                }
                InstructionResult::NoInstruction => {}
                InstructionResult::RedoConstraint => {
                    return InstructionResult::RedoConstraint;
                }
            }
        }

        InstructionResult::from_instructions(instructions)
    }
}
