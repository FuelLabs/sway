use super::{AbstractEntry, AbstractProgram, AllocatedProgram, ProgramKind};

use crate::{
    asm_generation::fuel::{
        abstract_instruction_set::AbstractInstructionSet,
        allocated_abstract_instruction_set::AllocatedAbstractInstructionSet,
        compiler_constants,
        data_section::{DataSection, Entry},
        register_sequencer::RegisterSequencer,
    },
    asm_lang::{
        allocated_ops::{AllocatedOpcode, AllocatedRegister},
        AllocatedAbstractOp, ConstantRegister, ControlFlowOp, VirtualImmediate12,
        VirtualImmediate18,
    },
    ExperimentalFlags,
};

use sway_error::error::CompileError;

use either::Either;

impl AbstractProgram {
    pub(crate) fn new(
        kind: ProgramKind,
        data_section: DataSection,
        entries: Vec<AbstractEntry>,
        non_entries: Vec<AbstractInstructionSet>,
        reg_seqr: RegisterSequencer,
        experimental: ExperimentalFlags,
    ) -> Self {
        AbstractProgram {
            kind,
            data_section,
            entries,
            non_entries,
            reg_seqr,
            experimental,
        }
    }

    /// True if the [AbstractProgram] does not contain any instructions, or entries, or data in the data section.
    pub(crate) fn is_empty(&self) -> bool {
        self.non_entries.is_empty()
            && self.entries.is_empty()
            && self.data_section.value_pairs.is_empty()
    }

