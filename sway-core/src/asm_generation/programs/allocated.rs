use super::{AllocatedProgram, FinalProgram};

use crate::asm_generation::fuel::allocated_abstract_instruction_set::AllocatedAbstractInstructionSet;

impl AllocatedProgram {
    pub(crate) fn into_final_program(mut self) -> Result<FinalProgram, crate::CompileError> {
        // Concat the prologue and all the functions together.
        let abstract_ops = AllocatedAbstractInstructionSet {
            ops: std::iter::once(self.prologue.ops)
                .chain(self.functions.into_iter().map(|f| f.ops))
                .flatten()
                .collect(),
        };

        let (realized_ops, mut label_offsets) =
            abstract_ops.realize_labels(&mut self.data_section)?;
        let ops = realized_ops.allocated_ops();

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

        Ok(FinalProgram::Fuel {
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
