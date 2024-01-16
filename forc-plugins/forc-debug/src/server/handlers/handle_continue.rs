use crate::names::register_index;
use bimap::BiMap;
use dap::events::{BreakpointEventBody, OutputEventBody, StoppedEventBody};
use dap::responses::*;
use forc_test::execute::{DebugResult, TestExecutor};
use fuel_core_client::client::schema::schema::__fields::Mutation::_set_breakpoint_arguments::breakpoint;
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs,
    io::{BufReader, BufWriter, Stdin, Stdout},
    ops::Deref,
    path::PathBuf,
    process,
    sync::Arc,
};
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
use fuel_vm::prelude::*;
use thiserror::Error;

impl DapServer {
    /// Handle a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> Result<bool, AdapterError> {
        // Set all breakpoints in the VM
        self.update_vm_breakpoints();

        if let Some(executor) = self.executors.get_mut(0) {
            let program_path = self.program_path.clone().unwrap();

            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.test_results.push( result);
                }

                DebugResult::Breakpoint(pc) => {
                    return self.send_stopped_event(pc);
                }
            }
            self.executors.remove(0);
        }

        // If there are tests remaning, we should start debugging those until another breakpoint is hit.
        while let Some(next_test_executor) = self.executors.get_mut(0) {
            match next_test_executor.start_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.test_results.push( result);
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