    pub(crate) fn into_allocated_program(
        mut self,
        fallback_fn: Option<crate::asm_lang::Label>,
    ) -> Result<AllocatedProgram, CompileError> {
        // Build our bytecode prologue which has a preamble and for contracts is the switch based on
        // function selector.
        let mut prologue = self.build_preamble();

        match (self.experimental.new_encoding, self.kind) {
            (true, ProgramKind::Contract) => {
                self.build_jump_to_entry(&mut prologue);
            }
            (false, ProgramKind::Contract) => {
                self.build_contract_abi_switch(&mut prologue, fallback_fn);
            }
            _ => {}
        }

        // Keep track of the labels (and names) that represent program entry points.
        let entries = self
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.selector,
                    entry.label,
                    entry.name.clone(),
                    entry.test_decl_ref.clone(),
                )
            })
            .collect();

        // Gather all the functions together, optimise and then verify the instructions.
        let abstract_functions = self
            .entries
            .into_iter()
            .map(|entry| entry.ops)
            .chain(self.non_entries)
            .map(|ais| ais.optimize(&self.data_section))
            .map(AbstractInstructionSet::verify)
            .collect::<Result<Vec<_>, _>>()?;

        // Allocate the registers for each function.
        let functions = abstract_functions
            .into_iter()
            .map(|fn_ops| {
                fn_ops
                    .allocate_registers()
                    .map(AllocatedAbstractInstructionSet::emit_pusha_popa)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // XXX need to verify that the stack use for each function is balanced.

        Ok(AllocatedProgram {
            kind: self.kind,
            data_section: self.data_section,
            prologue,
            functions,
            entries,
        })
    }

    /// Builds the asm preamble, which includes metadata and a jump past the metadata.
    /// Right now, it looks like this:
    ///
    /// WORD OP
    /// 1    MOV $scratch $pc
    /// -    JMPF $zero i2
    /// 2    DATA_START (0-32) (in bytes, offset from $is)
    /// -    DATA_START (32-64)
    /// 3    LW $ds $scratch 1
    /// -    ADD $ds $ds $scratch
    /// 4    .program_start:
    fn build_preamble(&mut self) -> AllocatedAbstractInstructionSet {
        let label = self.reg_seqr.get_label();
        AllocatedAbstractInstructionSet {
            ops: [
                AllocatedAbstractOp {
                    opcode: Either::Left(AllocatedOpcode::MOVE(
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                    )),
                    comment: String::new(),
                    owning_span: None,
                },
                // word 1.5
                AllocatedAbstractOp {
                    opcode: Either::Right(ControlFlowOp::Jump(label)),
                    comment: String::new(),
                    owning_span: None,
                },
                // word 2 -- full word u64 placeholder
                AllocatedAbstractOp {
                    opcode: Either::Right(ControlFlowOp::DataSectionOffsetPlaceholder),
                    comment: "data section offset".into(),
                    owning_span: None,
                },
                AllocatedAbstractOp {
                    opcode: Either::Right(ControlFlowOp::Label(label)),
                    comment: "end of metadata".into(),
                    owning_span: None,
                },
                // word 3 -- load the data offset into $ds
                AllocatedAbstractOp {
                    opcode: Either::Left(AllocatedOpcode::LW(
                        AllocatedRegister::Constant(ConstantRegister::DataSectionStart),
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12::new_unchecked(1, "1 doesn't fit in 12 bits"),
                    )),
                    comment: "".into(),
                    owning_span: None,
                },
                // word 3.5 -- add $ds $ds $is
                AllocatedAbstractOp {
                    opcode: Either::Left(AllocatedOpcode::ADD(
                        AllocatedRegister::Constant(ConstantRegister::DataSectionStart),
                        AllocatedRegister::Constant(ConstantRegister::DataSectionStart),
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                    )),
                    comment: "".into(),
                    owning_span: None,
                },
            ]
            .to_vec(),
        }
    }

    // WHen the new encoding is used, jumps to the `__entry`  function
    fn build_jump_to_entry(&mut self, asm_buf: &mut AllocatedAbstractInstructionSet) {
        let entry = self.entries.iter().find(|x| x.name == "__entry").unwrap();
        asm_buf.ops.push(AllocatedAbstractOp {
            opcode: Either::Right(ControlFlowOp::Jump(entry.label)),
            comment: "jump to abi method selector".into(),
            owning_span: None,
        });
    }

    /// Builds the contract switch statement based on the first argument to a contract call: the
    /// 'selector'.
    /// See https://fuellabs.github.io/fuel-specs/master/vm#call-frames which
    /// describes the first argument to be at word offset 73.
    fn build_contract_abi_switch(
        &mut self,
        asm_buf: &mut AllocatedAbstractInstructionSet,
        fallback_fn: Option<crate::asm_lang::Label>,
    ) {
        const SELECTOR_WORD_OFFSET: u64 = 73;
        const INPUT_SELECTOR_REG: AllocatedRegister = AllocatedRegister::Allocated(0);
        const PROG_SELECTOR_REG: AllocatedRegister = AllocatedRegister::Allocated(1);
        const CMP_RESULT_REG: AllocatedRegister = AllocatedRegister::Allocated(2);

        // Build the switch statement for selectors.
        asm_buf.ops.push(AllocatedAbstractOp {
            opcode: Either::Right(ControlFlowOp::Comment),
            comment: "Begin contract ABI selector switch".into(),
            owning_span: None,
        });

        // Load the selector from the call frame.
        asm_buf.ops.push(AllocatedAbstractOp {
            opcode: Either::Left(AllocatedOpcode::LW(
                INPUT_SELECTOR_REG,
                AllocatedRegister::Constant(ConstantRegister::FramePointer),
                VirtualImmediate12::new_unchecked(
                    SELECTOR_WORD_OFFSET,
                    "constant infallible value",
                ),
            )),
            comment: "load input function selector".into(),
            owning_span: None,
        });

        // Add a 'case' for each entry with a selector.
        for entry in &self.entries {
            let selector = match entry.selector {
                Some(sel) => sel,
                // Skip entries that don't have a selector - they're probably tests.
                None => continue,
            };

            // Put the selector in the data section.
            let data_label = self.data_section.insert_data_value(Entry::new_word(
                u32::from_be_bytes(selector) as u64,
                None,
                None,
            ));

            // Load the data into a register for comparison.
            asm_buf.ops.push(AllocatedAbstractOp {
                opcode: Either::Left(AllocatedOpcode::LoadDataId(PROG_SELECTOR_REG, data_label)),
                comment: format!("load fn selector for comparison {}", entry.name),
                owning_span: None,
            });

            // Compare with the input selector.
            asm_buf.ops.push(AllocatedAbstractOp {
                opcode: Either::Left(AllocatedOpcode::EQ(
                    CMP_RESULT_REG,
                    INPUT_SELECTOR_REG,
                    PROG_SELECTOR_REG,
                )),
                comment: "function selector comparison".into(),
                owning_span: None,
            });

            // Jump to the function label if the selector was equal.
            asm_buf.ops.push(AllocatedAbstractOp {
                // If the comparison result is _not_ equal to 0, then it was indeed equal.
                opcode: Either::Right(ControlFlowOp::JumpIfNotZero(CMP_RESULT_REG, entry.label)),
                comment: "jump to selected function".into(),
                owning_span: None,
            });
        }

        if let Some(fallback_fn) = fallback_fn {
            asm_buf.ops.push(AllocatedAbstractOp {
                opcode: Either::Right(ControlFlowOp::Call(fallback_fn)),
                comment: "call fallback function".into(),
                owning_span: None,
            });
        }

        asm_buf.ops.push(AllocatedAbstractOp {
            opcode: Either::Left(AllocatedOpcode::MOVI(
                AllocatedRegister::Constant(ConstantRegister::Scratch),
                VirtualImmediate18 {
                    value: compiler_constants::MISMATCHED_SELECTOR_REVERT_CODE,
                },
            )),
            comment: "special code for mismatched selector".into(),
            owning_span: None,
        });
        asm_buf.ops.push(AllocatedAbstractOp {
            opcode: Either::Left(AllocatedOpcode::RVRT(AllocatedRegister::Constant(
                ConstantRegister::Scratch,
            ))),
            comment: "revert if no selectors matched".into(),
            owning_span: None,
        });
    }
}

impl std::fmt::Display for AbstractProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ";; Program kind: {:?}", self.kind)?;
        writeln!(f, ";; --- Entries ---")?;
        for entry in &self.entries {
            writeln!(f, "{}\n", entry.ops)?;
        }
        writeln!(f, ";; --- Functions ---")?;
        for function in &self.non_entries {
            writeln!(f, "{function}\n")?;
        }
        writeln!(f, ";; --- Data ---")?;
        write!(f, "{}", self.data_section)
    }
}
