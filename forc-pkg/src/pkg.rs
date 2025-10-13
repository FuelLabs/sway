use crate::manifest::GenericManifestFile;
use crate::{
    lock::Lock,
    manifest::{Dependency, ManifestFile, MemberManifestFiles, PackageManifestFile},
    source::{self, IPFSNode, Source},
    BuildProfile,
};
use anyhow::{anyhow, bail, Context, Error, Result};
use byte_unit::{Byte, UnitType};
use forc_tracing::{println_action_green, println_warning};
use forc_util::{
    default_output_directory, find_file_name, kebab_to_snake_case, print_compiling,
    print_on_failure, print_warnings,
};
use petgraph::{
    self, dot,
    visit::{Bfs, Dfs, EdgeRef, Walker},
    Directed, Direction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeSet, HashMap, HashSet},
    fmt,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};
use sway_core::namespace::Package;
use sway_core::transform::AttributeArg;
pub use sway_core::Programs;
use sway_core::{
    abi_generation::{
        evm_abi,
        fuel_abi::{self, AbiContext},
    },
    asm_generation::ProgramABI,
    decl_engine::DeclRefFunction,
    fuel_prelude::{
        fuel_crypto,
        fuel_tx::{self, Contract, ContractId, StorageSlot},
    },
    language::parsed::TreeType,
    semantic_analysis::namespace,
    source_map::SourceMap,
    write_dwarf, BuildTarget, Engines, FinalizedEntry, LspConfig,
};
use sway_core::{set_bytecode_configurables_offset, DbgGeneration, PrintAsm, PrintIr};
use sway_error::{error::CompileError, handler::Handler, warning::CompileWarning};
use sway_features::ExperimentalFeatures;
use sway_types::{Ident, ProgramId, Span, Spanned};
use sway_utils::{constants, time_expr, PerformanceData, PerformanceMetric};
use tracing::{debug, info};

type GraphIx = u32;
type Node = Pinned;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Edge {
    /// The name specified on the left hand side of the `=` in a dependency declaration under
    /// `[dependencies]` or `[contract-dependencies]` within a forc manifest.
    ///
    /// The name of a dependency may differ from the package name in the case that the dependency's
    /// `package` field is specified.
    ///
    /// For example, in the following, `foo` is assumed to be both the package name and the dependency
    /// name:
    ///
    /// ```toml
    /// foo = { git = "https://github.com/owner/repo", branch = "master" }
    /// ```
    ///
    /// In the following case however, `foo` is the package name, but the dependency name is `foo-alt`:
    ///
    /// ```toml
    /// foo-alt = { git = "https://github.com/owner/repo", branch = "master", package = "foo" }
    /// ```
    pub name: String,
    pub kind: DepKind,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DepKind {
    /// The dependency is a library and declared under `[dependencies]`.
    Library,
    /// The dependency is a contract and declared under `[contract-dependencies]`.
    Contract { salt: fuel_tx::Salt },
}

pub type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;
pub type EdgeIx = petgraph::graph::EdgeIndex<GraphIx>;
pub type NodeIx = petgraph::graph::NodeIndex<GraphIx>;
pub type ManifestMap = HashMap<PinnedId, PackageManifestFile>;

/// A unique ID for a pinned package.
///
/// The internal value is produced by hashing the package's name and `source::Pinned`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PinnedId(u64);

/// The result of successfully compiling a package.
#[derive(Debug, Clone)]
pub struct BuiltPackage {
    pub descriptor: PackageDescriptor,
    pub program_abi: ProgramABI,
    pub storage_slots: Vec<StorageSlot>,
    pub warnings: Vec<CompileWarning>,
    pub source_map: SourceMap,
    pub tree_type: TreeType,
    pub bytecode: BuiltPackageBytecode,
    /// `Some` for contract member builds where tests were included. This is
    /// required so that we can deploy once instance of the contract (without
    /// tests) with a valid contract ID before executing the tests as scripts.
    ///
    /// For non-contract members, this is always `None`.
    pub bytecode_without_tests: Option<BuiltPackageBytecode>,
}

/// The package descriptors that a `BuiltPackage` holds so that the source used for building the
/// package can be retrieved later on.
#[derive(Debug, Clone)]
pub struct PackageDescriptor {
    pub name: String,
    pub target: BuildTarget,
    pub manifest_file: PackageManifestFile,
    pub pinned: Pinned,
}

/// The bytecode associated with a built package along with its entry points.
#[derive(Debug, Clone)]
pub struct BuiltPackageBytecode {
    pub bytes: Vec<u8>,
    pub entries: Vec<PkgEntry>,
}

/// Represents a package entry point.
#[derive(Debug, Clone)]
pub struct PkgEntry {
    pub finalized: FinalizedEntry,
    pub kind: PkgEntryKind,
}

/// Data specific to each kind of package entry point.
#[derive(Debug, Clone)]
pub enum PkgEntryKind {
    Main,
    Test(PkgTestEntry),
}

/// The possible conditions for a test result to be considered "passing".
#[derive(Debug, Clone)]
pub enum TestPassCondition {
    ShouldRevert(Option<u64>),
    ShouldNotRevert,
}

/// Data specific to the test entry point.
#[derive(Debug, Clone)]
pub struct PkgTestEntry {
    pub pass_condition: TestPassCondition,
    pub span: Span,
    pub file_path: Arc<PathBuf>,
}

/// The result of successfully compiling a workspace.
pub type BuiltWorkspace = Vec<Arc<BuiltPackage>>;

#[derive(Debug, Clone)]
pub enum Built {
    /// Represents a standalone package build.
    Package(Arc<BuiltPackage>),
    /// Represents a workspace build.
    Workspace(BuiltWorkspace),
}

/// The result of the `compile` function, i.e. compiling a single package.
pub struct CompiledPackage {
    pub source_map: SourceMap,
    pub tree_type: TreeType,
    pub program_abi: ProgramABI,
    pub storage_slots: Vec<StorageSlot>,
    pub bytecode: BuiltPackageBytecode,
    pub namespace: namespace::Package,
    pub warnings: Vec<CompileWarning>,
    pub metrics: PerformanceData,
}

/// Compiled contract dependency parts relevant to calculating a contract's ID.
pub struct CompiledContractDependency {
    pub bytecode: Vec<u8>,
    pub storage_slots: Vec<StorageSlot>,
}

/// The set of compiled contract dependencies, provided to dependency namespace construction.
pub type CompiledContractDeps = HashMap<NodeIx, CompiledContractDependency>;

/// A package uniquely identified by name along with its source.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Pkg {
    /// The unique name of the package as declared in its manifest.
    pub name: String,
    /// Where the package is sourced from.
    pub source: Source,
}

/// A package uniquely identified by name along with its pinned source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned {
    pub name: String,
    pub source: source::Pinned,
}

/// Represents the full build plan for a project.
#[derive(Clone, Debug)]
pub struct BuildPlan {
    graph: Graph,
    manifest_map: ManifestMap,
    compilation_order: Vec<NodeIx>,
}

/// Error returned upon failed parsing of `PinnedId::from_str`.
#[derive(Clone, Debug)]
pub struct PinnedIdParseError;

#[derive(Default, Clone)]
pub struct PkgOpts {
    /// Path to the project, if not specified, current working directory will be used.
    pub path: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    pub offline: bool,
    /// Terse mode. Limited warning and error output.
    pub terse: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    pub locked: bool,
    /// The directory in which the sway compiler output artifacts are placed.
    ///
    /// By default, this is `<project-root>/out`.
    pub output_directory: Option<String>,
    /// The IPFS node to be used for fetching IPFS sources.
    pub ipfs_node: IPFSNode,
}

#[derive(Default, Clone)]
pub struct PrintOpts {
    /// Print the generated Sway AST (Abstract Syntax Tree).
    pub ast: bool,
    /// Print the computed Sway DCA (Dead Code Analysis) graph to the specified path.
    /// If not specified prints to stdout.
    pub dca_graph: Option<String>,
    /// Specifies the url format to be used in the generated dot file.
    /// Variables {path}, {line} {col} can be used in the provided format.
    /// An example for vscode would be: "vscode://file/{path}:{line}:{col}"
    pub dca_graph_url_format: Option<String>,
    /// Print the generated ASM.
    pub asm: PrintAsm,
    /// Print the bytecode. This is the final output of the compiler.
    pub bytecode: bool,
    /// Print the original source code together with bytecode.
    pub bytecode_spans: bool,
    /// Print the generated Sway IR (Intermediate Representation).
    pub ir: PrintIr,
    /// Output build errors and warnings in reverse order.
    pub reverse_order: bool,
}

#[derive(Default, Clone)]
pub struct MinifyOpts {
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    pub json_abi: bool,
    /// By default the JSON for initial storage slots is formatted for human readability. By using
    /// this option JSON output will be "minified", i.e. all on one line without whitespace.
    pub json_storage_slots: bool,
}

/// Represents a compiled contract ID as a pub const in a contract.
type ContractIdConst = String;

/// The set of options provided to the `build` functions.
#[derive(Default, Clone)]
pub struct BuildOpts {
    pub pkg: PkgOpts,
    pub print: PrintOpts,
    pub minify: MinifyOpts,
    /// If set, generates a JSON file containing the hex-encoded script binary.
    pub hex_outfile: Option<String>,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs debug info to the provided file.
    /// If the argument provided ends with .json, a JSON is emitted,
    /// otherwise, an ELF file containing DWARF is emitted.
    pub debug_outfile: Option<String>,
    /// Build target to use.
    pub build_target: BuildTarget,
    /// Name of the build profile to use.
    pub build_profile: String,
    /// Use the release build profile.
    /// The release profile can be customized in the manifest file.
    pub release: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
    /// Profile the build process.
    pub profile: bool,
    /// If set, outputs compilation metrics info in JSON format.
    pub metrics_outfile: Option<String>,
    /// Warnings must be treated as compiler errors.
    pub error_on_warnings: bool,
    /// Include all test functions within the build.
    pub tests: bool,
    /// The set of options to filter by member project kind.
    pub member_filter: MemberFilter,
    /// Set of enabled experimental flags
    pub experimental: Vec<sway_features::Feature>,
    /// Set of disabled experimental flags
    pub no_experimental: Vec<sway_features::Feature>,
    /// Do not output any build artifacts, e.g., bytecode, ABI JSON, etc.
    pub no_output: bool,
}

/// The set of options to filter type of projects to build in a workspace.
#[derive(Clone)]
pub struct MemberFilter {
    pub build_contracts: bool,
    pub build_scripts: bool,
    pub build_predicates: bool,
    pub build_libraries: bool,
}

impl Default for MemberFilter {
    fn default() -> Self {
        Self {
            build_contracts: true,
            build_scripts: true,
            build_predicates: true,
            build_libraries: true,
        }
    }
}

impl MemberFilter {
    /// Returns a new `MemberFilter` that only builds scripts.
    pub fn only_scripts() -> Self {
        Self {
            build_contracts: false,
            build_scripts: true,
            build_predicates: false,
            build_libraries: false,
        }
    }

    /// Returns a new `MemberFilter` that only builds contracts.
    pub fn only_contracts() -> Self {
        Self {
            build_contracts: true,
            build_scripts: false,
            build_predicates: false,
            build_libraries: false,
        }
    }

    /// Returns a new `MemberFilter`, that only builds predicates.
    pub fn only_predicates() -> Self {
        Self {
            build_contracts: false,
            build_scripts: false,
            build_predicates: true,
            build_libraries: false,
        }
    }

