use crate::server::AdapterError;
use crate::server::DapServer;
use forc_pkg::{self, BuildProfile, Built, BuiltPackage, PackageManifestFile};
use forc_test::execute::{DebugResult, TestExecutor};
use forc_test::BuiltTests;
use std::{cmp::min, collections::HashMap, fs, path::PathBuf, sync::Arc};
use sway_types::span::Position;

impl DapServer {
    /// Handle a `launch` request. Returns true if the server should continue running.
    pub(crate) fn handle_launch(&mut self, program_path: PathBuf) -> Result<bool, AdapterError> {
        // 1. Build the packages
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&program_path)
            .map_err(|_| AdapterError::BuildFailed)?;
        let pkg_manifest: PackageManifestFile = manifest_file
            .clone()
            .try_into()
            .map_err(|_| AdapterError::BuildFailed)?;
        let mut member_manifests = manifest_file
            .member_manifests()
            .map_err(|_| AdapterError::BuildFailed)?;
        let lock_path = manifest_file
            .lock_path()
            .map_err(|_| AdapterError::BuildFailed)?;
        let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            Default::default(),
        )
        .map_err(|_| AdapterError::BuildFailed)?;

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
        .map_err(|_| AdapterError::BuildFailed)?;

        // 2. Store the source maps
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
                        Some((path_buf, source))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            source_map.map.iter().for_each(|(instruction, sm_span)| {
                let path_buf: &PathBuf = paths.get(sm_span.path.0).unwrap();

                if let Some(source_code) = source_code.get(path_buf) {
                    if let Some(start_pos) = Position::new(source_code, sm_span.range.start) {
                        let (line, _) = start_pos.line_col();
                        let (line, instruction) = (line as i64, *instruction as u64);

                        self.state
                            .source_map
                            .entry(path_buf.clone())
                            .and_modify(|new_map| {
                                new_map
                                    .entry(line)
                                    .and_modify(|val| {
                                        // Choose the first instruction that maps to this line
                                        *val = min(instruction, *val);
                                    })
                                    .or_insert(instruction);
                            })
                            .or_insert(HashMap::from([(line, instruction)]));
                    } else {
                        self.error(format!(
                            "Couldn't get position: {:?} in file: {:?}",
                            sm_span.range.start, path_buf
                        ));
                    }
                } else {
                    self.error(format!("Couldn't read file: {:?}", path_buf));
                }
            });
        });

        // 3. Build the tests
        let pkg_to_debug = pkg_to_debug.ok_or_else(|| {
            self.error(format!("Couldn't find built package for {}", project_name));
            AdapterError::BuildFailed
        })?;

        let built = Built::Package(Arc::from(pkg_to_debug.clone()));

        let built_tests =
            BuiltTests::from_built(built, &build_plan).map_err(|_| AdapterError::BuildFailed)?;

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg) => pkg,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildFailed);
            }
        };

        let entries = pkg_to_debug.bytecode.entries.iter().filter_map(|entry| {
            if let Some(test_entry) = entry.kind.test() {
                return Some((entry, test_entry));
            }
            None
        });

        // 4. Construct a TestExecutor for each test and store it
        let executors: Vec<TestExecutor> = entries
            .enumerate()
            .filter_map(|(order, (entry, test_entry))| {
                let offset = u32::try_from(entry.finalized.imm)
                    .expect("test instruction offset out of range");
                let name = entry.finalized.fn_name.clone();

                if let Ok(test_setup) = pkg_tests.setup() {
                    return Some(TestExecutor::new(
                        &pkg_to_debug.bytecode.bytes,
                        offset,
                        test_setup,
                        test_entry,
                        name.clone(),
                        order as u64,
                    ));
                }
                None
            })
            .collect();

        self.state.init_executors(executors);

        // 5. Start debugging
        self.state.update_vm_breakpoints();
        while let Some(executor) = self.state.executors.get_mut(0) {
            executor.interpreter.set_single_stepping(false);

            match executor.start_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_results.push(result);
                }
                DebugResult::Breakpoint(pc) => {
                    return self.stop_on_breakpoint(pc);
                }
            };
            self.state.executors.remove(0);
        }

        self.log_test_results();
        Ok(false)
    }
}
