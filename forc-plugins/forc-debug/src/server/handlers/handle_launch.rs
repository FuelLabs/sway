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
    /// Handle a `launch` request. Returns true if the server should continue running.
    pub(crate) fn handle_launch(&mut self, program_path: PathBuf) -> Result<bool, AdapterError> {
        self.log("launch!\n\n".into());

        // 1. Build the packages
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&program_path)
            .map_err(|_| AdapterError::BuildError)?;

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

        let mut pkg_to_debug: Option<&BuiltPackage> = None;

        // 2. Store the source maps
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
        });

        // 3. Build the tests
        let pkg_to_debug = pkg_to_debug.ok_or_else(|| {
            self.log(format!("Couldn't find built package for {}", project_name));
            AdapterError::BuildError
        })?;

        let built = Built::Package(Arc::from(pkg_to_debug.clone()));

        let built_tests =
            BuiltTests::from_built(built, &build_plan).map_err(|_| AdapterError::BuildError)?;

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg) => pkg,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildError);
            }
        };

        let entries = pkg_to_debug.bytecode.entries.iter().filter_map(|entry| {
            if let Some(test_entry) = entry.kind.test() {
                // If a test filter is specified, only the tests containing the filter phrase in
                // their name are going to be executed.
                let name = entry.finalized.fn_name.clone();
                return Some((entry, test_entry));
            }
            None
        });

        // 3. Run the tests
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

            // Set all breakpoints in the VM
            self.update_vm_breakpoints();

            if let Some(executor) = &mut self.test_executor {
                let opcode_offset = offset as u64 / 4;

                return match executor.start_debugging()? {
                    DebugResult::TestComplete(result) => {
                        test_results.push(result);
                        self.log(format!(
                            "finished executing {} tests, results: {:?}\n\n",
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
        }
        return Ok(true);
    }
}
