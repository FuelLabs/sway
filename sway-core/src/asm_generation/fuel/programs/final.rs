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
