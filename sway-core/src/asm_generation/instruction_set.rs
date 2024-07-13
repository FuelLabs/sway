use crate::asm_lang::allocated_ops::AllocatedOp;
use std::fmt;

/// An [InstructionSet] is produced by allocating registers on an [AbstractInstructionSet].
#[derive(Clone)]
pub enum InstructionSet {
    Fuel {
        ops: Vec<AllocatedOp>,
    },
    Evm {
        ops: Vec<etk_asm::ops::AbstractOp>,
    },
    MidenVM {
        ops: Vec<crate::asm_generation::DirectOp>,
    },
}

impl fmt::Display for InstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ops_str = match self {
            InstructionSet::Fuel { ops } 
            | InstructionSet::Evm { ops } 
            | InstructionSet::MidenVM { ops } => ops
                .iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<_>>()
                .join("\n"),
        };
        write!(f, ".program:\n{}", ops_str)
    }
}
