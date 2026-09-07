use crate::{
    asm_generation::{
        fuel::data_section::DataSection, instruction_set::InstructionSet, ProgramKind,
    },
    asm_lang::allocated_ops::{AllocatedInstruction, AllocatedOp},
    decl_engine::DeclRefFunction,
    FinalizedAsm, FinalizedEntry,
};

use super::{FnName, ImmOffset, SelectorOpt};

/// A [FinalProgram] represents code which may be serialized to VM bytecode.
pub(crate) struct FinalProgram {
    pub(crate) kind: ProgramKind,
    pub(crate) data_section: DataSection,
    pub(crate) ops: Vec<AllocatedOp>,
    pub(crate) entries: Vec<(SelectorOpt, ImmOffset, FnName, Option<DeclRefFunction>)>,
}

impl FinalProgram {
    /// Finalizes the program layout and turns it into a [FinalizedAsm].
    ///
    /// This is the last point in the compilation pipeline at which the program is
    /// mutated. The produced [FinalizedAsm] is the final program form.
    /// The instructions, the data section, and all the offsets are complete and fixed,
    /// and generating the bytecode out of it is a pure serialization step
    /// (see [FinalizedAsm::to_bytecode]).
    ///
    /// The layout finalization steps are:
    /// 1. Freeze the layout of the data section (it cannot change anymore, all the
    ///    far jump target words were inserted when the jumps were realized) and
    ///    precompute the entry offsets.
    /// 2. Word-align the start of the data section by appending a NOOP to the
    ///    instructions if needed.
    /// 3. Fill the values of the reserved data section pointer slots, one for each non-copy
    ///    [AllocatedInstruction::LoadDataId]. The values are relative to the
    ///    instruction that loads them, so they can only be calculated now, when all
    ///    the instruction offsets and the total code size are fixed.
    pub(crate) fn finalize(self) -> FinalizedAsm {
        let FinalProgram {
            kind,
            mut data_section,
            mut ops,
            entries,
        } = self;

        data_section.freeze_layout();

        // Word-align the data section by appending NOOPs if needed.
        let mut offset_to_data_section_in_bytes: u64 =
            ops.iter().map(|op| op.size_in_bytes(&data_section)).sum();
        if !offset_to_data_section_in_bytes.is_multiple_of(8) {
            ops.push(AllocatedOp {
                opcode: AllocatedInstruction::NOOP,
                comment: "word-align the data section".into(),
                owning_span: None,
            });
            offset_to_data_section_in_bytes += 4;
        }

        // Fill the reserved pointer slots.
        let mut offset_from_instr_start = 0;
        for op in &ops {
            if let AllocatedInstruction::LoadDataId(_reg, data_id) = &op.opcode {
                if !data_section
                    .has_copy_type(data_id)
                    .expect("`data_id` references data non-existent in the data section")
                {
                    // A non-copy load loads a pointer to its target entry instead of
                    // the entry itself. The pointer is stored in one of the reserved
                    // pointer slots, and its value is relative to the load
                    // instruction: the realized load adds `$pc` to it.
                    let offset_bytes = data_section.data_id_to_offset(data_id) as u64;
                    // The -4 is because $pc is added in the *next* instruction.
                    let pointer_offset_from_current_instr =
                        offset_to_data_section_in_bytes - offset_from_instr_start + offset_bytes
                            - 4;
                    data_section.append_pointer(pointer_offset_from_current_instr);
                }
            }
            offset_from_instr_start += op.size_in_bytes(&data_section);
        }

        FinalizedAsm {
            data_section,
            program_section: InstructionSet::Fuel { ops },
            program_kind: kind,
            entries: entries
                .into_iter()
                .map(|(selector, imm, fn_name, test_decl_ref)| FinalizedEntry {
                    imm,
                    fn_name,
                    selector,
                    test_decl_ref,
                })
                .collect(),
            abi: None,
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FinalProgram {
            kind,
            data_section,
            ops,
            ..
        } = self;

        writeln!(f, ";; Program kind: {kind:?}")?;
        writeln!(
            f,
            ".program:\n{}\n{}",
            ops.iter()
                .map(|x| format!("{x}"))
                .collect::<Vec<_>>()
                .join("\n"),
            data_section,
        )
    }
}
