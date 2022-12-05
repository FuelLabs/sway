use super::{AllocatedProgram, FinalProgram};

use crate::asm_generation::{AllocatedAbstractInstructionSet, InstructionSet};

impl AllocatedProgram {
    pub(crate) fn into_final_program(mut self) -> Result<FinalProgram, crate::CompileError> {
        // Concat the prologue and all the functions together.
        let abstract_ops = AllocatedAbstractInstructionSet {
            ops: std::iter::once(self.prologue.ops)
                .chain(self.functions.into_iter().map(|f| f.ops))
                .flatten()
                .collect(),
        };

        let (realized_ops, mut label_offsets) = abstract_ops
            .relocate_control_flow(&self.data_section)
            .realize_labels(&mut self.data_section)?;
        let ops = InstructionSet {
            ops: realized_ops.pad_to_even(),
        };

        // Collect the entry point offsets.
        let entries = self
            .entries
            .into_iter()
            .map(|(selector, label, name)| {
                let offset = label_offsets
                    .remove(&label)
                    .expect("no offset for entry")
                    .offs;
                (selector, offset, name)
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
        writeln!(f, ";; {:?}", self.kind)?;
        writeln!(f, ";; --- Prologue ---\n{}\n", self.prologue)?;
        writeln!(f, ";; --- Functions ---")?;
        for function in &self.functions {
            writeln!(f, "{function}\n")?;
        }
        writeln!(f, ";; --- Data ---")?;
        writeln!(f, "{}", self.data_section)
    }
}
