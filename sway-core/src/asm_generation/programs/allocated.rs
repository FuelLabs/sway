use super::{AllocatedProgram, FinalProgram};

use crate::{
    asm_generation::{AllocatedAbstractInstructionSet, InstructionSet},
    asm_lang::allocated_ops::AllocatedOpcode,
};

impl AllocatedProgram {
    pub(crate) fn into_final_program(mut self) -> Result<FinalProgram, crate::CompileError> {
        // Concat the prologue and all the functions together.
        let abstract_ops = AllocatedAbstractInstructionSet {
            ops: std::iter::once(self.prologue.ops)
                .chain(self.functions.into_iter().map(|f| f.ops))
                .flatten()
                .collect(),
        };

        // TODO for multiple sections - realize_labels also realise LWDataID into multiple
        // instructions and finalises the data section.

        let (realized_ops, mut label_offsets) =
            abstract_ops.realize_labels(&mut self.data_section)?;
        let mut ops = InstructionSet {
            ops: realized_ops.pad_to_even(),
        };

        // This points at the byte (*4*8) address immediately following (+1) the last instruction.
        // Some LWs are expanded into two ops to allow for data larger than one word, so we
        // calculate exactly how many ops will be generated to calculate the offset.
        let data_section_offset = ops.ops.iter().fold(0, |acc, item| match &item.opcode {
            AllocatedOpcode::LWDataId(_reg, data_id) if self.data_section.is_reference(data_id) => {
                acc + 8
            }
            AllocatedOpcode::DataSectionOffsetPlaceholder(_) => acc + 8,
            AllocatedOpcode::BLOB(count) => acc + count.value as u64 * 4,
            _ => acc + 4,
        });

        // Finalize the data section.
        let imm_data_section = self.data_section.finalize(data_section_offset);

        // Update placeholders.
        for op in ops.ops.iter_mut() {
            if let AllocatedOpcode::DataSectionOffsetPlaceholder(ref mut offs) = op.opcode {
                *offs = data_section_offset;
                break;
            }
        }

        // Collect the entry point offsets.
        let entries = self
            .entries
            .into_iter()
            .map(|(selector, label, name)| {
                let offset = label_offsets.remove(&label).expect("no offset for entry");
                (selector, offset, name)
            })
            .collect();

        Ok(FinalProgram {
            kind: self.kind,
            imm_data_section,
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