    /// Filter given target of output nodes according to the this `MemberFilter`.
    pub fn filter_outputs(
        &self,
        build_plan: &BuildPlan,
        outputs: HashSet<NodeIx>,
    ) -> HashSet<NodeIx> {
        let graph = build_plan.graph();
        let manifest_map = build_plan.manifest_map();
        outputs
            .into_iter()
            .filter(|&node_ix| {
                let pkg = &graph[node_ix];
                let pkg_manifest = &manifest_map[&pkg.id()];
                let program_type = pkg_manifest.program_type();
                // Since parser cannot recover for program type detection, for the scenarios that
                // parser fails to parse the code, program type detection is not possible. So in
                // failing to parse cases we should try to build at least until
                // https://github.com/FuelLabs/sway/issues/3017 is fixed. Until then we should
                // build those members because of two reasons:
                //
                // 1. The member could already be from the desired member type
                // 2. If we do not try to build there is no way users can know there is a code
                //    piece failing to be parsed in their workspace.
                match program_type {
                    Ok(program_type) => match program_type {
                        TreeType::Predicate => self.build_predicates,
                        TreeType::Script => self.build_scripts,
                        TreeType::Contract => self.build_contracts,
                        TreeType::Library => self.build_libraries,
                    },
                    Err(_) => true,
                }
            })
            .collect()
    }
}

impl BuildOpts {
    /// Return a `BuildOpts` with modified `tests` field.
    pub fn include_tests(self, include_tests: bool) -> Self {
        Self {
            tests: include_tests,
            ..self
        }
    }
}

impl Edge {
    pub fn new(name: String, kind: DepKind) -> Edge {
        Edge { name, kind }
    }
}

impl BuiltPackage {
    /// Writes bytecode of the BuiltPackage to the given `path`.
    pub fn write_bytecode(&self, path: &Path) -> Result<()> {
        fs::write(path, &self.bytecode.bytes)?;
        Ok(())
    }

    pub fn write_hexcode(&self, path: &Path) -> Result<()> {
        let hex_file = serde_json::json!({
            "hex": format!("0x{}", hex::encode(&self.bytecode.bytes)),
        });

        fs::write(path, hex_file.to_string())?;
        Ok(())
    }

    /// Writes debug_info (source_map) of the BuiltPackage to the given `out_file`.
    pub fn write_debug_info(&self, out_file: &Path) -> Result<()> {
        if matches!(out_file.extension(), Some(ext) if ext == "json") {
            let source_map_json =
                serde_json::to_vec(&self.source_map).expect("JSON serialization failed");
            fs::write(out_file, source_map_json)?;
        } else {
            let primary_dir = self.descriptor.manifest_file.dir();
            let primary_src = self.descriptor.manifest_file.entry_path();
            write_dwarf(&self.source_map, primary_dir, &primary_src, out_file)?;
        }
        Ok(())
    }

    pub fn json_abi_string(&self, minify_json_abi: bool) -> Result<Option<String>> {
        match &self.program_abi {
            ProgramABI::Fuel(program_abi) => {
                if !program_abi.functions.is_empty() {
                    let json_string = if minify_json_abi {
                        serde_json::to_string(&program_abi)
                    } else {
                        serde_json::to_string_pretty(&program_abi)
                    }?;
                    Ok(Some(json_string))
                } else {
                    Ok(None)
                }
            }
            ProgramABI::Evm(program_abi) => {
                if !program_abi.is_empty() {
                    let json_string = if minify_json_abi {
                        serde_json::to_string(&program_abi)
                    } else {
                        serde_json::to_string_pretty(&program_abi)
                    }?;
                    Ok(Some(json_string))
                } else {
                    Ok(None)
                }
            }
            // TODO?
            ProgramABI::MidenVM(()) => Ok(None),
        }
    }

    /// Writes the ABI in JSON format to the given `path`.
    pub fn write_json_abi(&self, path: &Path, minify: &MinifyOpts) -> Result<()> {
        if let Some(json_abi_string) = self.json_abi_string(minify.json_abi)? {
            let mut file = File::create(path)?;
            file.write_all(json_abi_string.as_bytes())?;
        }
        Ok(())
    }

    /// Writes BuiltPackage to `output_dir`.
    pub fn write_output(
        &self,
        minify: &MinifyOpts,
        pkg_name: &str,
        output_dir: &Path,
    ) -> Result<()> {
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }
        // Place build artifacts into the output directory.
        let bin_path = output_dir.join(pkg_name).with_extension("bin");

        self.write_bytecode(&bin_path)?;

        let program_abi_stem = format!("{pkg_name}-abi");
        let json_abi_path = output_dir.join(program_abi_stem).with_extension("json");
        self.write_json_abi(&json_abi_path, minify)?;

        debug!(
            "      Bytecode size: {} bytes ({})",
            self.bytecode.bytes.len(),
            format_bytecode_size(self.bytecode.bytes.len())
        );

        // Additional ops required depending on the program type
        match self.tree_type {
            TreeType::Contract => {
                // For contracts, emit a JSON file with all the initialized storage slots.
                let storage_slots_stem = format!("{pkg_name}-storage_slots");
                let storage_slots_path = output_dir.join(storage_slots_stem).with_extension("json");
                let storage_slots_file = File::create(storage_slots_path)?;
                let res = if minify.json_storage_slots {
                    serde_json::to_writer(&storage_slots_file, &self.storage_slots)
                } else {
                    serde_json::to_writer_pretty(&storage_slots_file, &self.storage_slots)
                };

                res?;
            }
            TreeType::Predicate => {
                // Get the root hash of the bytecode for predicates and store the result in a file in the output directory
                let root = format!(
                    "0x{}",
                    fuel_tx::Input::predicate_owner(&self.bytecode.bytes)
                );
                let root_file_name = format!("{}{}", &pkg_name, SWAY_BIN_ROOT_SUFFIX);
                let root_path = output_dir.join(root_file_name);
                fs::write(root_path, &root)?;
                info!("      Predicate root: {}", root);
            }
            TreeType::Script => {
                // hash the bytecode for scripts and store the result in a file in the output directory
                let bytecode_hash =
                    format!("0x{}", fuel_crypto::Hasher::hash(&self.bytecode.bytes));
                let hash_file_name = format!("{}{}", &pkg_name, SWAY_BIN_HASH_SUFFIX);
                let hash_path = output_dir.join(hash_file_name);
                fs::write(hash_path, &bytecode_hash)?;
                debug!("      Bytecode hash: {}", bytecode_hash);
            }
            _ => (),
        }

        Ok(())
    }
}

impl Built {
    /// Returns an iterator yielding all member built packages.
    pub fn into_members<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (&'a Pinned, Arc<BuiltPackage>)> + 'a> {
        // NOTE: Since pkg is a `Arc<_>`, pkg clones in this function are only reference
        // increments. `BuiltPackage` struct does not get copied.`
        match self {
            Built::Package(pkg) => {
                let pinned = &pkg.as_ref().descriptor.pinned;
                let pkg = pkg.clone();
                Box::new(std::iter::once((pinned, pkg)))
            }
            Built::Workspace(workspace) => Box::new(
                workspace
                    .iter()
                    .map(|pkg| (&pkg.descriptor.pinned, pkg.clone())),
            ),
        }
    }

    /// Tries to retrieve the `Built` as a `BuiltPackage`.
    pub fn expect_pkg(self) -> Result<Arc<BuiltPackage>> {
        match self {
            Built::Package(built_pkg) => Ok(built_pkg),
            Built::Workspace(_) => bail!("expected `Built` to be `Built::Package`"),
        }
    }
}

impl BuildPlan {
    /// Create a new build plan for the project from the build options provided.
    ///
    /// To do so, it tries to read the manifet file at the target path and creates the plan with
    /// `BuildPlan::from_lock_and_manifest`.
    pub fn from_pkg_opts(pkg_options: &PkgOpts) -> Result<Self> {
        let path = &pkg_options.path;

        let manifest_dir = if let Some(ref path) = path {
            PathBuf::from(path)
        } else {
            std::env::current_dir()?
        };

        let manifest_file = ManifestFile::from_dir(manifest_dir)?;
        let member_manifests = manifest_file.member_manifests()?;
        // Check if we have members to build so that we are not trying to build an empty workspace.
        if member_manifests.is_empty() {
            bail!("No member found to build")
        }
        let lock_path = manifest_file.lock_path()?;
        Self::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            pkg_options.locked,
            pkg_options.offline,
            &pkg_options.ipfs_node,
        )
    }

    /// Create a new build plan for the project by fetching and pinning all dependencies.
    ///
    /// To account for an existing lock file, use `from_lock_and_manifest` instead.
    pub fn from_manifests(
        manifests: &MemberManifestFiles,
        offline: bool,
        ipfs_node: &IPFSNode,
    ) -> Result<Self> {
        // Check toolchain version
        validate_version(manifests)?;
        let mut graph = Graph::default();
        let mut manifest_map = ManifestMap::default();
        fetch_graph(manifests, offline, ipfs_node, &mut graph, &mut manifest_map)?;
        // Validate the graph, since we constructed the graph from scratch the paths will not be a
        // problem but the version check is still needed
        validate_graph(&graph, manifests)?;
        let compilation_order = compilation_order(&graph)?;
        Ok(Self {
            graph,
            manifest_map,
            compilation_order,
        })
    }

    /// Create a new build plan taking into account the state of both the PackageManifest and the existing
    /// lock file if there is one.
    ///
    /// This will first attempt to load a build plan from the lock file and validate the resulting
    /// graph using the current state of the PackageManifest.
    ///
    /// This includes checking if the [dependencies] or [patch] tables have changed and checking
    /// the validity of the local path dependencies. If any changes are detected, the graph is
    /// updated and any new packages that require fetching are fetched.
    ///
    /// The resulting build plan should always be in a valid state that is ready for building or
    /// checking.
    // TODO: Currently (if `--locked` isn't specified) this writes the updated lock directly. This
    // probably should not be the role of the `BuildPlan` constructor - instead, we should return
    // the manifest alongside some lock diff type that can be used to optionally write the updated
    // lock file and print the diff.
    pub fn from_lock_and_manifests(
        lock_path: &Path,
        manifests: &MemberManifestFiles,
        locked: bool,
        offline: bool,
        ipfs_node: &IPFSNode,
    ) -> Result<Self> {
        // Check toolchain version
        validate_version(manifests)?;
        // Keep track of the cause for the new lock file if it turns out we need one.
        let mut new_lock_cause = None;

        // First, attempt to load the lock.
        let lock = Lock::from_path(lock_path).unwrap_or_else(|e| {
            new_lock_cause = if e.to_string().contains("No such file or directory") {
                Some(anyhow!("lock file did not exist"))
            } else {
                Some(e)
            };
            Lock::default()
        });

        // Next, construct the package graph from the lock.
        let mut graph = lock.to_graph().unwrap_or_else(|e| {
            new_lock_cause = Some(anyhow!("Invalid lock: {}", e));
            Graph::default()
        });

        // Since the lock file was last created there are many ways in which it might have been
        // invalidated. E.g. a package's manifest `[dependencies]` table might have changed, a user
        // might have edited the `Forc.lock` file when they shouldn't have, a path dependency no
        // longer exists at its specified location, etc. We must first remove all invalid nodes
        // before we can determine what we need to fetch.
        let invalid_deps = validate_graph(&graph, manifests)?;
        let members: HashSet<String> = manifests
            .iter()
            .map(|(member_name, _)| member_name.clone())
            .collect();
        remove_deps(&mut graph, &members, &invalid_deps);

        // We know that the remaining nodes have valid paths, otherwise they would have been
        // removed. We can safely produce an initial `manifest_map`.
        let mut manifest_map = graph_to_manifest_map(manifests, &graph)?;

        // Attempt to fetch the remainder of the graph.
        let _added = fetch_graph(manifests, offline, ipfs_node, &mut graph, &mut manifest_map)?;

        // Determine the compilation order.
        let compilation_order = compilation_order(&graph)?;

        let plan = Self {
            graph,
            manifest_map,
            compilation_order,
        };

        // Construct the new lock and check the diff.
        let new_lock = Lock::from_graph(plan.graph());
        let lock_diff = new_lock.diff(&lock);
        if !lock_diff.removed.is_empty() || !lock_diff.added.is_empty() {
            new_lock_cause.get_or_insert(anyhow!("lock file did not match manifest"));
        }

        // If there was some change in the lock file, write the new one and print the cause.
        if let Some(cause) = new_lock_cause {
            if locked {
                bail!(
                    "The lock file {} needs to be updated (Cause: {}) \
                    but --locked was passed to prevent this.",
                    lock_path.to_string_lossy(),
                    cause,
                );
            }
            println_action_green(
                "Creating",
                &format!("a new `Forc.lock` file. (Cause: {})", cause),
            );
            let member_names = manifests
                .iter()
                .map(|(_, manifest)| manifest.project.name.to_string())
                .collect();
            crate::lock::print_diff(&member_names, &lock_diff);
            let string = toml::ser::to_string_pretty(&new_lock)
                .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
            fs::write(lock_path, string)
                .map_err(|e| anyhow!("failed to write lock file: {}", e))?;
            debug!("   Created new lock file at {}", lock_path.display());
        }

        Ok(plan)
    }

