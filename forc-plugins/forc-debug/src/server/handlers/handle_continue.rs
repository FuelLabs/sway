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


        let mut executor = self.test_executor.as_mut().unwrap();
        let src_path = PathBuf::from(
            "/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/main.sw",
        );

        // Set all breakpoints in the VM
        // self.log(format!("setting vm bps\n"));

        

        let opcode_offset = executor.test_offset as u64 / 4;
        // let vm_bps = self.breakpoints.iter().map(|bp| {

        //     // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
        //     // from the beginning of the script.
        //     let opcode_index = *self.source_map.get(&src_path).unwrap().get(&bp.line.unwrap()).unwrap();
        //     let pc_1 = opcode_index + opcode_offset;
        //     let bp = Breakpoint::script(pc_1); // instruction count.
        //     executor.interpreter.set_breakpoint(bp);
        //     bp.clone()
        // });

        // self.log(format!("vm bps: {:?}\n", vm_bps.collect::<Vec<_>>())); // TODO: removing this breaks?

        // TODO: refresh breakpoints
        // self.breakpoints.iter().for_each(|bp| {
        //     // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
        //     // from the beginning of the script.
        //     let opcode_index = *self.source_map.get(&src_path).unwrap().get(&bp.line.unwrap()).unwrap();
        //     let pc_1 = opcode_index + opcode_offset;
        //     let bp = Breakpoint::script(pc_1); // instruction count.
        //     executor.interpreter.set_breakpoint(bp);
        // });

        // self.log(format!("calling executor.debug \n"));

        let debug_res = executor.continue_debugging()?;
        // self.log(format!("opcode_offset: {:?}\n", opcode_offset));
        // self.log(format!("debug_res: {:?}\n", debug_res));
        // self.log(format!("source_map: {:?}\n", self.source_map));
        match debug_res {
            DebugResult::TestComplete(result) => {
                // self.log(format!("finished executing test: {}\n", name));
                test_results.push(result);
            }
            DebugResult::Breakpoint(pc) => {
                // self.log(format!("stopped executing test: {}\n", name));
                // let (line, _) = self.source_map.get(&src_path).unwrap().get_by_right(&pc).unwrap();
                // let breakpoint_id = self.breakpoints.iter().find(|bp| bp.line == Some(*line)).unwrap().id.unwrap();
                // let breakpoint_id = 1; //self.breakpoints.first().unwrap().id.unwrap_or(1);
                // self.log(format!("breakpoints: {:?}\n", self.breakpoints));
                ////
                ///
                // let opcode_index = *self.source_map.get(&src_path).unwrap().get(&bp.line.unwrap()).unwrap();
                // let opcode_offset = offset as u64 / 4;
                // let pc_1 = opcode_index + opcode_offset;
                let to_look_up = pc / 4 - opcode_offset;

                // self.log(format!("to_look_up: {:?}\n", to_look_up));

                let (line, _) = self
                    .source_map
                    .get(&src_path)
                    .unwrap()
                    .iter()
                    .find(|(_, pc)| {
                        // self.log(format!("pc: {}, to_look_up: {}\n", pc, to_look_up));
                        **pc == to_look_up
                    })
                    .unwrap();
                // let line = 12;

                // self.log(format!("line: {:?}\n", line));

                // self.log(format!("breakpoints: {:?}\n", self.breakpoints));

                let breakpoint_id = self
                    .breakpoints
                    .iter()
                    .find(|bp| bp.id.is_some() && bp.line == Some(*line))
                    .unwrap()
                    .id
                    .unwrap();
                ////
                ///
                ///
                // let breakpoint_id: i64 =self.breakpoints.first().unwrap().id.unwrap();
                // let breakpoint_id: i64 = 1;

                // self.log(format!(
                //     "sending event for breakpoint: {}\n\n",
                //     breakpoint_id
                // ));
                let _ = self.server.send_event(Event::Stopped(StoppedEventBody {
                    reason: types::StoppedEventReason::Breakpoint,
                    hit_breakpoint_ids: Some(vec![breakpoint_id]),
                    description: Some(format!("Stopped at breakpoint {}", breakpoint_id)),
                    thread_id: Some(THREAD_ID),
                    preserve_focus_hint: None, //Some(true),
                    text: Some(format!("Stopped at breakpoint {}", breakpoint_id)),
                    all_threads_stopped: None, //Some(true),
                }));
                return Ok(true);
            }
        }

        self.log(format!(
            "finished continue executing {} tests, results: {:?}\n\n",
            test_results.len(),
            test_results
        ));

        // print_tested_pkg(&tested_pkg, &test_print_opts)?; TODO

        Ok(false)
    }
}
