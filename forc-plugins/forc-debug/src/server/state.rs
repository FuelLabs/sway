use crate::{
    error::AdapterError,
    types::{Breakpoints, Instruction},
};
use dap::types::StartDebuggingRequestKind;
use forc_pkg::BuiltPackage;
use forc_test::{execute::TestExecutor, setup::TestSetup, TestResult};
use std::path::PathBuf;
use sway_core::source_map::SourceMap;

#[derive(Default, Debug, Clone)]
/// The state of the DAP server.
pub struct ServerState {
    // DAP state
    pub program_path: PathBuf,
    pub mode: Option<StartDebuggingRequestKind>,
    pub initialized_event_sent: bool,
    pub started_debugging: bool,
    pub configuration_done: bool,
    pub breakpoints_need_update: bool,
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
        self.executors.clone_from(&self.original_executors);
        self.built_package = None;
        self.test_setup = None;
        self.test_results = vec![];
        self.stopped_on_breakpoint_id = None;
        self.breakpoints_need_update = true;
    }

    /// Initializes the executor stores.
    pub fn init_executors(&mut self, executors: Vec<TestExecutor>) {
        self.executors.clone_from(&executors);
        self.original_executors = executors;
    }

    /// Returns the active [TestExecutor], if any.
    pub fn executor(&mut self) -> Option<&mut TestExecutor> {
        self.executors.first_mut()
    }

    /// Finds the source location matching a VM program counter.
    pub fn vm_pc_to_source_location(
        &self,
        pc: Instruction,
    ) -> Result<(PathBuf, i64), AdapterError> {
        // Convert PC to instruction index (divide by 4 for byte offset)
        let instruction_idx = (pc / 4) as usize;
        if let Some((path, range)) = self.source_map.addr_to_span(instruction_idx) {
            Ok((path, range.start.line as i64))
        } else {
            Err(AdapterError::MissingSourceMap { pc })
        }
    }

    /// Updates the breakpoints in the VM for all remaining [TestExecutor]s.
    pub(crate) fn update_vm_breakpoints(&mut self) {
        if !self.breakpoints_need_update {
            return;
        }

        // Convert breakpoints to instruction offsets using the source map
        let opcode_indexes = self
            .breakpoints
            .iter()
            .flat_map(|(source_path, breakpoints)| {
                breakpoints
                    .iter()
                    .filter_map(|bp| {
                        bp.line.and_then(|line| {
                            // Find any instruction that maps to this line in the source map
                            self.source_map.map.iter().find_map(|(pc, _)| {
                                self.source_map
                                    .addr_to_span(*pc)
                                    .filter(|(path, range)| {
                                        path == source_path && range.start.line as i64 == line
                                    })
                                    .map(|_| pc)
                            })
                        })
                    })
                    .collect::<Vec<_>>()
            });

        // Set breakpoints in the VM
        self.executors.iter_mut().for_each(|executor| {
            let bps: Vec<_> = opcode_indexes
                .clone()
                .map(|opcode_index| fuel_vm::state::Breakpoint::script(*opcode_index as u64))
                .collect();
            executor.interpreter.overwrite_breakpoints(&bps);
        });

        self.breakpoints_need_update = false;
    }

    /// Finds the breakpoint matching a VM program counter.
    pub fn vm_pc_to_breakpoint_id(&self, pc: u64) -> Result<i64, AdapterError> {
        let (source_path, source_line) = self.vm_pc_to_source_location(pc)?;

        // Find the breakpoint ID matching the source location.
        let source_bps = self
            .breakpoints
            .get(&source_path)
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

    pub(crate) fn test_complete(&mut self, result: TestResult) {
        self.test_results.push(result);
        self.executors.remove(0);
    }
}