    /// Produce an iterator yielding all contract dependencies of given node in the order of
    /// compilation.
    pub fn contract_dependencies(&self, node: NodeIx) -> impl Iterator<Item = NodeIx> + '_ {
        let graph = self.graph();
        let connected: HashSet<_> = Dfs::new(graph, node).iter(graph).collect();
        self.compilation_order()
            .iter()
            .cloned()
            .filter(move |&n| n != node)
            .filter(|&n| {
                graph
                    .edges_directed(n, Direction::Incoming)
                    .any(|edge| matches!(edge.weight().kind, DepKind::Contract { .. }))
            })
            .filter(move |&n| connected.contains(&n))
    }

    /// Produce an iterator yielding all workspace member nodes in order of compilation.
    ///
    /// In the case that this [BuildPlan] was constructed for a single package,
    /// only that package's node will be yielded.
    pub fn member_nodes(&self) -> impl Iterator<Item = NodeIx> + '_ {
        self.compilation_order()
            .iter()
            .copied()
            .filter(|&n| self.graph[n].source == source::Pinned::MEMBER)
    }

    /// Produce an iterator yielding all workspace member pinned pkgs in order of compilation.
    ///
    /// In the case that this `BuildPlan` was constructed for a single package,
    /// only that package's pinned pkg will be yielded.
    pub fn member_pinned_pkgs(&self) -> impl Iterator<Item = Pinned> + '_ {
        let graph = self.graph();
        self.member_nodes().map(|node| &graph[node]).cloned()
    }

    /// View the build plan's compilation graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// View the build plan's map of pinned package IDs to their associated manifest.
    pub fn manifest_map(&self) -> &ManifestMap {
        &self.manifest_map
    }

    /// The order in which nodes are compiled, determined via a toposort of the package graph.
    pub fn compilation_order(&self) -> &[NodeIx] {
        &self.compilation_order
    }

    /// Produce the node index of the member with the given name.
    pub fn find_member_index(&self, member_name: &str) -> Option<NodeIx> {
        self.member_nodes()
            .find(|node_ix| self.graph[*node_ix].name == member_name)
    }

    /// Produce an iterator yielding indices for the given node and its dependencies in BFS order.
    pub fn node_deps(&self, n: NodeIx) -> impl '_ + Iterator<Item = NodeIx> {
        let bfs = Bfs::new(&self.graph, n);
        // Return an iterator yielding visitable nodes from the given node.
        bfs.iter(&self.graph)
    }

    /// Produce an iterator yielding build profiles from the member nodes of this BuildPlan.
    pub fn build_profiles(&self) -> impl '_ + Iterator<Item = (String, BuildProfile)> {
        let manifest_map = &self.manifest_map;
        let graph = &self.graph;
        self.member_nodes().flat_map(|member_node| {
            manifest_map[&graph[member_node].id()]
                .build_profiles()
                .map(|(n, p)| (n.clone(), p.clone()))
        })
    }

    /// Returns a salt for the given pinned package if it is a contract and `None` for libraries.
    pub fn salt(&self, pinned: &Pinned) -> Option<fuel_tx::Salt> {
        let graph = self.graph();
        let node_ix = graph
            .node_indices()
            .find(|node_ix| graph[*node_ix] == *pinned);
        node_ix.and_then(|node| {
            graph
                .edges_directed(node, Direction::Incoming)
                .map(|e| match e.weight().kind {
                    DepKind::Library => None,
                    DepKind::Contract { salt } => Some(salt),
                })
                .next()
                .flatten()
        })
    }

    /// Returns a [String] representing the build dependency graph in GraphViz DOT format.
    pub fn visualize(&self, url_file_prefix: Option<String>) -> String {
        format!(
            "{:?}",
            dot::Dot::with_attr_getters(
                &self.graph,
                &[dot::Config::NodeNoLabel, dot::Config::EdgeNoLabel],
                &|_, _| String::new(),
                &|_, nr| {
                    let url = url_file_prefix.clone().map_or(String::new(), |prefix| {
                        self.manifest_map
                            .get(&nr.1.id())
                            .map_or(String::new(), |manifest| {
                                format!("URL = \"{}{}\"", prefix, manifest.path().to_string_lossy())
                            })
                    });
                    format!("label = \"{}\" shape = box {url}", nr.1.name)
                },
            )
        )
    }
}

/// Given a graph and the known project name retrieved from the manifest, produce an iterator
/// yielding any nodes from the graph that might potentially be a project node.
fn potential_proj_nodes<'a>(g: &'a Graph, proj_name: &'a str) -> impl 'a + Iterator<Item = NodeIx> {
    member_nodes(g).filter(move |&n| g[n].name == proj_name)
}

/// Given a graph, find the project node.
///
/// This should be the only node that satisfies the following conditions:
///
/// - The package name matches `proj_name`
/// - The node has no incoming edges, i.e. is not a dependency of another node.
fn find_proj_node(graph: &Graph, proj_name: &str) -> Result<NodeIx> {
    let mut potentials = potential_proj_nodes(graph, proj_name);
    let proj_node = potentials
        .next()
        .ok_or_else(|| anyhow!("graph contains no project node"))?;
    match potentials.next() {
        None => Ok(proj_node),
        Some(_) => Err(anyhow!("graph contains more than one project node")),
    }
}

/// Checks if the toolchain version is in compliance with minimum implied by `manifest`.
///
/// If the `manifest` is a ManifestFile::Workspace, check all members of the workspace for version
/// validation. Otherwise only the given package is checked.
fn validate_version(member_manifests: &MemberManifestFiles) -> Result<()> {
    for member_pkg_manifest in member_manifests.values() {
        validate_pkg_version(member_pkg_manifest)?;
    }
    Ok(())
}

/// Check minimum forc version given in the package manifest file
///
/// If required minimum forc version is higher than current forc version return an error with
/// upgrade instructions
fn validate_pkg_version(pkg_manifest: &PackageManifestFile) -> Result<()> {
    if let Some(min_forc_version) = &pkg_manifest.project.forc_version {
        // Get the current version of the toolchain
        let crate_version = env!("CARGO_PKG_VERSION");
        let toolchain_version = semver::Version::parse(crate_version)?;
        if toolchain_version < *min_forc_version {
            bail!(
                "{:?} requires forc version {} but current forc version is {}\nUpdate the toolchain by following: https://fuellabs.github.io/sway/v{}/introduction/installation.html",
                pkg_manifest.project.name,
                min_forc_version,
                crate_version,
                crate_version
            );
        }
    };
    Ok(())
}

fn member_nodes(g: &Graph) -> impl Iterator<Item = NodeIx> + '_ {
    g.node_indices()
        .filter(|&n| g[n].source == source::Pinned::MEMBER)
}

/// Validates the state of the pinned package graph against the given ManifestFile.
///
/// Returns the set of invalid dependency edges.
fn validate_graph(graph: &Graph, manifests: &MemberManifestFiles) -> Result<BTreeSet<EdgeIx>> {
    let mut member_pkgs: HashMap<&String, &PackageManifestFile> = manifests.iter().collect();
    let member_nodes: Vec<_> = member_nodes(graph)
        .filter_map(|n| {
            member_pkgs
                .remove(&graph[n].name.to_string())
                .map(|pkg| (n, pkg))
        })
        .collect();

    // If no member nodes, the graph is either empty or corrupted. Remove all edges.
    if member_nodes.is_empty() {
        return Ok(graph.edge_indices().collect());
    }

    let mut visited = HashSet::new();
    let edges = member_nodes
        .into_iter()
        .flat_map(move |(n, _)| validate_deps(graph, n, manifests, &mut visited))
        .collect();

    Ok(edges)
}

/// Recursively validate all dependencies of the given `node`.
///
/// Returns the set of invalid dependency edges.
fn validate_deps(
    graph: &Graph,
    node: NodeIx,
    manifests: &MemberManifestFiles,
    visited: &mut HashSet<NodeIx>,
) -> BTreeSet<EdgeIx> {
    let mut remove = BTreeSet::default();
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_name = edge.weight();
        let dep_node = edge.target();
        match validate_dep(graph, manifests, dep_name, dep_node) {
            Err(_) => {
                remove.insert(edge.id());
            }
            Ok(_) => {
                if visited.insert(dep_node) {
                    let rm = validate_deps(graph, dep_node, manifests, visited);
                    remove.extend(rm);
                }
                continue;
            }
        }
    }
    remove
}

/// Check the validity of a node's dependency within the graph.
///
/// Returns the `ManifestFile` in the case that the dependency is valid.
fn validate_dep(
    graph: &Graph,
    manifests: &MemberManifestFiles,
    dep_edge: &Edge,
    dep_node: NodeIx,
) -> Result<PackageManifestFile> {
    let dep_name = &dep_edge.name;
    let node_manifest = manifests
        .get(dep_name)
        .ok_or_else(|| anyhow!("Couldn't find manifest file for {}", dep_name))?;
    // Check the validity of the dependency path, including its path root.
    let dep_path = dep_path(graph, node_manifest, dep_node, manifests).map_err(|e| {
        anyhow!(
            "failed to construct path for dependency {:?}: {}",
            dep_name,
            e
        )
    })?;

    // Ensure the manifest is accessible.
    let dep_manifest = PackageManifestFile::from_dir(&dep_path)?;

    // Check that the dependency's source matches the entry in the parent manifest.
    let dep_entry = node_manifest
        .dep(dep_name)
        .ok_or_else(|| anyhow!("no entry in parent manifest"))?;
    let dep_source =
        Source::from_manifest_dep_patched(node_manifest, dep_name, dep_entry, manifests)?;
    let dep_pkg = graph[dep_node].unpinned(&dep_path);
    if dep_pkg.source != dep_source {
        bail!("dependency node's source does not match manifest entry");
    }

    validate_dep_manifest(&graph[dep_node], &dep_manifest, dep_edge)?;

    Ok(dep_manifest)
}
/// Part of dependency validation, any checks related to the dependency's manifest content.
fn validate_dep_manifest(
    dep: &Pinned,
    dep_manifest: &PackageManifestFile,
    dep_edge: &Edge,
) -> Result<()> {
    let dep_program_type = dep_manifest.program_type()?;
    // Check if the dependency is either a library or a contract declared as a contract dependency
    match (&dep_program_type, &dep_edge.kind) {
        (TreeType::Contract, DepKind::Contract { salt: _ })
        | (TreeType::Library, DepKind::Library) => {}
        _ => bail!(
            "\"{}\" is declared as a {} dependency, but is actually a {}",
            dep.name,
            dep_edge.kind,
            dep_program_type
        ),
    }
    // Ensure the name matches the manifest project name.
    if dep.name != dep_manifest.project.name {
        bail!(
            "dependency name {:?} must match the manifest project name {:?} \
            unless `package = {:?}` is specified in the dependency declaration",
            dep.name,
            dep_manifest.project.name,
            dep_manifest.project.name,
        );
    }
    validate_pkg_version(dep_manifest)?;
    Ok(())
}

