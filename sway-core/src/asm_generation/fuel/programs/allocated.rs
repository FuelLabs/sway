use super::{FinalProgram, FnName, SelectorOpt};

use crate::{
    asm_generation::{
        fuel::{
            allocated_abstract_instruction_set::AllocatedAbstractInstructionSet,
            data_section::DataSection,
        },
        ProgramKind,
    },
    asm_lang::{allocated_ops::AllocatedInstruction, Label},
    decl_engine::DeclRefFunction,
};

use either::Either;

/// An [AllocatedProgram] represents code which has allocated registers but still has abstract
/// control flow.
pub(crate) struct AllocatedProgram {
    pub(crate) kind: ProgramKind,
    pub(crate) data_section: DataSection,
    pub(crate) prologue: AllocatedAbstractInstructionSet,
    pub(crate) functions: Vec<AllocatedAbstractInstructionSet>,
    pub(crate) entries: Vec<(SelectorOpt, Label, FnName, Option<DeclRefFunction>)>,
}

impl AllocatedProgram {
    pub(crate) fn into_final_program(mut self) -> Result<FinalProgram, crate::CompileError> {
        // Concat the prologue and all the functions together.
        let abstract_ops = AllocatedAbstractInstructionSet {
            function: None,
            ops: std::iter::once(self.prologue.ops)
                .chain(self.functions.into_iter().map(|f| f.ops))
                .flatten()
                .collect(),
        };

        // Fix the layout of the data section before any jump labels are resolved.
        //
        // This ensures that the instruction sizes, which can depend on data section offsets
        // (`AddrDataId` is realized into one or two instructions depending on the offset of its
        // target), can never change afterward.
        //
        // We reserve one data section pointer slot for each non-copy `LoadDataId`.
        // The slots are filled in-place during the bytecode generation,
        // which does not change the layout.
        //
        // We freeze the worst-case offset of the configurables region, pessimistically
        // assuming that every far jump whose realization can insert a target word
        // into the data section does insert one. `AddrDataId`s pointing to
        // configurables are sized against this worst-case offset. Note that in practice
        // this "pessimization" almost never results in generating two instructions
        // `MOVI` + `ADD` instead of one `ADDI`.
        let num_non_copy_loads = abstract_ops
            .ops
            .iter()
            .filter(|op| match &op.opcode {
                Either::Left(AllocatedInstruction::LoadDataId(_, data_id)) => !self
                    .data_section
                    .has_copy_type(data_id)
                    .expect("`LoadDataId` refers to a non-existent data section entry"),
                _ => false,
            })
            .count();
        self.data_section.reserve_pointer_slots(num_non_copy_loads);

        let (far_jump_sizes, worst_case_far_jump_words) = abstract_ops.collect_far_jumps();
        self.data_section
            .freeze_configurables_base_offset(8 * worst_case_far_jump_words);

        let (realized_ops, mut label_offsets) =
            abstract_ops.lower_to_realized_ops(&mut self.data_section, &far_jump_sizes)?;
        let ops = realized_ops.lower_to_allocated_ops();

        // Collect the entry point offsets.
        let entries = self
            .entries
            .into_iter()
            .map(|(selector, label, name, test_decl_ref)| {
                let offset = label_offsets
                    .remove(&label)
                    .expect("no offset for entry")
                    .offs;
                (selector, offset, name, test_decl_ref)
            })
            .collect();

        Ok(FinalProgram {
            kind: self.kind,
            data_section: self.data_section,
            ops,
            entries,
        })
    }
}

impl std::fmt::Display for AllocatedProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ";; Program kind: {:?}", self.kind)?;
        writeln!(f, ";; --- Prologue ---\n{}\n", self.prologue)?;
        writeln!(f, ";; --- Functions ---")?;
        for function in &self.functions {
            writeln!(f, "{function}\n")?;
        }
        writeln!(f, ";; --- Data ---")?;
        writeln!(f, "{}", self.data_section)
    }
}
