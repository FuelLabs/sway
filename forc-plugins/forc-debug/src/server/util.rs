use crate::types::Instruction;
use dap::types::Source;
use fuel_vm::fuel_asm::RegId;
use std::path::Path;

#[derive(Debug, Clone)]
/// Utility for generating unique, incremental IDs.
pub(crate) struct IdGenerator {
    next_id: i64,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator {
    pub(crate) fn new() -> Self {
        Self { next_id: 0 }
    }

    pub(crate) fn next(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub(crate) fn path_into_source(path: &Path) -> Source {
    Source {
        path: Some(path.to_string_lossy().into_owned()),
        ..Default::default()
    }
}

pub(crate) fn current_instruction(registers: &[u64]) -> Instruction {
    let pc = registers[RegId::PC];
    let is = registers[RegId::IS];
    pc - is
}