/// Returns the canonical, local path to the given dependency node if it exists, `None` otherwise.
///
/// Also returns `Err` in the case that the dependency is a `Path` dependency and the path root is
/// invalid.
fn dep_path(
    graph: &Graph,
    node_manifest: &PackageManifestFile,
    dep_node: NodeIx,
    manifests: &MemberManifestFiles,
) -> Result<PathBuf> {
    let dep = &graph[dep_node];
    let dep_name = &dep.name;
    match dep.source.dep_path(&dep.name)? {
        source::DependencyPath::ManifestPath(path) => Ok(path),
        source::DependencyPath::Root(path_root) => {
            validate_path_root(graph, dep_node, path_root)?;

            // Check if the path is directly from the dependency.
            if let Some(path) = node_manifest.dep_path(dep_name) {
                if path.exists() {
                    return Ok(path);
                }
            }

            // Otherwise, check if it comes from a patch.
            for (_, patch_map) in node_manifest.patches() {
                if let Some(Dependency::Detailed(details)) = patch_map.get(&dep_name.to_string()) {
                    if let Some(ref rel_path) = details.path {
                        if let Ok(path) = node_manifest.dir().join(rel_path).canonicalize() {
                            if path.exists() {
                                return Ok(path);
                            }
                        }
                    }
                }
            }

            bail!(
                "no dependency or patch with name {:?} in manifest of {:?}",
                dep_name,
                node_manifest.project.name
            )
        }
        source::DependencyPath::Member => {
            // If a node has a root dependency it is a member of the workspace.
            manifests
                .values()
                .find(|manifest| manifest.project.name == *dep_name)
                .map(|manifest| manifest.path().to_path_buf())
                .ok_or_else(|| anyhow!("cannot find dependency in the workspace"))
        }
    }
}

/// Remove the given set of dependency edges from the `graph`.
///
/// Also removes all nodes that are no longer connected to any root node as a result.
fn remove_deps(
    graph: &mut Graph,
    member_names: &HashSet<String>,
    edges_to_remove: &BTreeSet<EdgeIx>,
) {
    // Retrieve the project nodes for workspace members.
    let member_nodes: HashSet<_> = member_nodes(graph)
        .filter(|&n| member_names.contains(&graph[n].name.to_string()))
        .collect();

    // Before removing edges, sort the nodes in order of dependency for the node removal pass.
    let node_removal_order = if let Ok(nodes) = petgraph::algo::toposort(&*graph, None) {
        nodes
    } else {
        // If toposort fails the given graph is cyclic, so invalidate everything.
        graph.clear();
        return;
    };

    // Remove the given set of dependency edges.
    for &edge in edges_to_remove {
        graph.remove_edge(edge);
    }

    // Remove all nodes that are no longer connected to any project node as a result.
    let nodes = node_removal_order.into_iter();
    for node in nodes {
        if !has_parent(graph, node) && !member_nodes.contains(&node) {
            graph.remove_node(node);
        }
    }
}

fn has_parent(graph: &Graph, node: NodeIx) -> bool {
    graph
        .edges_directed(node, Direction::Incoming)
        .next()
        .is_some()
}

impl Pinned {
    /// Retrieve the unique ID for the pinned package.
    ///
    /// The internal value is produced by hashing the package's name and `source::Pinned`.
    pub fn id(&self) -> PinnedId {
        PinnedId::new(&self.name, &self.source)
    }

    /// Retrieve the unpinned version of this source.
    pub fn unpinned(&self, path: &Path) -> Pkg {
        let source = self.source.unpinned(path);
        let name = self.name.clone();
        Pkg { name, source }
    }
}

impl PinnedId {
    /// Hash the given name and pinned source to produce a unique pinned package ID.
    pub fn new(name: &str, source: &source::Pinned) -> Self {
        let mut hasher = hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        source.hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl fmt::Display for DepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepKind::Library => write!(f, "library"),
            DepKind::Contract { .. } => write!(f, "contract"),
        }
    }
}

impl fmt::Display for PinnedId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format the inner `u64` as hex.
        write!(f, "{:016X}", self.0)
    }
}

impl FromStr for PinnedId {
    type Err = PinnedIdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            u64::from_str_radix(s, 16).map_err(|_| PinnedIdParseError)?,
        ))
    }
}

/// The `pkg::Graph` is of *a -> b* where *a* depends on *b*. We can determine compilation order by
/// performing a toposort of the graph with reversed edges. The resulting order ensures all
/// dependencies are always compiled before their dependents.
pub fn compilation_order(graph: &Graph) -> Result<Vec<NodeIx>> {
    let rev_pkg_graph = petgraph::visit::Reversed(&graph);
    petgraph::algo::toposort(rev_pkg_graph, None).map_err(|_| {
        // Find the strongly connected components.
        // If the vector has an element with length > 1, it contains a cyclic path.
        let scc = petgraph::algo::kosaraju_scc(&graph);
        let mut path = String::new();
        scc.iter()
            .filter(|path| path.len() > 1)
            .for_each(|cyclic_path| {
                // We are sure that there is an element in cyclic_path vec.
                let starting_node = &graph[*cyclic_path.last().unwrap()];

                // Adding first node of the path
                path.push_str(&starting_node.name.to_string());
                path.push_str(" -> ");

                for (node_index, node) in cyclic_path.iter().enumerate() {
                    path.push_str(&graph[*node].name.to_string());
                    if node_index != cyclic_path.len() - 1 {
                        path.push_str(" -> ");
                    }
                }
                path.push('\n');
            });
        anyhow!("dependency cycle detected: {}", path)
    })
}

/// Given a graph collects ManifestMap while taking in to account that manifest can be a
/// ManifestFile::Workspace. In the case of a workspace each pkg manifest map is collected and
/// their added node lists are merged.
fn graph_to_manifest_map(manifests: &MemberManifestFiles, graph: &Graph) -> Result<ManifestMap> {
    let mut manifest_map = HashMap::new();
    for pkg_manifest in manifests.values() {
        let pkg_name = &pkg_manifest.project.name;
        manifest_map.extend(pkg_graph_to_manifest_map(manifests, pkg_name, graph)?);
    }
    Ok(manifest_map)
}

/// Given a graph of pinned packages and the project manifest, produce a map containing the
/// manifest of for every node in the graph.
///
/// Assumes the given `graph` only contains valid dependencies (see `validate_graph`).
///
/// `pkg_graph_to_manifest_map` starts from each node (which corresponds to the given proj_manifest)
/// and visits children to collect their manifest files.
fn pkg_graph_to_manifest_map(
    manifests: &MemberManifestFiles,
    pkg_name: &str,
    graph: &Graph,
) -> Result<ManifestMap> {
    let proj_manifest = manifests
        .get(pkg_name)
        .ok_or_else(|| anyhow!("Cannot find manifest for {}", pkg_name))?;
    let mut manifest_map = ManifestMap::new();

    // Traverse the graph from the project node.
    let Ok(proj_node) = find_proj_node(graph, &proj_manifest.project.name) else {
        return Ok(manifest_map);
    };
    let proj_id = graph[proj_node].id();
    manifest_map.insert(proj_id, proj_manifest.clone());

    // Resolve all parents before their dependencies as we require the parent path to construct the
    // dependency path. Skip the already added project node at the beginning of traversal.
    let mut bfs = Bfs::new(graph, proj_node);
    bfs.next(graph);
    while let Some(dep_node) = bfs.next(graph) {
        // Retrieve the parent node whose manifest is already stored.
        let (parent_manifest, dep_name) = graph
            .edges_directed(dep_node, Direction::Incoming)
            .find_map(|edge| {
                let parent_node = edge.source();
                let dep_name = &edge.weight().name;
                let parent = &graph[parent_node];
                let parent_manifest = manifest_map.get(&parent.id())?;
                Some((parent_manifest, dep_name))
            })
            .ok_or_else(|| anyhow!("more than one root package detected in graph"))?;
        let dep_path = dep_path(graph, parent_manifest, dep_node, manifests).map_err(|e| {
            anyhow!(
                "failed to construct path for dependency {:?}: {}",
                dep_name,
                e
            )
        })?;
        let dep_manifest = PackageManifestFile::from_dir(&dep_path)?;
        let dep = &graph[dep_node];
        manifest_map.insert(dep.id(), dep_manifest);
    }

    Ok(manifest_map)
}

/// Given a `graph`, the node index of a path dependency within that `graph`, and the supposed
/// `path_root` of the path dependency, ensure that the `path_root` is valid.
///
/// See the `path_root` field of the [SourcePathPinned] type for further details.
fn validate_path_root(graph: &Graph, path_dep: NodeIx, path_root: PinnedId) -> Result<()> {
    let path_root_node = find_path_root(graph, path_dep)?;
    if graph[path_root_node].id() != path_root {
        bail!(
            "invalid `path_root` for path dependency package {:?}",
            &graph[path_dep].name
        )
    }
    Ok(())
}

/// Given any node in the graph, find the node that is the path root for that node.
fn find_path_root(graph: &Graph, mut node: NodeIx) -> Result<NodeIx> {
    loop {
        let pkg = &graph[node];
        match pkg.source {
            source::Pinned::Path(ref src) => {
                let parent = graph
                    .edges_directed(node, Direction::Incoming)
                    .next()
                    .map(|edge| edge.source())
                    .ok_or_else(|| {
                        anyhow!(
                            "Failed to find path root: `path` dependency \"{}\" has no parent",
                            src
                        )
                    })?;
                node = parent;
            }
            source::Pinned::Git(_)
            | source::Pinned::Ipfs(_)
            | source::Pinned::Member(_)
            | source::Pinned::Registry(_) => {
                return Ok(node);
            }
        }
    }
}

/// Given an empty or partially completed `graph`, complete the graph.
///
/// If the given `manifest` is of type ManifestFile::Workspace resulting graph will have multiple
/// root nodes, each representing a member of the workspace. Otherwise resulting graph will only
/// have a single root node, representing the package that is described by the ManifestFile::Package
///
/// Checks the created graph after fetching for conflicting salt declarations.
fn fetch_graph(
    member_manifests: &MemberManifestFiles,
    offline: bool,
    ipfs_node: &IPFSNode,
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
) -> Result<HashSet<NodeIx>> {
    let mut added_nodes = HashSet::default();
    for member_pkg_manifest in member_manifests.values() {
        added_nodes.extend(&fetch_pkg_graph(
            member_pkg_manifest,
            offline,
            ipfs_node,
            graph,
            manifest_map,
            member_manifests,
        )?);
    }
    validate_contract_deps(graph)?;
    Ok(added_nodes)
}

