use super::FinalProgram;

use crate::{
    asm_generation::{
        fuel::data_section::DataSection, instruction_set::InstructionSet, ProgramABI,
    },
    FinalizedAsm, FinalizedEntry,
};

impl FinalProgram {
    pub(crate) fn finalize(self) -> FinalizedAsm {
        match self {
            FinalProgram::Fuel {
                kind,
                data_section,
                ops,
                entries,
            } => FinalizedAsm {
                data_section,
                program_section: InstructionSet::Fuel { ops },
                program_kind: kind,
                entries: entries
                    .into_iter()
                    .map(|(selector, imm, fn_name, test_decl_id)| FinalizedEntry {
                        imm,
                        fn_name,
                        selector,
                        test_decl_id,
                    })
                    .collect(),
                abi: None,
            },
            FinalProgram::Evm { ops, abi } => FinalizedAsm {
                data_section: DataSection {
                    ..Default::default()
                },
                program_section: InstructionSet::Evm { ops },
                program_kind: super::ProgramKind::Script,
                entries: vec![],
                abi: Some(ProgramABI::Evm(abi)),
            },
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalProgram::Fuel {
                data_section, ops, ..
            } => write!(f, "{ops:?}\n{data_section}"),
            FinalProgram::Evm { ops, .. } => {
                let mut separator = etk_dasm::blocks::basic::Separator::new();

                let ctx = etk_asm::ops::Context::new();
                let concretized_ops = ops
                    .iter()
                    .map(|op| etk_asm::disasm::Offset {
                        item: op.clone().concretize(ctx).unwrap(),
                        offset: 0,
                    })
                    .collect::<Vec<_>>();
                separator.push_all(concretized_ops);

                let basic_blocks = separator
                    .take()
                    .into_iter()
                    .chain(separator.finish().into_iter());

                for block in basic_blocks {
                    let mut offset = block.offset;
                    for op in block.ops {
                        let len = op.size();
                        let off = etk_asm::disasm::Offset::new(offset, etk_dasm::DisplayOp(op));
                        offset += len;

                        writeln!(f, "{}", off.item)?;
                    }
                }

                Ok(())
            }
        }
    }
}
