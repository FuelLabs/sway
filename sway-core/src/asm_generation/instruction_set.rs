use crate::asm_lang::allocated_ops::AllocatedOp;
use std::fmt;

/// An [InstructionSet] is produced by allocating registers on an [AbstractInstructionSet].
#[derive(Clone)]
pub enum InstructionSet {
    Fuel { ops: Vec<AllocatedOp> },
    Evm { ops: Vec<etk_asm::ops::AbstractOp> },
}

impl fmt::Display for InstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            match self {
                InstructionSet::Fuel { ops } => ops
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                InstructionSet::Evm { ops } => ops
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            }
        )
    }
}