/// Given an empty or partially completed package `graph`, complete the graph.
///
/// The given `graph` may be empty, partially complete, or fully complete. All existing nodes
/// should already be confirmed to be valid nodes via `validate_graph`. All invalid nodes should
/// have been removed prior to calling this.
///
/// Recursively traverses dependencies listed within each package's manifest, fetching and pinning
/// each dependency if it does not already exist within the package graph.
///
/// The accompanying `path_map` should contain a path entry for every existing node within the
/// `graph` and will `panic!` otherwise.
///
/// Upon success, returns the set of nodes that were added to the graph during traversal.
fn fetch_pkg_graph(
    proj_manifest: &PackageManifestFile,
    offline: bool,
    ipfs_node: &IPFSNode,
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
    member_manifests: &MemberManifestFiles,
) -> Result<HashSet<NodeIx>> {
    // Retrieve the project node, or create one if it does not exist.
    let proj_node = if let Ok(proj_node) = find_proj_node(graph, &proj_manifest.project.name) {
        proj_node
    } else {
        let name = proj_manifest.project.name.clone();
        let source = source::Pinned::MEMBER;
        let pkg = Pinned { name, source };
        let pkg_id = pkg.id();
        manifest_map.insert(pkg_id, proj_manifest.clone());
        graph.add_node(pkg)
    };

    // Traverse the rest of the graph from the root.
    let fetch_ts = std::time::Instant::now();
    let fetch_id = source::fetch_id(proj_manifest.dir(), fetch_ts);
    let path_root = graph[proj_node].id();
    let mut fetched = graph
        .node_indices()
        .map(|n| {
            let pinned = &graph[n];
            let manifest = &manifest_map[&pinned.id()];
            let pkg = pinned.unpinned(manifest.dir());
            (pkg, n)
        })
        .collect();
    let mut visited = HashSet::default();
    fetch_deps(
        fetch_id,
        offline,
        ipfs_node,
        proj_node,
        path_root,
        graph,
        manifest_map,
        &mut fetched,
        &mut visited,
        member_manifests,
    )
}

/// Visit the unvisited dependencies of the given node and fetch missing nodes as necessary.
///
/// Assumes the `node`'s manifest already exists within the `manifest_map`.
#[allow(clippy::too_many_arguments)]
fn fetch_deps(
    fetch_id: u64,
    offline: bool,
    ipfs_node: &IPFSNode,
    node: NodeIx,
    path_root: PinnedId,
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
    fetched: &mut HashMap<Pkg, NodeIx>,
    visited: &mut HashSet<NodeIx>,
    member_manifests: &MemberManifestFiles,
) -> Result<HashSet<NodeIx>> {
    let mut added = HashSet::default();
    let parent_id = graph[node].id();
    let package_manifest = &manifest_map[&parent_id];
    // If the current package is a contract, we need to first get the deployment dependencies
    let deps: Vec<(String, Dependency, DepKind)> = package_manifest
        .contract_deps()
        .map(|(n, d)| {
            (
                n.clone(),
                d.dependency.clone(),
                DepKind::Contract { salt: d.salt.0 },
            )
        })
        .chain(
            package_manifest
                .deps()
                .map(|(n, d)| (n.clone(), d.clone(), DepKind::Library)),
        )
        .collect();
    for (dep_name, dep, dep_kind) in deps {
        let name = dep.package().unwrap_or(&dep_name);
        let parent_manifest = &manifest_map[&parent_id];
        let source =
            Source::from_manifest_dep_patched(parent_manifest, name, &dep, member_manifests)
                .context(format!("Failed to source dependency: {dep_name}"))?;

        // If we haven't yet fetched this dependency, fetch it, pin it and add it to the graph.
        let dep_pkg = Pkg {
            name: name.to_string(),
            source,
        };
        let dep_node = match fetched.entry(dep_pkg) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let pkg = entry.key();
                let ctx = source::PinCtx {
                    fetch_id,
                    path_root,
                    name: &pkg.name,
                    offline,
                    ipfs_node,
                };
                let source = pkg.source.pin(ctx, manifest_map)?;
                let name = pkg.name.clone();
                let dep_pinned = Pinned { name, source };
                let dep_node = graph.add_node(dep_pinned);
                added.insert(dep_node);
                *entry.insert(dep_node)
            }
        };

        let dep_edge = Edge::new(dep_name.to_string(), dep_kind.clone());
        // Ensure we have an edge to the dependency.
        graph.update_edge(node, dep_node, dep_edge.clone());

        // If we've visited this node during this traversal already, no need to traverse it again.
        if !visited.insert(dep_node) {
            continue;
        }

        let dep_pinned = &graph[dep_node];
        let dep_pkg_id = dep_pinned.id();
        validate_dep_manifest(dep_pinned, &manifest_map[&dep_pkg_id], &dep_edge).map_err(|e| {
            let parent = &graph[node];
            anyhow!(
                "dependency of {:?} named {:?} is invalid: {}",
                parent.name,
                dep_name,
                e
            )
        })?;

        let path_root = match dep_pinned.source {
            source::Pinned::Member(_)
            | source::Pinned::Git(_)
            | source::Pinned::Ipfs(_)
            | source::Pinned::Registry(_) => dep_pkg_id,
            source::Pinned::Path(_) => path_root,
        };

        // Fetch the children.
        added.extend(fetch_deps(
            fetch_id,
            offline,
            ipfs_node,
            dep_node,
            path_root,
            graph,
            manifest_map,
            fetched,
            visited,
            member_manifests,
        )?);
    }
    Ok(added)
}

/// Given a `forc_pkg::BuildProfile`, produce the necessary `sway_core::BuildConfig` required for
/// compilation.
pub fn sway_build_config(
    manifest_dir: &Path,
    entry_path: &Path,
    build_target: BuildTarget,
    build_profile: &BuildProfile,
    dbg_generation: sway_core::DbgGeneration,
) -> Result<sway_core::BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let file_name = find_file_name(manifest_dir, entry_path)?;
    let build_config = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.to_path_buf(),
        build_target,
        dbg_generation,
    )
    .with_print_dca_graph(build_profile.print_dca_graph.clone())
    .with_print_dca_graph_url_format(build_profile.print_dca_graph_url_format.clone())
    .with_print_asm(build_profile.print_asm)
    .with_print_bytecode(
        build_profile.print_bytecode,
        build_profile.print_bytecode_spans,
    )
    .with_print_ir(build_profile.print_ir.clone())
    .with_include_tests(build_profile.include_tests)
    .with_time_phases(build_profile.time_phases)
    .with_profile(build_profile.profile)
    .with_metrics(build_profile.metrics_outfile.clone())
    .with_optimization_level(build_profile.optimization_level);
    Ok(build_config)
}

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
///
/// This function ensures that if `std` exists in the graph (the vastly common case) it is also
/// present within the namespace. This is a necessity for operators to work for example.
///
/// This function also ensures that if `std` exists in the graph,
/// then the std prelude will also be added.
///
/// `contract_id_value` should only be Some when producing the `dependency_namespace` for a contract with tests enabled.
/// This allows us to provide a contract's `CONTRACT_ID` constant to its own unit tests.
#[allow(clippy::too_many_arguments)]
pub fn dependency_namespace(
    lib_namespace_map: &HashMap<NodeIx, namespace::Package>,
    compiled_contract_deps: &CompiledContractDeps,
    graph: &Graph,
    node: NodeIx,
    engines: &Engines,
    contract_id_value: Option<ContractIdConst>,
    program_id: ProgramId,
    experimental: ExperimentalFeatures,
    dbg_generation: sway_core::DbgGeneration,
) -> Result<namespace::Package, vec1::Vec1<CompileError>> {
    // TODO: Clean this up when config-time constants v1 are removed.
    let node_idx = &graph[node];
    let name = Ident::new_no_span(node_idx.name.clone());
    let mut namespace = if let Some(contract_id_value) = contract_id_value {
        namespace::package_with_contract_id(
            engines,
            name.clone(),
            program_id,
            contract_id_value,
            experimental,
            dbg_generation,
        )?
    } else {
        Package::new(name.clone(), None, program_id, false)
    };

    // Add direct dependencies.
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_node = edge.target();
        let dep_name = kebab_to_snake_case(&edge.weight().name);
        let dep_edge = edge.weight();
        let dep_namespace = match dep_edge.kind {
            DepKind::Library => lib_namespace_map
                .get(&dep_node)
                .cloned()
                .expect("no root namespace module")
                .clone(),
            DepKind::Contract { salt } => {
                let dep_contract_id = compiled_contract_deps
                    .get(&dep_node)
                    .map(|dep| contract_id(&dep.bytecode, dep.storage_slots.clone(), &salt))
                    // On `check` we don't compile contracts, so we use a placeholder.
                    .unwrap_or_default();
                // Construct namespace with contract id
                let contract_id_value = format!("0x{dep_contract_id}");
                let node_idx = &graph[dep_node];
                let name = Ident::new_no_span(node_idx.name.clone());
                namespace::package_with_contract_id(
                    engines,
                    name.clone(),
                    program_id,
                    contract_id_value,
                    experimental,
                    dbg_generation,
                )?
            }
        };
        namespace.add_external(dep_name, dep_namespace);
    }

    Ok(namespace)
}

