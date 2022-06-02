use crate::asm_generation::{
    AbstractInstructionSet, DataSection, RegisterAllocatedAsmSet, RegisterSequencer,
};
use std::fmt;

/// Represents an ASM set which has had jump labels and jumps optimized
pub enum JumpOptimizedAsmSet {
    ContractAbi {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    ScriptMain {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    PredicateMain {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    // Libraries do not generate any asm.
    Library,
}

impl JumpOptimizedAsmSet {
    pub(crate) fn allocate_registers(
        self,
        register_sequencer: &mut RegisterSequencer,
    ) -> RegisterAllocatedAsmSet {
        match self {
            JumpOptimizedAsmSet::Library => RegisterAllocatedAsmSet::Library,
            JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => {
                let program_section = program_section
                    .realize_labels(&data_section)
                    .allocate_registers(register_sequencer);
                RegisterAllocatedAsmSet::ScriptMain {
                    data_section,
                    program_section,
                }
            }
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => {
                let program_section = program_section
                    .realize_labels(&data_section)
                    .allocate_registers(register_sequencer);
                RegisterAllocatedAsmSet::PredicateMain {
                    data_section,
                    program_section,
                }
            }
            JumpOptimizedAsmSet::ContractAbi {
                program_section,
                data_section,
            } => RegisterAllocatedAsmSet::ContractAbi {
                program_section: program_section
                    .realize_labels(&data_section)
                    .allocate_registers(register_sequencer),
                data_section,
            },
        }
    }
}

impl fmt::Display for JumpOptimizedAsmSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            JumpOptimizedAsmSet::ContractAbi {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            // Libraries do not directly generate any asm.
            JumpOptimizedAsmSet::Library => write!(f, ""),
        }
    }
}
