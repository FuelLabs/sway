use super::{FinalProgram, FnName, SelectorOpt};

use crate::{
    asm_generation::{
        fuel::{
            allocated_abstract_instruction_set::AllocatedAbstractInstructionSet,
            data_section::DataSection,
        },
        ProgramKind,
    },
    asm_lang::Label,
    decl_engine::DeclRefFunction,
};

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

        let far_jump_sizes = abstract_ops.collect_far_jumps();
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