/// Compiles the given package.
///
/// ## Program Types
///
/// Behaviour differs slightly based on the package's program type.
///
/// ### Library Packages
///
/// A Library package will have JSON ABI generated for all publicly exposed `abi`s. The library's
/// namespace is returned as the second argument of the tuple.
///
/// ### Contract
///
/// Contracts will output both their JSON ABI and compiled bytecode.
///
/// ### Script, Predicate
///
/// Scripts and Predicates will be compiled to bytecode and will not emit any JSON ABI.
pub fn compile(
    pkg: &PackageDescriptor,
    profile: &BuildProfile,
    engines: &Engines,
    namespace: namespace::Package,
    source_map: &mut SourceMap,
    experimental: ExperimentalFeatures,
    dbg_generation: DbgGeneration,
) -> Result<CompiledPackage> {
    let mut metrics = PerformanceData::default();

    let entry_path = pkg.manifest_file.entry_path();
    let sway_build_config = sway_build_config(
        pkg.manifest_file.dir(),
        &entry_path,
        pkg.target,
        profile,
        dbg_generation,
    )?;
    let terse_mode = profile.terse;
    let reverse_results = profile.reverse_results;
    let fail = |handler: Handler| {
        let (errors, warnings) = handler.consume();
        print_on_failure(
            engines.se(),
            terse_mode,
            &warnings,
            &errors,
            reverse_results,
        );
        bail!("Failed to compile {}", pkg.name);
    };
    let source = pkg.manifest_file.entry_string()?;

    let handler = Handler::default();

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = time_expr!(
        pkg.name,
        "compile to ast",
        "compile_to_ast",
        sway_core::compile_to_ast(
            &handler,
            engines,
            source,
            namespace,
            Some(&sway_build_config),
            &pkg.name,
            None,
            experimental
        ),
        Some(sway_build_config.clone()),
        metrics
    );

    let programs = match ast_res {
        Err(_) => return fail(handler),
        Ok(programs) => programs,
    };
    let typed_program = match programs.typed.as_ref() {
        Err(_) => return fail(handler),
        Ok(typed_program) => typed_program,
    };

    if profile.print_ast {
        tracing::info!("{:#?}", typed_program);
    }

    let storage_slots = typed_program.storage_slots.clone();
    let tree_type = typed_program.kind.tree_type();

    if handler.has_errors() {
        return fail(handler);
    }

    let asm_res = time_expr!(
        pkg.name,
        "compile ast to asm",
        "compile_ast_to_asm",
        sway_core::ast_to_asm(
            &handler,
            engines,
            &programs,
            &sway_build_config,
            experimental
        ),
        Some(sway_build_config.clone()),
        metrics
    );

    let mut asm = match asm_res {
        Err(_) => return fail(handler),
        Ok(asm) => asm,
    };

    const ENCODING_V0: &str = "0";
    const ENCODING_V1: &str = "1";
    const SPEC_VERSION: &str = "1.1";

    let mut program_abi = match pkg.target {
        BuildTarget::Fuel => {
            let program_abi_res = time_expr!(
                pkg.name,
                "generate JSON ABI program",
                "generate_json_abi",
                fuel_abi::generate_program_abi(
                    &handler,
                    &mut AbiContext {
                        program: typed_program,
                        panic_occurrences: &asm.panic_occurrences,
                        abi_with_callpaths: true,
                        type_ids_to_full_type_str: HashMap::<String, String>::new(),
                    },
                    engines,
                    if experimental.new_encoding {
                        ENCODING_V1.into()
                    } else {
                        ENCODING_V0.into()
                    },
                    SPEC_VERSION.into()
                ),
                Some(sway_build_config.clone()),
                metrics
            );
            let program_abi = match program_abi_res {
                Err(_) => return fail(handler),
                Ok(program_abi) => program_abi,
            };
            ProgramABI::Fuel(program_abi)
        }
        BuildTarget::EVM => {
            // Merge the ABI output of ASM gen with ABI gen to handle internal constructors
            // generated by the ASM backend.
            let mut ops = match &asm.finalized_asm.abi {
                Some(ProgramABI::Evm(ops)) => ops.clone(),
                _ => vec![],
            };

            let abi = time_expr!(
                pkg.name,
                "generate JSON ABI program",
                "generate_json_abi",
                evm_abi::generate_abi_program(typed_program, engines),
                Some(sway_build_config.clone()),
                metrics
            );

            ops.extend(abi);

            ProgramABI::Evm(ops)
        }
    };

    let entries = asm
        .finalized_asm
        .entries
        .iter()
        .map(|finalized_entry| PkgEntry::from_finalized_entry(finalized_entry, engines))
        .collect::<anyhow::Result<_>>()?;

    let bc_res = time_expr!(
        pkg.name,
        "compile asm to bytecode",
        "compile_asm_to_bytecode",
        sway_core::asm_to_bytecode(
            &handler,
            &mut asm,
            source_map,
            engines.se(),
            &sway_build_config
        ),
        Some(sway_build_config.clone()),
        metrics
    );

    let errored = handler.has_errors() || (handler.has_warnings() && profile.error_on_warnings);

    let mut compiled = match bc_res {
        Ok(compiled) if !errored => compiled,
        _ => return fail(handler),
    };

    let (_, warnings) = handler.consume();

    print_warnings(engines.se(), terse_mode, &pkg.name, &warnings, &tree_type);

    // Metadata to be placed into the binary.
    let mut md = [0u8, 0, 0, 0, 0, 0, 0, 0];
    // TODO: This should probably be in `fuel_abi_json::generate_json_abi_program`?
    // If ABI requires knowing config offsets, they should be inputs to ABI gen.
    if let ProgramABI::Fuel(ref mut program_abi) = program_abi {
        let mut configurables_offset = compiled.bytecode.len() as u64;
        if let Some(ref mut configurables) = program_abi.configurables {
            // Filter out all dead configurables (i.e. ones without offsets in the bytecode)
            configurables.retain(|c| {
                compiled
                    .named_data_section_entries_offsets
                    .contains_key(&c.name)
            });
            // Set the actual offsets in the JSON object
            for (config, offset) in &compiled.named_data_section_entries_offsets {
                if *offset < configurables_offset {
                    configurables_offset = *offset;
                }
                if let Some(idx) = configurables.iter().position(|c| &c.name == config) {
                    configurables[idx].offset = *offset;
                }
            }
        }

        md = configurables_offset.to_be_bytes();
    }

    // We know to set the metadata only for fuelvm right now.
    if let BuildTarget::Fuel = pkg.target {
        set_bytecode_configurables_offset(&mut compiled, &md);
    }

    metrics.bytecode_size = compiled.bytecode.len();
    let bytecode = BuiltPackageBytecode {
        bytes: compiled.bytecode,
        entries,
    };
    let compiled_package = CompiledPackage {
        source_map: source_map.clone(),
        program_abi,
        storage_slots,
        tree_type,
        bytecode,
        namespace: typed_program.namespace.current_package_ref().clone(),
        warnings,
        metrics,
    };
    if sway_build_config.profile {
        report_assembly_information(&asm, &compiled_package);
    }

    Ok(compiled_package)
}

/// Reports assembly information for a compiled package to an external `dyno` process through `stdout`.
fn report_assembly_information(
    compiled_asm: &sway_core::CompiledAsm,
    compiled_package: &CompiledPackage,
) {
    // Get the bytes of the compiled package.
    let mut bytes = compiled_package.bytecode.bytes.clone();

    // Attempt to get the data section offset out of the compiled package bytes.
    let data_offset = u64::from_be_bytes(
        bytes
            .iter()
            .skip(8)
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );
    let data_section_size = bytes.len() as u64 - data_offset;

    // Remove the data section from the compiled package bytes.
    bytes.truncate(data_offset as usize);

    // Calculate the unpadded size of each data section section.
    // Implementation based directly on `sway_core::asm_generation::Entry::to_bytes`, referenced here:
    // https://github.com/FuelLabs/sway/blob/afd6a6709e7cb11c676059a5004012cc466e653b/sway-core/src/asm_generation/fuel/data_section.rs#L147
    fn calculate_entry_size(entry: &sway_core::asm_generation::Entry) -> u64 {
        match &entry.value {
            sway_core::asm_generation::Datum::Byte(value) => std::mem::size_of_val(value) as u64,

            sway_core::asm_generation::Datum::Word(value) => std::mem::size_of_val(value) as u64,

            sway_core::asm_generation::Datum::ByteArray(bytes)
            | sway_core::asm_generation::Datum::Slice(bytes) => {
                if bytes.len() % 8 == 0 {
                    bytes.len() as u64
                } else {
                    ((bytes.len() + 7) & 0xfffffff8_usize) as u64
                }
            }

            sway_core::asm_generation::Datum::Collection(items) => {
                items.iter().map(calculate_entry_size).sum()
            }
        }
    }

    // Compute the assembly information to be reported.
    let asm_information = sway_core::asm_generation::AsmInformation {
        bytecode_size: bytes.len() as _,
        data_section: sway_core::asm_generation::DataSectionInformation {
            size: data_section_size,
            used: compiled_asm
                .finalized_asm
                .data_section
                .iter_all_entries()
                .map(|entry| calculate_entry_size(&entry))
                .sum(),
            value_pairs: compiled_asm
                .finalized_asm
                .data_section
                .iter_all_entries()
                .collect(),
        },
    };

    // Report the assembly information to the `dyno` process through `stdout`.
    println!(
        "/dyno info {}",
        serde_json::to_string(&asm_information).unwrap()
    );
}

impl PkgEntry {
    /// Returns whether this `PkgEntry` corresponds to a test.
    pub fn is_test(&self) -> bool {
        self.kind.test().is_some()
    }

    fn from_finalized_entry(finalized_entry: &FinalizedEntry, engines: &Engines) -> Result<Self> {
        let pkg_entry_kind = match &finalized_entry.test_decl_ref {
            Some(test_decl_ref) => {
                let pkg_test_entry = PkgTestEntry::from_decl(test_decl_ref, engines)?;
                PkgEntryKind::Test(pkg_test_entry)
            }
            None => PkgEntryKind::Main,
        };

        Ok(Self {
            finalized: finalized_entry.clone(),
            kind: pkg_entry_kind,
        })
    }
}

impl PkgEntryKind {
    /// Returns `Some` if the `PkgEntryKind` is `Test`.
    pub fn test(&self) -> Option<&PkgTestEntry> {
        match self {
            PkgEntryKind::Test(test) => Some(test),
            _ => None,
        }
    }
}

impl PkgTestEntry {
    fn from_decl(decl_ref: &DeclRefFunction, engines: &Engines) -> Result<Self> {
        fn get_invalid_revert_code_error_msg(
            test_function_name: &Ident,
            should_revert_arg: &AttributeArg,
        ) -> String {
            format!("Invalid revert code for test \"{}\".\nA revert code must be a string containing a \"u64\", e.g.: \"42\".\nThe invalid revert code was: {}.",
                test_function_name,
                should_revert_arg.value.as_ref().expect("`get_string_opt` returned either a value or an error, which means that the invalid value must exist").span().as_str(),
            )
        }

        let span = decl_ref.span();
        let test_function_decl = engines.de().get_function(decl_ref);

        let Some(test_attr) = test_function_decl.attributes.test() else {
            unreachable!("`test_function_decl` is guaranteed to be a test function and it must have a `#[test]` attribute");
        };

        let pass_condition = match test_attr
            .args
            .iter()
            // Last "should_revert" argument wins ;-)
            .rfind(|arg| arg.is_test_should_revert())
        {
            Some(should_revert_arg) => {
                match should_revert_arg.get_string_opt(&Handler::default()) {
                    Ok(should_revert_arg_value) => TestPassCondition::ShouldRevert(
                        should_revert_arg_value
                            .map(|val| val.parse::<u64>())
                            .transpose()
                            .map_err(|_| {
                                anyhow!(get_invalid_revert_code_error_msg(
                                    &test_function_decl.name,
                                    should_revert_arg
                                ))
                            })?,
                    ),
                    Err(_) => bail!(get_invalid_revert_code_error_msg(
                        &test_function_decl.name,
                        should_revert_arg
                    )),
                }
            }
            None => TestPassCondition::ShouldNotRevert,
        };

        let file_path =
            Arc::new(engines.se().get_path(span.source_id().ok_or_else(|| {
                anyhow!("Missing span for test \"{}\".", test_function_decl.name)
            })?));
        Ok(Self {
            pass_condition,
            span,
            file_path,
        })
    }
}

/// The suffix that helps identify the file which contains the hash of the binary file created when
/// scripts are built_package.
pub const SWAY_BIN_HASH_SUFFIX: &str = "-bin-hash";

/// The suffix that helps identify the file which contains the root hash of the binary file created
/// when predicates are built_package.
pub const SWAY_BIN_ROOT_SUFFIX: &str = "-bin-root";

/// Selects the build profile from all available build profiles in the workspace using build_opts.
fn build_profile_from_opts(
    build_profiles: &HashMap<String, BuildProfile>,
    build_options: &BuildOpts,
) -> Result<BuildProfile> {
    let BuildOpts {
        pkg,
        print,
        time_phases,
        profile: profile_opt,
        build_profile,
        release,
        metrics_outfile,
        tests,
        error_on_warnings,
        ..
    } = build_options;

    let selected_profile_name = match release {
        true => BuildProfile::RELEASE,
        false => build_profile,
    };

    // Retrieve the specified build profile
    let mut profile = build_profiles
        .get(selected_profile_name)
        .cloned()
        .unwrap_or_else(|| {
            println_warning(&format!(
                "The provided profile option {selected_profile_name} is not present in the manifest file. \
            Using default profile."
            ));
            BuildProfile::default()
        });
    profile.name = selected_profile_name.into();
    profile.print_ast |= print.ast;
    if profile.print_dca_graph.is_none() {
        profile.print_dca_graph.clone_from(&print.dca_graph);
    }
    if profile.print_dca_graph_url_format.is_none() {
        profile
            .print_dca_graph_url_format
            .clone_from(&print.dca_graph_url_format);
    }
    profile.print_ir |= print.ir.clone();
    profile.print_asm |= print.asm;
    profile.print_bytecode |= print.bytecode;
    profile.print_bytecode_spans |= print.bytecode_spans;
    profile.terse |= pkg.terse;
    profile.time_phases |= time_phases;
    profile.profile |= profile_opt;
    if profile.metrics_outfile.is_none() {
        profile.metrics_outfile.clone_from(metrics_outfile);
    }
    profile.include_tests |= tests;
    profile.error_on_warnings |= error_on_warnings;
    // profile.experimental = *experimental;

    Ok(profile)
}

