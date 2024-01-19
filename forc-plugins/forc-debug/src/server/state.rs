use crate::types::SourceMap;
use dap::types::Breakpoint;
use dap::types::StartDebuggingRequestKind;
use forc_test::execute::TestExecutor;
use std::path::PathBuf;

use super::AdapterError;

#[derive(Default, Debug, Clone)]
pub struct ServerState {
    pub mode: Option<StartDebuggingRequestKind>,
    pub program_path: Option<PathBuf>,
    pub source_map: SourceMap,
    pub current_breakpoint_id: Option<i64>,
    pub breakpoints: Vec<Breakpoint>,
    pub initialized_event_sent: bool,
    pub started_debugging: bool,
    pub configuration_done: bool,
    pub test_results: Vec<forc_test::TestResult>,
    pub executors: Vec<TestExecutor>,
    original_executors: Vec<TestExecutor>,
}

impl ServerState {
    /// Resets the data for a new run of the tests.
    pub fn reset(&mut self) {
        self.started_debugging = false;
        self.executors = self.original_executors.clone();
        self.test_results = vec![];
        self.current_breakpoint_id = None;
    }

    /// Initializes the executor stores.
    pub fn init_executors(&mut self, executors: Vec<TestExecutor>) {
        self.executors = executors.clone();
        self.original_executors = executors;
    }

    /// Returns the active [TestExecutor], if any.
    pub fn executor(&mut self) -> Option<&mut TestExecutor> {
        self.executors.first_mut()
    }

    /// Finds the breakpoint matching a VM program counter.
    pub fn vm_pc_to_breakpoint_id(&mut self, pc: u64) -> Result<i64, AdapterError> {
        if let Some(executor) = self.executors.first_mut() {
            if let Some(program_path) = &self.program_path {
                if let Some(source_map) = &self.source_map.get(program_path) {
                    // Divide by 4 to get the opcode offset rather than the program counter offset.
                    let instruction_offset = pc / 4 - (executor.opcode_offset); // TODO: fix offset for 2nd or 3rd test

                    let (line, _) = source_map
                        .iter()
                        .find(|(_, pc)| **pc == instruction_offset)
                        .ok_or(AdapterError::MissingSourceMap { pc })?;

                    let breakpoint_id = self
                        .breakpoints
                        .iter()
                        .find(|bp| bp.line == Some(*line))
                        .ok_or(AdapterError::UnknownBreakpoint)?
                        .id
                        .ok_or(AdapterError::UnknownBreakpoint)?;

                    return Ok(breakpoint_id);
                }
            }
        }
        Err(AdapterError::UnknownBreakpoint)
    }

    /// Updates the breakpoints in the VM for all remaining [TestExecutor]s.
    pub fn update_vm_breakpoints(&mut self) {
        if let Some(program_path) = &self.program_path {
            let opcode_indexes = self.breakpoints.iter().map(|bp| {
                // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
                // from the beginning of the script.
                *self
                    .source_map
                    .get(program_path)
                    .unwrap()
                    .get(&bp.line.unwrap())
                    .unwrap()
            });
            self.executors.iter_mut().for_each(|executor| {
                // TODO: use overwrite_breakpoints when released
                opcode_indexes.clone().for_each(|opcode_index| {
                    let bp =
                        fuel_vm::state::Breakpoint::script(opcode_index + executor.opcode_offset);
                    executor.interpreter.set_breakpoint(bp);
                });
            });
        }
    }
}
