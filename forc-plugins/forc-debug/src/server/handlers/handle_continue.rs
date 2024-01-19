use crate::names::register_index;
use dap::events::{BreakpointEventBody, OutputEventBody, StoppedEventBody};
use dap::responses::*;
use forc_test::execute::{DebugResult, TestExecutor};
use fuel_core_client::client::schema::schema::__fields::Mutation::_set_breakpoint_arguments::breakpoint;
use std::{path::PathBuf, process, sync::Arc};
use sway_core::source_map::PathIndex;
use sway_types::{span::Position, Span};
// use sway_core::source_map::SourceMap;
use crate::server::{AdapterError, THREAD_ID};
use crate::{server::DapServer, types::DynResult};
use dap::prelude::*;
use forc_pkg::{
    self, manifest::ManifestFile, BuildProfile, Built, BuiltPackage, PackageManifest,
    PackageManifestFile,
};
use forc_test::BuiltTests;
use thiserror::Error;

impl DapServer {
    /// Handle a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> Result<bool, AdapterError> {
        let program_path  = self.program_path.clone().unwrap();

        if let Some(executor) = self.executors.get_mut(0) {
            // Set all breakpoints in the VM

            self.breakpoints.iter().for_each(|bp| {
                // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
                // from the beginning of the script.
                let opcode_index = *self
                    .source_map
                    .get(&program_path)
                    .unwrap()
                    .get(&bp.line.unwrap())
                    .unwrap();
                let bp = fuel_vm::state::Breakpoint::script(opcode_index + executor.opcode_offset);

                // TODO: set all breakpoints in the VM
                executor.interpreter.set_breakpoint(bp);
            });

            // self.update_vm_breakpoints();

            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.test_results.push(result);
                }

                DebugResult::Breakpoint(pc) => {
                    return self.send_stopped_event(pc);
                }
            }
            self.executors.remove(0);
        }

        // If there are tests remaning, we should start debugging those until another breakpoint is hit.
        while let Some(next_test_executor) = self.executors.get_mut(0) {
            self.breakpoints.iter().for_each(|bp| {
                // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
                // from the beginning of the script.
                let opcode_index = *self
                    .source_map
                    .get(&program_path)
                    .unwrap()
                    .get(&bp.line.unwrap())
                    .unwrap();
                let bp = fuel_vm::state::Breakpoint::script(opcode_index + next_test_executor.opcode_offset);

                // TODO: set all breakpoints in the VM
                next_test_executor.interpreter.set_breakpoint(bp);
            });

                        // self.update_vm_breakpoints();


            match next_test_executor.start_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.test_results.push(result);
                }
                DebugResult::Breakpoint(pc) => {
                    return self.send_stopped_event(pc);
                }
            };
            self.executors.remove(0);
        }

        self.log_test_results();
        return Ok(false);
    }
}
