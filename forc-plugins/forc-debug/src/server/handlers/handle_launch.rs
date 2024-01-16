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
    pub(crate) fn handle_launch(&mut self, program_path: String) -> Result<ResponseBody, AdapterError> {
        self.log("launch!\n\n".into());

        // let compiled_program = args.additional_data.
        // let program =
        //     "/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/main.sw";
        let src_path = PathBuf::from(program_path);
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&src_path)
            .map_err(|_| AdapterError::BuildError)?;
        // let pkg_manifest = if let ManifestFile::Package(manifest) = manifest_file {
        //     manifest
        // } else {
        //     return Err(AdapterError::BuildError);
        // };
        let pkg_manifest: PackageManifestFile = manifest_file
            .clone()
            .try_into()
            .map_err(|_| AdapterError::BuildError)?;
        let mut member_manifests = manifest_file
            .member_manifests()
            .map_err(|_| AdapterError::BuildError)?;
        let lock_path = manifest_file
            .lock_path()
            .map_err(|_| AdapterError::BuildError)?;
        let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            Default::default(),
        )
        .map_err(|_| AdapterError::BuildError)?;

        // self.log(format!("build plan!\n{:?}\n", build_plan));

        // let compiled = forc_pkg::check(&plan, Default::default(), false, true, Default::default())?;

        let project_name = member_manifests
            .first_entry()
            .unwrap()
            .get()
            .project
            .name
            .clone();
        let outputs =
            std::iter::once(build_plan.find_member_index(&project_name).unwrap()).collect();

        let built_packages = forc_pkg::build(
            &build_plan,
            Default::default(),
            &BuildProfile {
                include_tests: true,
                ..Default::default()
            },
            &outputs,
        )
        .map_err(|_| AdapterError::BuildError)?;

        // self.log(format!("built!\n{:?}\n", built_packages));

        let mut pkg_to_debug: Option<&BuiltPackage> = None;

        built_packages.iter().for_each(|(_, built_pkg)| {
            if built_pkg.descriptor.manifest_file == pkg_manifest {
                pkg_to_debug = Some(built_pkg);
            }
            let source_map = &built_pkg.source_map;


            let paths = &source_map.paths;
            // Cache the source code for every path in the map, since we'll need it later.
            let source_code = paths
                .iter()
                .filter_map(|path_buf| {
                    if let Ok(source) = fs::read_to_string(path_buf) {
                        return Some((path_buf, source));
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            source_map.map.iter().for_each(|(instruction, sm_span)| {
                let path_buf: &PathBuf = paths.get(sm_span.path.0).unwrap();

                if let Some(source_code) = source_code.get(path_buf) {
                    if let Some(start_pos) = Position::new(&source_code, sm_span.range.start) {
                        let (line, _) = start_pos.line_col();
                        let (line, instruction) = (line as i64, *instruction as u64);

                        self.source_map
                            .entry(path_buf.clone())
                            .and_modify(|new_map| {
                                new_map
                                    .entry(line as i64)
                                    .and_modify(|val| {
                                        // Choose the first instruction that maps to this line
                                        *val = min(instruction, *val);
                                    })
                                    .or_insert(instruction);
                            })
                            .or_insert(HashMap::from([(line, instruction)]));
                    } else {
                        self.log(format!(
                            "Couldn't get position: {:?} in file: {:?}",
                            sm_span.range.start, path_buf
                        ));
                    }
                } else {
                    self.log(format!("Couldn't read file: {:?}", path_buf));
                }
            });

            // self.log("Writing source map!\n\n".into());
            // let pretty = serde_json::to_string_pretty(&self.source_map.clone()).unwrap();
            // fs::write(
            //     "/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/tmp2.txt",
            //     pretty,
            // )
            // .expect("Unable to write file");
        });
        // Run forc test
        // let test_runners = rayon::ThreadPoolBuilder::new()
        // .num_threads(1)
        // .build().map_err(|_| AdapterError::BuildError)?;

        let pkg_to_debug = pkg_to_debug.ok_or_else(|| {
            self.log(format!("Couldn't find built package for {}", project_name));
            AdapterError::BuildError
        })?;

        let built = Built::Package(Arc::from(pkg_to_debug.clone()));

        // Build the tests
        // let built_members: HashMap<&forc_pkg::Pinned, Arc<BuiltPackage>> = built.into_members().collect();

        // // For each member node collect their contract dependencies.
        // let member_contract_dependencies: HashMap<forc_pkg::Pinned, Vec<Arc<forc_pkg::BuiltPackage>>> =
        //     build_plan
        //         .member_nodes()
        //         .map(|member_node| {
        //             let graph = build_plan.graph();
        //             let pinned_member = graph[member_node].clone();
        //             let contract_dependencies = build_plan
        //                 .contract_dependencies(member_node)
        //                 .map(|contract_depency_node_ix| graph[contract_depency_node_ix].clone())
        //                 .filter_map(|pinned| built_members.get(&pinned))
        //                 .cloned()
        //                 .collect();
        //         });

        let built_tests =
            BuiltTests::from_built(built, &build_plan).map_err(|_| AdapterError::BuildError)?;

        // if let BuiltTests::Package(pkg) = built_tests {
        //     let tested_pkg = pkg.run_tests(test_runners, test_filter.as_ref())?;
        // }

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg) => pkg,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildError);
            }
        };

        let entries = pkg_to_debug.bytecode.entries.iter().filter_map(|entry| {
            // self.log(format!("checking entry: {:?}\n", entry));
            if let Some(test_entry) = entry.kind.test() {
                // If a test filter is specified, only the tests containing the filter phrase in
                // their name are going to be executed.
                let name = entry.finalized.fn_name.clone();
                // self.log(format!("found test: {}\n", name));
                // if let Some(filter) = test_filter {
                //     if !filter.filter(&name) {
                //         return None;
                //     }
                // }
                return Some((entry, test_entry));
            }
            None
        });

        // self.log(format!("got entries length: {:?}\n", entries.size_hint()));

        let mut test_results = Vec::new();
        for (entry, test_entry) in entries {
            // Execute the test and return the result.
            let offset =
                u32::try_from(entry.finalized.imm).expect("test instruction offset out of range");
            let name = entry.finalized.fn_name.clone();
            let test_setup = pkg_tests.setup()?;
            // self.log(format!("executing test: {}\n", name));
            // TODO: print test output to terminal

            self.test_executor = Some(TestExecutor::new(
                &pkg_to_debug.bytecode.bytes,
                offset,
                test_setup,
                test_entry,
                name,
            ));
            let mut executor = self.test_executor.as_mut().unwrap();
            // let src_path = PathBuf::from(program_path);

            // Set all breakpoints in the VM
            // self.log(format!("setting vm bps\n"));

            let opcode_offset = offset as u64 / 4;
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

            self.breakpoints.iter().for_each(|bp| {
                // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
                // from the beginning of the script.
                let opcode_index = *self.source_map.get(&src_path).unwrap().get(&bp.line.unwrap()).unwrap();
                let pc_1 = opcode_index + opcode_offset;
                let bp = Breakpoint::script(pc_1); // instruction count.
                executor.interpreter.set_breakpoint(bp);
            });


            // self.log(format!("calling executor.debug \n"));

            let debug_res = executor.start_debugging()?;
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

    
                    let (line, _) = self.source_map.get(&src_path).unwrap().iter().find(|(_, pc)| {
                        // self.log(format!("pc: {}, to_look_up: {}\n", pc, to_look_up));
                        **pc == to_look_up
                    }).unwrap();
                    // let line = 12;

                    // self.log(format!("line: {:?}\n", line));

                    // self.log(format!("breakpoints: {:?}\n", self.breakpoints));

                    let breakpoint_id = self.breakpoints.iter().find(|bp| bp.id.is_some() && bp.line == Some(*line)).unwrap().id.unwrap();
                    // let breakpoint_id: i64 =self.breakpoints.first().unwrap().id.unwrap();
                    // let breakpoint_id: i64 = 1;

                    // self.log(format!(
                    //     "sending event for breakpoint: {}\n\n",
                    //     breakpoint_id
                    // ));
                    self.current_breakpoint_id = Some(breakpoint_id);
                    let _ = self.server.send_event(Event::Stopped(StoppedEventBody {
                        reason: types::StoppedEventReason::Breakpoint,
                        hit_breakpoint_ids: Some(vec![breakpoint_id]),
                        description: Some(format!("Stopped at breakpoint {}", breakpoint_id)),
                        thread_id: Some(THREAD_ID),
                        preserve_focus_hint: None, //Some(true),
                        text: Some(format!("Stopped at breakpoint {}", breakpoint_id)),
                        all_threads_stopped: None,//Some(true),
                    }));
                    // self.log(format!("sent stopped bp event!\n\n"));
                    break;
                }
            }
        }

        self.log(format!(
            "finished executing {} tests, results: {:?}\n\n",
            test_results.len(),
            test_results
        ));

        // print_tested_pkg(&tested_pkg, &test_print_opts)?; TODO

        Ok(ResponseBody::Attach)
    }
}
