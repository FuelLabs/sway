use crate::types::Breakpoints;
use crate::types::SourceMap;
use dap::types::StartDebuggingRequestKind;
use forc_pkg::BuiltPackage;
use forc_test::execute::TestExecutor;
use forc_test::setup::TestSetup;
use std::path::PathBuf;

use super::AdapterError;

#[derive(Default, Debug, Clone)]
pub struct ServerState {
    // DAP state
    pub program_path: PathBuf,
    pub mode: Option<StartDebuggingRequestKind>,
    pub initialized_event_sent: bool,
    pub started_debugging: bool,
    pub configuration_done: bool,
    pub stopped_on_breakpoint_id: Option<i64>,
    pub breakpoints: Breakpoints,

    // Build state
    pub source_map: SourceMap,
    pub built_package: Option<BuiltPackage>,

    // Test state
    pub test_setup: Option<TestSetup>,
    pub test_results: Vec<forc_test::TestResult>,
    pub executors: Vec<TestExecutor>,
    original_executors: Vec<TestExecutor>,
}

impl ServerState {
    /// Resets the data for a new run of the tests.
    pub fn reset(&mut self) {
        self.started_debugging = false;
        self.executors = self.original_executors.clone();
        self.built_package = None;
        self.test_setup = None;
        self.test_results = vec![];
        self.stopped_on_breakpoint_id = None;
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
        // First, try to find the source location by looking for the program counter in the source map.
        let (source_path, source_line) = self
            .source_map
            .iter()
            .find_map(|(source_path, source_map)| {
                let line = source_map.iter().find_map(|(&line, &instruction)| {
                    // Divide by 4 to get the opcode offset rather than the program counter offset.
                    let instruction_offset = pc / 4;
                    if instruction_offset == instruction {
                        return Some(line);
                    }
                    None
                });
                if let Some(line) = line {
                    return Some((source_path, line));
                }
                None
            })
            .ok_or(AdapterError::MissingSourceMap { pc })?;

        // Next, find the breakpoint ID matching the source location.
        let source_bps = self
            .breakpoints
            .get(source_path)
            .ok_or(AdapterError::UnknownBreakpoint { pc })?;
        let breakpoint_id = source_bps
            .iter()
            .find_map(|bp| {
                if bp.line == Some(source_line) {
                    bp.id
                } else {
                    None
                }
            })
            .ok_or(AdapterError::UnknownBreakpoint { pc })?;

        Ok(breakpoint_id)
    }

    /// Updates the breakpoints in the VM for all remaining [TestExecutor]s.
    pub fn update_vm_breakpoints(&mut self) {
        let opcode_indexes = self
            .breakpoints
            .iter()
            .flat_map(|(source_path, breakpoints)| {
                if let Some(source_map) = self.source_map.get(&PathBuf::from(source_path)) {
                    breakpoints
                        .iter()
                        .filter_map(|bp| bp.line.and_then(|line| source_map.get(&line)))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            });

        self.executors.iter_mut().for_each(|executor| {
            // TODO: use `overwrite_breakpoints` when released
            opcode_indexes.clone().for_each(|opcode_index| {
                let bp: fuel_vm::prelude::Breakpoint =
                    fuel_vm::state::Breakpoint::script(*opcode_index);
                executor.interpreter.set_breakpoint(bp);
            });
        });
    }
}
