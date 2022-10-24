use super::{FinalProgram, ProgramKind};

use crate::FinalizedAsm;

impl FinalProgram {
    pub(crate) fn finalize(self) -> FinalizedAsm {
        match self.kind {
            ProgramKind::Script => FinalizedAsm::ScriptMain {
                data_section:    self.data_section,
                program_section: self.ops,
            },
            ProgramKind::Contract => FinalizedAsm::ContractAbi {
                data_section:    self.data_section,
                program_section: self.ops,
            },
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.ops, self.data_section)
    }
}
