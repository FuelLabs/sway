use crate::monomorphize::priv_prelude::*;

#[derive(Clone)]
pub(crate) enum InstructionResult {
    NewInstructions(Vec<Instruction>),

    /// Redo the most recent constraint.
    RedoConstraint,
}

impl InstructionResult {
    pub(super) fn empty() -> InstructionResult {
        InstructionResult::NewInstructions(vec![])
    }

    pub(super) fn new(instructions: Vec<Instruction>) -> InstructionResult {
        InstructionResult::NewInstructions(instructions)
    }

    pub(super) fn redo() -> InstructionResult {
        InstructionResult::RedoConstraint
    }

    pub(super) fn and<F>(self, f: F) -> InstructionResult
    where
        F: Fn() -> InstructionResult,
    {
        use InstructionResult::*;
        match self {
            NewInstructions(left) => match f() {
                NewInstructions(right) => NewInstructions([left, right].concat()),
                RedoConstraint => RedoConstraint,
            },
            RedoConstraint => RedoConstraint,
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
                e @ InstructionResult::RedoConstraint => {
                    return e;
                }
            }
        }

        InstructionResult::new(instructions)
    }
}
