use super::FinalProgram;

use crate::{FinalizedAsm, FinalizedEntry};

impl FinalProgram {
    pub(crate) fn finalize(self) -> FinalizedAsm {
        FinalizedAsm {
            data_section: self.data_section,
            program_section: self.ops,
            program_kind: self.kind,
            entries: self
                .entries
                .into_iter()
                .map(|(selector, imm, fn_name, test_decl_id)| FinalizedEntry {
                    imm,
                    fn_name,
                    selector,
                    test_decl_id,
                })
                .collect(),
        }
    }
}

impl std::fmt::Display for FinalProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.ops, self.data_section)
    }
}