/// Returns a formatted string of the selected build profile and targets.
fn profile_target_string(profile_name: &str, build_target: &BuildTarget) -> String {
    let mut targets = vec![format!("{build_target}")];
    match profile_name {
        BuildProfile::DEBUG => targets.insert(0, "unoptimized".into()),
        BuildProfile::RELEASE => targets.insert(0, "optimized".into()),
        _ => {}
    };
    format!("{profile_name} [{}] target(s)", targets.join(" + "))
}
/// Returns the size of the bytecode in a human-readable format.
pub fn format_bytecode_size(bytes_len: usize) -> String {
    let size = Byte::from_u64(bytes_len as u64);
    let adjusted_byte = size.get_appropriate_unit(UnitType::Decimal);
    adjusted_byte.to_string()
}

/// Check if the given node is a contract dependency of any node in the graph.
fn is_contract_dependency(graph: &Graph, node: NodeIx) -> bool {
    graph
        .edges_directed(node, Direction::Incoming)
        .any(|e| matches!(e.weight().kind, DepKind::Contract { .. }))
}

/// Builds a project with given BuildOptions.
pub fn build_with_options(build_options: &BuildOpts) -> Result<Built> {
    let BuildOpts {
        hex_outfile,
        minify,
        binary_outfile,
        debug_outfile,
        pkg,
        build_target,
        member_filter,
        experimental,
        no_experimental,
        no_output,
        ..
    } = &build_options;

    let current_dir = std::env::current_dir()?;
    let path = &build_options
        .pkg
        .path
        .as_ref()
        .map_or_else(|| current_dir, PathBuf::from);

    println_action_green("Building", &path.display().to_string());

    let build_plan = BuildPlan::from_pkg_opts(&build_options.pkg)?;
    let graph = build_plan.graph();
    let manifest_map = build_plan.manifest_map();

    // Check if manifest used to create the build plan is one of the member manifests or a
    // workspace manifest.
    let curr_manifest = manifest_map
        .values()
        .find(|&pkg_manifest| pkg_manifest.dir() == path);
    let build_profiles: HashMap<String, BuildProfile> = build_plan.build_profiles().collect();
    // Get the selected build profile using build options
    let build_profile = build_profile_from_opts(&build_profiles, build_options)?;
    // If this is a workspace we want to have all members in the output.
    let outputs = match curr_manifest {
        Some(pkg_manifest) => std::iter::once(
            build_plan
                .find_member_index(&pkg_manifest.project.name)
                .ok_or_else(|| anyhow!("Cannot found project node in the graph"))?,
        )
        .collect(),
        None => build_plan.member_nodes().collect(),
    };

    let outputs = member_filter.filter_outputs(&build_plan, outputs);

    // Build it!
    let mut built_workspace = Vec::new();
    let build_start = std::time::Instant::now();
    let built_packages = build(
        &build_plan,
        *build_target,
        &build_profile,
        &outputs,
        experimental,
        no_experimental,
    )?;
    let output_dir = pkg.output_directory.as_ref().map(PathBuf::from);
    let total_size = built_packages
        .iter()
        .map(|(_, pkg)| pkg.bytecode.bytes.len())
        .sum::<usize>();

    println_action_green(
        "Finished",
        &format!(
            "{} [{}] in {:.2}s",
            profile_target_string(&build_profile.name, build_target),
            format_bytecode_size(total_size),
            build_start.elapsed().as_secs_f32()
        ),
    );
    for (node_ix, built_package) in built_packages {
        print_pkg_summary_header(&built_package);
        let pinned = &graph[node_ix];
        let pkg_manifest = manifest_map
            .get(&pinned.id())
            .ok_or_else(|| anyhow!("Couldn't find member manifest for {}", pinned.name))?;
        let output_dir = output_dir.clone().unwrap_or_else(|| {
            default_output_directory(pkg_manifest.dir()).join(&build_profile.name)
        });
        // Output artifacts for the built package
        if let Some(outfile) = &binary_outfile {
            built_package.write_bytecode(outfile.as_ref())?;
        }
        // Generate debug symbols if explicitly requested via -g flag or if in debug build
        if debug_outfile.is_some() || build_profile.name == BuildProfile::DEBUG {
            let debug_path = debug_outfile
                .as_ref()
                .map(|p| output_dir.join(p))
                .unwrap_or_else(|| output_dir.join("debug_symbols.obj"));
            built_package.write_debug_info(&debug_path)?;
        }

        if let Some(hex_path) = hex_outfile {
            let hexfile_path = output_dir.join(hex_path);
            built_package.write_hexcode(&hexfile_path)?;
        }

        if !no_output {
            built_package.write_output(minify, &pkg_manifest.project.name, &output_dir)?;
        }

        built_workspace.push(Arc::new(built_package));
    }

    match curr_manifest {
        Some(pkg_manifest) => {
            let built_pkg = built_workspace
                .into_iter()
                .find(|pkg| pkg.descriptor.manifest_file == *pkg_manifest)
                .expect("package didn't exist in workspace");
            Ok(Built::Package(built_pkg))
        }
        None => Ok(Built::Workspace(built_workspace)),
    }
}

fn print_pkg_summary_header(built_pkg: &BuiltPackage) {
    let prog_ty_str = forc_util::program_type_str(&built_pkg.tree_type);
    // The ansiterm formatters ignore the `std::fmt` right-align
    // formatter, so we manually calculate the padding to align the program
    // type and name around the 10th column ourselves.
    let padded_ty_str = format!("{prog_ty_str:>10}");
    let padding = &padded_ty_str[..padded_ty_str.len() - prog_ty_str.len()];
    let ty_ansi = ansiterm::Colour::Green.bold().paint(prog_ty_str);
    let name_ansi = ansiterm::Style::new()
        .bold()
        .paint(&built_pkg.descriptor.name);
    debug!("{padding}{ty_ansi} {name_ansi}");
}

/// Returns the ContractId of a built_package contract with specified `salt`.
pub fn contract_id(
    bytecode: &[u8],
    mut storage_slots: Vec<StorageSlot>,
    salt: &fuel_tx::Salt,
) -> ContractId {
    // Construct the contract ID
    let contract = Contract::from(bytecode);
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    contract.id(salt, &contract.root(), &state_root)
}

/// Checks if there are conflicting `Salt` declarations for the contract dependencies in the graph.
fn validate_contract_deps(graph: &Graph) -> Result<()> {
    // For each contract dependency node in the graph, check if there are conflicting salt
    // declarations.
    for node in graph.node_indices() {
        let pkg = &graph[node];
        let name = pkg.name.clone();
        let salt_declarations: HashSet<fuel_tx::Salt> = graph
            .edges_directed(node, Direction::Incoming)
            .filter_map(|e| match e.weight().kind {
                DepKind::Library => None,
                DepKind::Contract { salt } => Some(salt),
            })
            .collect();
        if salt_declarations.len() > 1 {
            bail!(
                "There are conflicting salt declarations for contract dependency named: {}\nDeclared salts: {:?}",
                name,
                salt_declarations,
            )
        }
    }
    Ok(())
}

/// Build an entire forc package and return the built_package output.
///
/// This compiles all packages (including dependencies) in the order specified by the `BuildPlan`.
///
/// Also returns the resulting `sway_core::SourceMap` which may be useful for debugging purposes.
pub fn build(
    plan: &BuildPlan,
    target: BuildTarget,
    profile: &BuildProfile,
    outputs: &HashSet<NodeIx>,
    experimental: &[sway_features::Feature],
    no_experimental: &[sway_features::Feature],
) -> anyhow::Result<Vec<(NodeIx, BuiltPackage)>> {
    let mut built_packages = Vec::new();

    let required: HashSet<NodeIx> = outputs
        .iter()
        .flat_map(|output_node| plan.node_deps(*output_node))
        .collect();

    let engines = Engines::default();
    let include_tests = profile.include_tests;

    // This is the Contract ID of the current contract being compiled.
    // We will need this for `forc test`.
    let mut contract_id_value: Option<ContractIdConst> = None;

    let mut lib_namespace_map = HashMap::default();
    let mut compiled_contract_deps = HashMap::new();

    for &node in plan
        .compilation_order
        .iter()
        .filter(|node| required.contains(node))
    {
        let mut source_map = SourceMap::new();
        let pkg = &plan.graph()[node];
        let manifest = &plan.manifest_map()[&pkg.id()];
        let program_ty = manifest.program_type().ok();
        let dbg_generation = match (profile.is_release(), manifest.project.force_dbg_in_release) {
            (true, Some(true)) | (false, _) => DbgGeneration::Full,
            (true, _) => DbgGeneration::None,
        };

        print_compiling(
            program_ty.as_ref(),
            &pkg.name,
            &pkg.source.display_compiling(manifest.dir()),
        );

        let experimental = ExperimentalFeatures::new(
            &manifest.project.experimental,
            experimental,
            no_experimental,
        )
        .map_err(|err| anyhow!("{err}"))?;

        let descriptor = PackageDescriptor {
            name: pkg.name.clone(),
            target,
            pinned: pkg.clone(),
            manifest_file: manifest.clone(),
        };

        let fail = |warnings, errors| {
            print_on_failure(
                engines.se(),
                profile.terse,
                warnings,
                errors,
                profile.reverse_results,
            );
            bail!("Failed to compile {}", pkg.name);
        };

        let is_contract_dependency = is_contract_dependency(plan.graph(), node);
        // If we are building a contract and tests are enabled or we are building a contract
        // dependency, we need the tests excluded bytecode.
        let bytecode_without_tests = if (include_tests
            && matches!(manifest.program_type(), Ok(TreeType::Contract)))
            || is_contract_dependency
        {
            // We will build a contract with tests enabled, we will also need the same contract with tests
            // disabled for:
            //
            //   1. Interpreter deployment in `forc-test`.
            //   2. Contract ID injection in `forc-pkg` if this is a contract dependency to any
            //      other pkg, so that injected contract id is not effected by the tests.
            let profile = BuildProfile {
                include_tests: false,
                ..profile.clone()
            };

            let program_id = engines
                .se()
                .get_or_create_program_id_from_manifest_path(&manifest.entry_path());

            // `ContractIdConst` is a None here since we do not yet have a
            // contract ID value at this point.
            let dep_namespace = match dependency_namespace(
                &lib_namespace_map,
                &compiled_contract_deps,
                plan.graph(),
                node,
                &engines,
                None,
                program_id,
                experimental,
                dbg_generation,
            ) {
                Ok(o) => o,
                Err(errs) => return fail(&[], &errs),
            };

            let compiled_without_tests = compile(
                &descriptor,
                &profile,
                &engines,
                dep_namespace,
                &mut source_map,
                experimental,
                dbg_generation,
            )?;

            if let Some(outfile) = profile.metrics_outfile {
                let path = Path::new(&outfile);
                let metrics_json = serde_json::to_string(&compiled_without_tests.metrics)
                    .expect("JSON serialization failed");
                fs::write(path, metrics_json)?;
            }

            // If this contract is built because:
            // 1) it is a contract dependency, or
            // 2) tests are enabled,
            // we need to insert its CONTRACT_ID into a map for later use.
            if is_contract_dependency {
                let compiled_contract_dep = CompiledContractDependency {
                    bytecode: compiled_without_tests.bytecode.bytes.clone(),
                    storage_slots: compiled_without_tests.storage_slots.clone(),
                };
                compiled_contract_deps.insert(node, compiled_contract_dep);
            } else {
                // `forc-test` interpreter deployments are done with zeroed salt.
                let contract_id = contract_id(
                    &compiled_without_tests.bytecode.bytes,
                    compiled_without_tests.storage_slots.clone(),
                    &fuel_tx::Salt::zeroed(),
                );
                // We finally set the contract ID value here to use for compilation later if tests are enabled.
                contract_id_value = Some(format!("0x{contract_id}"));
            }
            Some(compiled_without_tests.bytecode)
        } else {
            None
        };

        // Build all non member nodes with tests disabled by overriding the current profile.
        let profile = if !plan.member_nodes().any(|member| member == node) {
            BuildProfile {
                include_tests: false,
                ..profile.clone()
            }
        } else {
            profile.clone()
        };

        let program_id = engines
            .se()
            .get_or_create_program_id_from_manifest_path(&manifest.entry_path());

        // Note that the contract ID value here is only Some if tests are enabled.
        let dep_namespace = match dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            plan.graph(),
            node,
            &engines,
            contract_id_value.clone(),
            program_id,
            experimental,
            dbg_generation,
        ) {
            Ok(o) => o,
            Err(errs) => {
                print_on_failure(
                    engines.se(),
                    profile.terse,
                    &[],
                    &errs,
                    profile.reverse_results,
                );
                bail!("Failed to compile {}", pkg.name);
            }
        };

        let compiled = compile(
            &descriptor,
            &profile,
            &engines,
            dep_namespace,
            &mut source_map,
            experimental,
            dbg_generation,
        )?;

        if let Some(outfile) = profile.metrics_outfile {
            let path = Path::new(&outfile);
            let metrics_json =
                serde_json::to_string(&compiled.metrics).expect("JSON serialization failed");
            fs::write(path, metrics_json)?;
        }

        if let TreeType::Library = compiled.tree_type {
            lib_namespace_map.insert(node, compiled.namespace);
        }
        source_map.insert_dependency(descriptor.manifest_file.dir());

        let built_pkg = BuiltPackage {
            descriptor,
            program_abi: compiled.program_abi,
            storage_slots: compiled.storage_slots,
            source_map: compiled.source_map,
            tree_type: compiled.tree_type,
            bytecode: compiled.bytecode,
            warnings: compiled.warnings,
            bytecode_without_tests,
        };

        if outputs.contains(&node) {
            built_packages.push((node, built_pkg));
        }
    }

    Ok(built_packages)
}

