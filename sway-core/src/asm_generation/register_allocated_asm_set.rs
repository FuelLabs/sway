use crate::{
    asm_generation::{DataSection, FinalizedAsm, InstructionSet},
    asm_lang::allocated_ops::AllocatedOp,
};
use std::fmt;

/// Represents an ASM set which has had registers allocated
pub enum RegisterAllocatedAsmSet {
    ContractAbi {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    ScriptMain {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    PredicateMain {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    // Libraries do not generate any asm.
    Library,
}

impl RegisterAllocatedAsmSet {
    pub(crate) fn optimize(self) -> FinalizedAsm {
        // TODO implement this -- noop for now
        match self {
            RegisterAllocatedAsmSet::Library => FinalizedAsm::Library,
            RegisterAllocatedAsmSet::ScriptMain {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::ScriptMain {
                    program_section,
                    data_section,
                }
            }
            RegisterAllocatedAsmSet::PredicateMain {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::PredicateMain {
                    program_section,
                    data_section,
                }
            }
            RegisterAllocatedAsmSet::ContractAbi {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::ContractAbi {
                    program_section,
                    data_section,
                }
            }
        }
    }
}

impl fmt::Display for RegisterAllocatedAsmSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterAllocatedAsmSet::ScriptMain {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            RegisterAllocatedAsmSet::PredicateMain {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            RegisterAllocatedAsmSet::ContractAbi {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            // Libraries do not directly generate any asm.
            RegisterAllocatedAsmSet::Library => write!(f, ""),
        }
    }
}
