use crate::{
    asm_generation::{
        fuel::data_section::DataSection, instruction_set::InstructionSet, ProgramKind,
    },
    asm_lang::allocated_ops::AllocatedOp,
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
    pub(crate) fn finalize(self) -> FinalizedAsm {
        let FinalProgram {
            kind,
            data_section,
            ops,
            entries,
        } = self;

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

        writeln!(f, ";; Program kind: {:?}", kind)?;
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

// }
// FinalProgram::Evm { ops, .. } => {
//     let mut separator = etk_dasm::blocks::basic::Separator::new();

//     let ctx = etk_asm::ops::Context::new();
//     let concretized_ops = ops
//         .iter()
//         .map(|op| etk_asm::disasm::Offset {
//             item: op.clone().concretize(ctx).unwrap(),
//             offset: 0,
//         })
//         .collect::<Vec<_>>();
//     separator.push_all(concretized_ops);

//     let basic_blocks = separator.take().into_iter().chain(separator.finish());

//     for block in basic_blocks {
//         let mut offset = block.offset;
//         for op in block.ops {
//             let len = op.size();
//             let off = etk_asm::disasm::Offset::new(offset, etk_dasm::DisplayOp(op));
//             offset += len;

//             writeln!(f, "{}", off.item)?;
//         }
//     }

//     Ok(())
// }
// FinalProgram::MidenVM { ops } => write!(f, "{ops:?}"),
// }