/// Compile the entire forc package and return the lexed, parsed and typed programs
/// of the dependencies and project.
/// The final item in the returned vector is the project.
#[allow(clippy::too_many_arguments)]
pub fn check(
    plan: &BuildPlan,
    build_target: BuildTarget,
    terse_mode: bool,
    lsp_mode: Option<LspConfig>,
    include_tests: bool,
    engines: &Engines,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    experimental: &[sway_features::Feature],
    no_experimental: &[sway_features::Feature],
    dbg_generation: sway_core::DbgGeneration,
) -> anyhow::Result<Vec<(Option<Programs>, Handler)>> {
    let mut lib_namespace_map = HashMap::default();
    let mut source_map = SourceMap::new();
    // During `check`, we don't compile so this stays empty.
    let compiled_contract_deps = HashMap::new();

    let mut results = vec![];
    for (idx, &node) in plan.compilation_order.iter().enumerate() {
        let pkg = &plan.graph[node];
        let manifest = &plan.manifest_map()[&pkg.id()];

        let experimental = ExperimentalFeatures::new(
            &manifest.project.experimental,
            experimental,
            no_experimental,
        )
        .map_err(|err| anyhow!("{err}"))?;

        // Only inject a dummy CONTRACT_ID in LSP mode, not when check() is called from tests or other non-LSP contexts,
        // to avoid polluting namespaces unnecessarily.
        let contract_id_value = if lsp_mode.is_some() && (idx == plan.compilation_order.len() - 1) {
            // This is necessary because `CONTRACT_ID` is a special constant that's injected into the
            // compiler's namespace. Although we only know the contract id during building, we are
            // inserting a dummy value here to avoid false error signals being reported in LSP.
            // We only do this for the last node in the compilation order because previous nodes
            // are dependencies.
            //
            // See this github issue for more context: https://github.com/FuelLabs/sway-vscode-plugin/issues/154
            const DUMMY_CONTRACT_ID: &str =
                "0x0000000000000000000000000000000000000000000000000000000000000000";
            Some(DUMMY_CONTRACT_ID.to_string())
        } else {
            None
        };

        let program_id = engines
            .se()
            .get_or_create_program_id_from_manifest_path(&manifest.entry_path());

        let dep_namespace = dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            &plan.graph,
            node,
            engines,
            contract_id_value,
            program_id,
            experimental,
            dbg_generation,
        )
        .expect("failed to create dependency namespace");

        let profile = BuildProfile {
            terse: terse_mode,
            ..BuildProfile::debug()
        };

        let build_config = sway_build_config(
            manifest.dir(),
            &manifest.entry_path(),
            build_target,
            &profile,
            dbg_generation,
        )?
        .with_include_tests(include_tests)
        .with_lsp_mode(lsp_mode.clone());

        let input = manifest.entry_string()?;
        let handler = Handler::default();
        let programs_res = sway_core::compile_to_ast(
            &handler,
            engines,
            input,
            dep_namespace,
            Some(&build_config),
            &pkg.name,
            retrigger_compilation.clone(),
            experimental,
        );

        if retrigger_compilation
            .as_ref()
            .is_some_and(|b| b.load(std::sync::atomic::Ordering::SeqCst))
        {
            bail!("compilation was retriggered")
        }

        let programs = match programs_res.as_ref() {
            Ok(programs) => programs,
            _ => {
                results.push((programs_res.ok(), handler));
                return Ok(results);
            }
        };

        if let Ok(typed_program) = programs.typed.as_ref() {
            if let TreeType::Library = typed_program.kind.tree_type() {
                let mut lib_namespace = typed_program.namespace.current_package_ref().clone();
                lib_namespace.root_module_mut().set_span(
                    Span::new(
                        manifest.entry_string()?,
                        0,
                        0,
                        Some(engines.se().get_source_id(&manifest.entry_path())),
                    )
                    .unwrap(),
                );
                lib_namespace_map.insert(node, lib_namespace);
            }
            source_map.insert_dependency(manifest.dir());
        } else {
            results.push((programs_res.ok(), handler));
            return Ok(results);
        }
        results.push((programs_res.ok(), handler));
    }

    if results.is_empty() {
        bail!("unable to check sway program: build plan contains no packages")
    }

    Ok(results)
}

/// Format an error message for an absent `Forc.toml`.
pub fn manifest_file_missing<P: AsRef<Path>>(dir: P) -> anyhow::Error {
    let message = format!(
        "could not find `{}` in `{}` or any parent directory",
        constants::MANIFEST_FILE_NAME,
        dir.as_ref().display()
    );
    Error::msg(message)
}

/// Format an error message for failed parsing of a manifest.
pub fn parsing_failed(project_name: &str, errors: &[CompileError]) -> anyhow::Error {
    let error = errors
        .iter()
        .map(|e| format!("{e}"))
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("Parsing {project_name} failed: \n{error}");
    Error::msg(message)
}

/// Format an error message if an incorrect program type is present.
pub fn wrong_program_type(
    project_name: &str,
    expected_types: &[TreeType],
    parse_type: TreeType,
) -> anyhow::Error {
    let message = format!("{project_name} is not a '{expected_types:?}' it is a '{parse_type:?}'");
    Error::msg(message)
}

/// Format an error message if a given URL fails to produce a working node.
pub fn fuel_core_not_running(node_url: &str) -> anyhow::Error {
    let message = format!("could not get a response from node at the URL {node_url}. Start a node with `fuel-core`. See https://github.com/FuelLabs/fuel-core#running for more information");
    Error::msg(message)
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Regex;
    use tempfile::NamedTempFile;

    fn setup_build_plan() -> BuildPlan {
        let current_dir = env!("CARGO_MANIFEST_DIR");
        let manifest_dir = PathBuf::from(current_dir)
            .parent()
            .unwrap()
            .join("test/src/e2e_vm_tests/test_programs/should_pass/forc/workspace_building/");
        let manifest_file = ManifestFile::from_dir(manifest_dir).unwrap();
        let member_manifests = manifest_file.member_manifests().unwrap();
        let lock_path = manifest_file.lock_path().unwrap();
        BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            &IPFSNode::default(),
        )
        .unwrap()
    }

    #[test]
    fn test_root_pkg_order() {
        let build_plan = setup_build_plan();
        let graph = build_plan.graph();
        let order: Vec<String> = build_plan
            .member_nodes()
            .map(|order| graph[order].name.clone())
            .collect();
        assert_eq!(order, vec!["test_lib", "test_contract", "test_script"])
    }

    #[test]
    fn test_visualize_with_url_prefix() {
        let build_plan = setup_build_plan();
        let result = build_plan.visualize(Some("some-prefix::".to_string()));
        let re = Regex::new(r#"digraph \{
    0 \[ label = "std" shape = box URL = "some-prefix::[[:ascii:]]+/sway-lib-std/Forc.toml"\]
    1 \[ label = "test_contract" shape = box URL = "some-prefix::/[[:ascii:]]+/test_contract/Forc.toml"\]
    2 \[ label = "test_lib" shape = box URL = "some-prefix::/[[:ascii:]]+/test_lib/Forc.toml"\]
    3 \[ label = "test_script" shape = box URL = "some-prefix::/[[:ascii:]]+/test_script/Forc.toml"\]
    3 -> 2 \[ \]
    3 -> 0 \[ \]
    3 -> 1 \[ \]
    1 -> 2 \[ \]
    1 -> 0 \[ \]
\}
"#).unwrap();
        dbg!(&result);
        assert!(!re.find(result.as_str()).unwrap().is_empty());
    }

    #[test]
    fn test_visualize_without_prefix() {
        let build_plan = setup_build_plan();
        let result = build_plan.visualize(None);
        let expected = r#"digraph {
    0 [ label = "std" shape = box ]
    1 [ label = "test_contract" shape = box ]
    2 [ label = "test_lib" shape = box ]
    3 [ label = "test_script" shape = box ]
    3 -> 2 [ ]
    3 -> 0 [ ]
    3 -> 1 [ ]
    1 -> 2 [ ]
    1 -> 0 [ ]
}
"#;
        assert_eq!(expected, result);
    }

    #[test]
    fn test_write_hexcode() -> Result<()> {
        // Create a temporary file for testing
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        let current_dir = env!("CARGO_MANIFEST_DIR");
        let manifest_dir = PathBuf::from(current_dir).parent().unwrap().join(
            "test/src/e2e_vm_tests/test_programs/should_pass/forc/workspace_building/test_contract",
        );

        // Create a test BuiltPackage with some bytecode
        let test_bytecode = vec![0x01, 0x02, 0x03, 0x04];
        let built_package = BuiltPackage {
            descriptor: PackageDescriptor {
                name: "test_package".to_string(),
                target: BuildTarget::Fuel,
                pinned: Pinned {
                    name: "built_test".to_owned(),
                    source: source::Pinned::MEMBER,
                },
                manifest_file: PackageManifestFile::from_dir(manifest_dir)?,
            },
            program_abi: ProgramABI::Fuel(fuel_abi_types::abi::program::ProgramABI {
                program_type: "".to_owned(),
                spec_version: "".into(),
                encoding_version: "".into(),
                concrete_types: vec![],
                metadata_types: vec![],
                functions: vec![],
                configurables: None,
                logged_types: None,
                messages_types: None,
                error_codes: None,
            }),
            storage_slots: vec![],
            warnings: vec![],
            source_map: SourceMap::new(),
            tree_type: TreeType::Script,
            bytecode: BuiltPackageBytecode {
                bytes: test_bytecode,
                entries: vec![],
            },
            bytecode_without_tests: None,
        };

        // Write the hexcode
        built_package.write_hexcode(path)?;

        // Read the file and verify its contents
        let contents = fs::read_to_string(path)?;
        let expected = r#"{"hex":"0x01020304"}"#;
        assert_eq!(contents, expected);

        Ok(())
    }
}
