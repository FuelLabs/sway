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
        self.log("continue!\n\n".into());

        let mut test_results = Vec::new();

        // Set all breakpoints in the VM
        self.update_vm_breakpoints();

        if let Some(executor) = &mut self.test_executor {
            // let mut executor = self.test_executor.as_mut().unwrap();

            let program_path = self.program_path.clone().unwrap();

            return match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    test_results.push(result);

                    self.log(format!(
                        "finished continue executing {} tests, results: {:?}\n\n",
                        test_results.len(),
                        test_results
                    ));

                    // print_tested_pkg(&tested_pkg, &test_print_opts)?; TODO

                    return Ok(false);
                }

                DebugResult::Breakpoint(pc) => {
                    let breakpoint_id = self.vm_pc_to_breakpoint_id(pc)?;
                    self.current_breakpoint_id = Some(breakpoint_id);
                    self.send_stopped_event(breakpoint_id);
                    return Ok(true);
                }
            };
        }
        Err(AdapterError::TestExecutionError {
            source: anyhow::anyhow!("No test executor"),
        })
    }
}
