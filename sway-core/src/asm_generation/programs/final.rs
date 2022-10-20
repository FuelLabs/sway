use super::FinalProgram;

use crate::FinalizedAsm;

impl FinalProgram {
    pub(crate) fn finalize(self) -> FinalizedAsm {
        FinalizedAsm {
            data_section: self.data_section,
            program_section: self.ops,
            program_kind: self.kind,
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.ops, self.data_section)
    }
}
