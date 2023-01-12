use super::FinalProgram;

use crate::{
    asm_generation::{instruction_set::InstructionSet, DataSection},
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
            },
            FinalProgram::Evm { ops } => FinalizedAsm {
                data_section: DataSection {
                    ..Default::default()
                },
                program_section: InstructionSet::Evm { ops },
                program_kind: super::ProgramKind::Script,
                entries: vec![],
            },
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalProgram::Fuel {
                data_section, ops, ..
            } => write!(f, "{:?}\n{}", ops, data_section),
            FinalProgram::Evm { ops } => write!(f, "{:?}", ops),
        }
    }
}
