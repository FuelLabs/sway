use crate::server::AdapterError;
use crate::server::DapServer;
use forc_pkg::{self, BuildProfile, Built, BuiltPackage, PackageManifestFile};
use forc_test::execute::{DebugResult, TestExecutor};
use forc_test::setup::TestSetup;
use forc_test::BuiltTests;
use std::{cmp::min, collections::HashMap, fs, path::PathBuf, sync::Arc};
use sway_types::span::Position;

impl DapServer {
    /// Handles a `launch` request. Returns true if the server should continue running.
    pub(crate) fn handle_launch(&mut self) -> Result<bool, AdapterError> {
        // Build tests for the given path.
        let (pkg_to_debug, test_setup) = self.build_tests()?;
        let entries = pkg_to_debug.bytecode.entries.iter().filter_map(|entry| {
            if let Some(test_entry) = entry.kind.test() {
                return Some((entry, test_entry));
            }
            None
        });

        // Construct a TestExecutor for each test and store it
        let executors: Vec<TestExecutor> = entries
            .enumerate()
            .map(|(order, (entry, test_entry))| {
                let offset = u32::try_from(entry.finalized.imm)
                    .expect("test instruction offset out of range");
                let name = entry.finalized.fn_name.clone();

                TestExecutor::new(
                    &pkg_to_debug.bytecode.bytes,
                    offset,
                    test_setup.clone(),
                    test_entry,
                    name.clone(),
                    order as u64,
                )
            })
            .collect();
        self.state.init_executors(executors);

        // Start debugging
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

    /// Builds the tests at the given [PathBuf] and stores the source maps.
    pub(crate) fn build_tests(&mut self) -> Result<(BuiltPackage, TestSetup), AdapterError> {
        if let Some(pkg) = &self.state.built_package {
            if let Some(setup) = &self.state.test_setup {
                return Ok((pkg.clone(), setup.clone()));
            }
        }

        // 1. Build the packages
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&self.state.program_path)
            .map_err(|_| AdapterError::BuildFailed {
                phase: "read manifest file".into(),
            })?;
        let pkg_manifest: PackageManifestFile =
            manifest_file
                .clone()
                .try_into()
                .map_err(|_| AdapterError::BuildFailed {
                    phase: "package manifest".into(),
                })?;
        let mut member_manifests =
            manifest_file
                .member_manifests()
                .map_err(|_| AdapterError::BuildFailed {
                    phase: "member manifests".into(),
                })?;
        let lock_path = manifest_file
            .lock_path()
            .map_err(|_| AdapterError::BuildFailed {
                phase: "lock path".into(),
            })?;
        let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            Default::default(),
        )
        .map_err(|_| AdapterError::BuildFailed {
            phase: "build plan".into(),
        })?;

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
                optimization_level: sway_core::OptLevel::Opt0,
                include_tests: true,
                ..Default::default()
            },
            &outputs,
        )
        .map_err(|_| AdapterError::BuildFailed {
            phase: "build packages".into(),
        })?;

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
        let built_package = pkg_to_debug.ok_or_else(|| {
            self.error(format!("Couldn't find built package for {}", project_name));
            AdapterError::BuildFailed {
                phase: "find package".into(),
            }
        })?;

        let built = Built::Package(Arc::from(built_package.clone()));

        let built_tests =
            BuiltTests::from_built(built, &build_plan).map_err(|_| AdapterError::BuildFailed {
                phase: "build tests".into(),
            })?;

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg_tests) => pkg_tests,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildFailed {
                    phase: "package tests".into(),
                })
            }
        };
        let test_setup = pkg_tests.setup().map_err(|_| AdapterError::BuildFailed {
            phase: "test setup".into(),
        })?;
        self.state.built_package = Some(built_package.clone());
        self.state.test_setup = Some(test_setup.clone());
        Ok((built_package.clone(), test_setup))
    }
}
