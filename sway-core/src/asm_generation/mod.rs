use crate::{
    asm_lang::{
        allocated_ops::AllocatedRegister, virtual_register::*, Label, Op, OrganizationalOp,
        VirtualImmediate12, VirtualOp,
    },
    parse_tree::Literal,
};
use std::{collections::BTreeSet, fmt};

use either::Either;

mod abstract_instruction_set;
pub(crate) mod checks;
pub(crate) mod compiler_constants;
mod data_section;
mod finalized_asm;
pub mod from_ir;
mod instruction_set;
mod jump_optimized_asm_set;
mod register_allocated_asm_set;
pub(crate) mod register_allocator;
mod register_sequencer;

pub use finalized_asm::FinalizedAsm;

use abstract_instruction_set::*;
pub(crate) use data_section::*;
use instruction_set::*;
use jump_optimized_asm_set::*;
use register_allocated_asm_set::*;
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
pub enum SwayAsmSet {
    ContractAbi {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    ScriptMain {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    #[allow(dead_code)]
    PredicateMain {
        data_section: DataSection,
        program_section: AbstractInstructionSet,
    },
    // Libraries do not generate any asm.
    #[allow(dead_code)]
    Library,
}

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

impl fmt::Display for SwayAsmSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwayAsmSet::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            SwayAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            SwayAsmSet::ContractAbi {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            // Libraries do not directly generate any asm.
            SwayAsmSet::Library => write!(f, ""),
        }
    }
}

impl SwayAsmSet {
    pub(crate) fn remove_unnecessary_jumps(self) -> JumpOptimizedAsmSet {
        match self {
            SwayAsmSet::ScriptMain {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
            SwayAsmSet::PredicateMain {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
            SwayAsmSet::Library {} => JumpOptimizedAsmSet::Library,
            SwayAsmSet::ContractAbi {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::ContractAbi {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
        }
    }
}

/// Builds the asm preamble, which includes metadata and a jump past the metadata.
/// Right now, it looks like this:
///
/// WORD OP
/// 1    JI program_start
/// -    NOOP
/// 2    DATA_START (0-32) (in bytes, offset from $is)
/// -    DATA_START (32-64)
/// 3    LW $ds $is               1 (where 1 is in words and $is is a byte address to base off of)
/// -    ADD $ds $ds $is
/// 4    .program_start:
fn build_preamble(register_sequencer: &mut RegisterSequencer) -> [Op; 6] {
    let label = register_sequencer.get_label();
    [
        // word 1
        Op::jump_to_label(label.clone()),
        // word 1.5
        Op {
            opcode: Either::Left(VirtualOp::NOOP),
            comment: "".into(),
            owning_span: None,
        },
        // word 2 -- full word u64 placeholder
        Op {
            opcode: Either::Right(OrganizationalOp::DataSectionOffsetPlaceholder),
            comment: "data section offset".into(),
            owning_span: None,
        },
        Op::unowned_jump_label_comment(label, "end of metadata"),
        // word 3 -- load the data offset into $ds
        Op {
            opcode: Either::Left(VirtualOp::DataSectionRegisterLoadPlaceholder),
            comment: "".into(),
            owning_span: None,
        },
        // word 3.5 -- add $ds $ds $is
        Op {
            opcode: Either::Left(VirtualOp::ADD(
                VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                VirtualRegister::Constant(ConstantRegister::InstructionStart),
            )),
            comment: "".into(),
            owning_span: None,
        },
    ]
}

/// Builds the contract switch statement, or function selector, which takes the selector
/// stored in the call frame (see https://github.com/FuelLabs/sway/issues/97#issuecomment-870150684
/// for an explanation of its location)
fn build_contract_abi_switch(
    register_sequencer: &mut RegisterSequencer,
    data_section: &mut DataSection,
    selectors_and_labels: Vec<([u8; 4], Label)>,
) -> Vec<Op> {
    let input_selector_register = register_sequencer.next();
    let mut asm_buf = vec![Op {
        opcode: Either::Right(OrganizationalOp::Comment),
        comment: "Begin contract ABI selector switch".into(),
        owning_span: None,
    }];
    // load the selector from the call frame
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::LW(
            input_selector_register.clone(),
            VirtualRegister::Constant(ConstantRegister::FramePointer),
            // see https://github.com/FuelLabs/fuel-specs/pull/193#issuecomment-876496372
            // We expect the last four bytes of this word to contain the selector, and the first
            // four bytes to all be 0.
            VirtualImmediate12::new_unchecked(73, "constant infallible value"),
        )),
        comment: "load input function selector".into(),
        owning_span: None,
    });

    for (selector, label) in selectors_and_labels {
        // put the selector in the data section
        let data_label =
            data_section.insert_data_value(&Literal::U32(u32::from_be_bytes(selector)));
        // load the data into a register for comparison
        let prog_selector_register = register_sequencer.next();
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::LWDataId(
                prog_selector_register.clone(),
                data_label,
            )),
            comment: "load fn selector for comparison".into(),
            owning_span: None,
        });
        // compare with the input selector
        let comparison_result_register = register_sequencer.next();
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::EQ(
                comparison_result_register.clone(),
                input_selector_register.clone(),
                prog_selector_register,
            )),
            comment: "function selector comparison".into(),
            owning_span: None,
        });

        // jump to the function label if the selector was equal
        asm_buf.push(Op {
            // if the comparison result is _not_ equal to 0, then it was indeed equal.
            opcode: Either::Right(OrganizationalOp::JumpIfNotZero(
                comparison_result_register,
                label,
            )),
            comment: "jump to selected function".into(),
            owning_span: None,
        });
    }

    // if none of the selectors matched, then revert
    asm_buf.push(Op {
        // see https://github.com/FuelLabs/sway/issues/97#issuecomment-875674105
        // and https://github.com/FuelLabs/sway/issues/444#issuecomment-1012507337
        opcode: Either::Left(VirtualOp::RVRT(VirtualRegister::Constant(
            ConstantRegister::Zero,
        ))),
        comment: "revert if no selectors matched".into(),
        owning_span: None,
    });

    asm_buf
}
