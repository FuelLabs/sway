use crate::language::asm_lang::{allocated_ops::AllocatedRegister, virtual_register::*};

use std::collections::BTreeSet;

mod abstract_instruction_set;
mod allocated_abstract_instruction_set;
mod asm_builder;
pub(crate) mod checks;
pub(crate) mod compiler_constants;
mod data_section;
mod finalized_asm;
pub mod from_ir;
mod instruction_set;
mod programs;
pub(crate) mod register_allocator;
mod register_sequencer;

pub use finalized_asm::FinalizedAsm;

use abstract_instruction_set::*;
use allocated_abstract_instruction_set::*;
pub(crate) use data_section::*;
use instruction_set::*;
use register_sequencer::*;

// Initially, the bytecode will have a lot of individual registers being used. Each register will
// have a new unique identifier. For example, two separate invocations of `+` will result in 4
// registers being used for arguments and 2 for outputs.
//
// After that, the level 0 bytecode will go through a process where register use is minified,
// producing level 1 bytecode. This process is as such:
//
// 1. Detect the last time a register is read. After that, it can be reused and recycled to fit the
//    needs of the next "level 0 bytecode" register
//
// 2. Detect needless assignments and movements, and substitute registers in.
//    i.e.
//    a = b
//    c = a
//
//    would become
//    c = b
//
//
// After the level 1 bytecode is produced, level 2 bytecode is created by limiting the maximum
// number of registers and inserting bytecode to read from/write to memory where needed. Ideally,
// the algorithm for determining which registers will be written off to memory is based on how
// frequently that register is accessed in a particular section of code. Using this strategy, we
// hope to minimize memory writing.
//
// For each line, the number of times a virtual register is accessed between then and the end of the
// program is its register precedence. A virtual register's precedence is 0 if it is currently in
// "memory", and the above described number if it is not. This prevents over-prioritization of
// registers that have already been written off to memory.
//
/// The [SwayAsmSet] contains either a contract ABI and corresponding ASM, a script's main
/// function's ASM, or a predicate's main function's ASM. ASM is never generated for libraries,
/// as that happens when the library itself is imported.

#[derive(Debug)]
struct RegisterAllocationStatus {
    reg: AllocatedRegister,
    used_by: BTreeSet<VirtualRegister>,
}

#[derive(Debug)]
pub(crate) struct RegisterPool {
    registers: Vec<RegisterAllocationStatus>,
}

impl RegisterPool {
    fn init() -> Self {
        let reg_pool: Vec<RegisterAllocationStatus> = (0
                // - 1 because we reserve the final register for the data_section begin
                ..compiler_constants::NUM_ALLOCATABLE_REGISTERS)
            .map(|x| RegisterAllocationStatus {
                reg: AllocatedRegister::Allocated(x),
                used_by: BTreeSet::new(),
            })
            .collect();
        Self {
            registers: reg_pool,
        }
    }

    pub(crate) fn get_register(
        &self,
        virtual_register: &VirtualRegister,
    ) -> Option<AllocatedRegister> {
        let allocated_reg =
            self.registers
                .iter()
                .find(|RegisterAllocationStatus { reg: _, used_by }| {
                    used_by.contains(virtual_register)
                });

        allocated_reg.map(|RegisterAllocationStatus { reg, used_by: _ }| reg.clone())
    }
}
