use crate::server::{AdapterError, DapServer};
use crate::types::Instruction;
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self, BuildProfile, Built, BuiltPackage, PackageManifestFile};
use forc_test::execute::TestExecutor;
use forc_test::setup::TestSetup;
use forc_test::BuiltTests;
use std::{collections::HashMap, sync::Arc};
use sway_types::LineCol;

impl DapServer {
    /// Handles a `launch` request. Returns true if the server should continue running.
    pub fn handle_launch(&mut self) -> Result<bool, AdapterError> {
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
            .filter_map(|(entry, test_entry)| {
                let offset = u32::try_from(entry.finalized.imm)
                    .expect("test instruction offset out of range");
                let name = entry.finalized.fn_name.clone();
                if test_entry.file_path.as_path() != self.state.program_path.as_path() {
                    return None;
                }

                TestExecutor::build(
                    &pkg_to_debug.bytecode.bytes,
                    offset,
                    test_setup.clone(),
                    test_entry,
                    name.clone(),
                )
                .ok()
            })
            .collect();
        self.state.init_executors(executors);

        // Start debugging
        self.start_debugging_tests(false)
    }

    /// Builds the tests at the given [PathBuf] and stores the source maps.
    pub(crate) fn build_tests(&mut self) -> Result<(BuiltPackage, TestSetup), AdapterError> {
        if let Some(pkg) = &self.state.built_package {
            if let Some(setup) = &self.state.test_setup {
                return Ok((pkg.clone(), setup.clone()));
            }
        }

        let experimental = sway_core::ExperimentalFlags {
            new_encoding: false,
        };

        // 1. Build the packages
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&self.state.program_path)
            .map_err(|err| AdapterError::BuildFailed {
                reason: format!("read manifest file: {:?}", err),
            })?;
        let pkg_manifest: PackageManifestFile =
            manifest_file
                .clone()
                .try_into()
                .map_err(|err: anyhow::Error| AdapterError::BuildFailed {
                    reason: format!("package manifest: {:?}", err),
                })?;
        let member_manifests =
            manifest_file
                .member_manifests()
                .map_err(|err| AdapterError::BuildFailed {
                    reason: format!("member manifests: {:?}", err),
                })?;
        let lock_path = manifest_file
            .lock_path()
            .map_err(|err| AdapterError::BuildFailed {
                reason: format!("lock path: {:?}", err),
            })?;
        let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            &Default::default(),
        )
        .map_err(|err| AdapterError::BuildFailed {
            reason: format!("build plan: {:?}", err),
        })?;

        let project_name = pkg_manifest.project_name();

        let outputs = std::iter::once(build_plan.find_member_index(project_name).ok_or(
            AdapterError::BuildFailed {
                reason: format!("find built project: {}", project_name),
            },
        )?)
        .collect();

        let built_packages = forc_pkg::build(
            &build_plan,
            Default::default(),
            &BuildProfile {
                optimization_level: sway_core::OptLevel::Opt0,
                include_tests: true,
                ..Default::default()
            },
            &outputs,
            experimental,
        )
        .map_err(|err| AdapterError::BuildFailed {
            reason: format!("build packages: {:?}", err),
        })?;

        // 2. Store the source maps
        let mut pkg_to_debug: Option<&BuiltPackage> = None;
        built_packages.iter().for_each(|(_, built_pkg)| {
            if built_pkg.descriptor.manifest_file == pkg_manifest {
                pkg_to_debug = Some(built_pkg);
            }
            let source_map = &built_pkg.source_map;

            let paths = &source_map.paths;
            source_map.map.iter().for_each(|(instruction, sm_span)| {
                if let Some(path_buf) = paths.get(sm_span.path.0) {
                    let LineCol { line, .. } = sm_span.range.start;
                    let (line, instruction) = (line as i64, *instruction as Instruction);

                    self.state
                        .source_map
                        .entry(path_buf.clone())
                        .and_modify(|new_map| {
                            new_map
                                .entry(line)
                                .and_modify(|val| {
                                    // Store the instructions in ascending order
                                    match val.binary_search(&instruction) {
                                        Ok(_) => {} // Ignore duplicates
                                        Err(pos) => val.insert(pos, instruction),
                                    }
                                })
                                .or_insert(vec![instruction]);
                        })
                        .or_insert(HashMap::from([(line, vec![instruction])]));
                } else {
                    self.error(format!(
                        "Path missing from source map: {:?}",
                        sm_span.path.0
                    ));
                }
            });
        });

        // 3. Build the tests
        let built_package = pkg_to_debug.ok_or(AdapterError::BuildFailed {
            reason: format!("find package: {}", project_name),
        })?;

        let built = Built::Package(Arc::from(built_package.clone()));

        let built_tests = BuiltTests::from_built(built, &build_plan).map_err(|err| {
            AdapterError::BuildFailed {
                reason: format!("build tests: {:?}", err),
            }
        })?;

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg_tests) => pkg_tests,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildFailed {
                    reason: "package tests: workspace tests not supported".into(),
                })
            }
        };
        let test_setup = pkg_tests.setup().map_err(|err| AdapterError::BuildFailed {
            reason: format!("test setup: {:?}", err),
        })?;
        self.state.built_package = Some(built_package.clone());
        self.state.test_setup = Some(test_setup.clone());
        Ok((built_package.clone(), test_setup))
    }
}
