use crate::{
    lock::Lock,
    manifest::{BuildProfile, Dependency, ManifestFile, MemberManifestFiles, PackageManifestFile},
    source::{self, Source},
    CORE, PRELUDE, STD,
};
use anyhow::{anyhow, bail, Context, Error, Result};
use forc_util::{
    default_output_directory, find_file_name, kebab_to_snake_case, print_compiling,
    print_on_failure, print_warnings, user_forc_directory,
};
use fuel_abi_types::program_abi;
use petgraph::{
    self,
    visit::{Bfs, Dfs, EdgeRef, Walker},
    Directed, Direction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeSet, HashMap, HashSet},
    fmt,
    fs::{self, File},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use sway_core::{
    abi_generation::{
        evm_json_abi,
        fuel_json_abi::{self, JsonAbiContext},
    },
    asm_generation::ProgramABI,
    decl_engine::{DeclEngine, DeclRefFunction},
    fuel_prelude::{
        fuel_crypto,
        fuel_tx::{self, Contract, ContractId, StorageSlot},
    },
    language::{
        lexed::LexedProgram,
        parsed::{ParseProgram, TreeType},
        ty,
    },
    semantic_analysis::namespace,
    source_map::SourceMap,
    transform::AttributeKind,
    BuildTarget, CompileResult, Engines, FinalizedEntry, TypeEngine,
};
use sway_error::{error::CompileError, warning::CompileWarning};
use sway_types::{Ident, Span, Spanned};
use sway_utils::constants;
use tracing::{info, warn};

type GraphIx = u32;
type Node = Pinned;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Edge {
    /// The name specified on the left hand side of the `=` in a depenedency declaration under
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
    source_map: SourceMap,
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
    ShouldRevert,
    ShouldNotRevert,
}

/// Data specific to the test entry point.
#[derive(Debug, Clone)]
pub struct PkgTestEntry {
    pub pass_condition: TestPassCondition,
    pub span: Span,
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
    pub namespace: namespace::Root,
    pub warnings: Vec<CompileWarning>,
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
    /// Outputs json abi with callpath instead of struct and enum names.
    pub json_abi_with_callpaths: bool,
}

#[derive(Default, Clone)]
pub struct PrintOpts {
    /// Print the generated Sway AST (Abstract Syntax Tree).
    pub ast: bool,
    /// Print the computed Sway DCA (Dead Code Analysis) graph.
    pub dca_graph: bool,
    /// Print the finalized ASM.
    ///
    /// This is the state of the ASM with registers allocated and optimisations applied.
    pub finalized_asm: bool,
    /// Print the generated ASM.
    ///
    /// This is the state of the ASM prior to performing register allocation and other ASM
    /// optimisations.
    pub intermediate_asm: bool,
    /// Print the generated Sway IR (Intermediate Representation).
    pub ir: bool,
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
#[derive(Default)]
pub struct BuildOpts {
    pub pkg: PkgOpts,
    pub print: PrintOpts,
    pub minify: MinifyOpts,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    pub debug_outfile: Option<String>,
    /// Build target to use.
    pub build_target: BuildTarget,
    /// Name of the build profile to use.
    /// If it is not specified, forc will use debug build profile.
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    ///  If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    pub release: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
    /// Warnings must be treated as compiler errors.
    pub error_on_warnings: bool,
    /// Include all test functions within the build.
    pub tests: bool,
    /// The set of options to filter by member project kind.
    pub member_filter: MemberFilter,
}

/// The set of options to filter type of projects to build in a workspace.
pub struct MemberFilter {
    pub build_contracts: bool,
    pub build_scripts: bool,
    pub build_predicates: bool,
    pub build_libraries: bool,
}

/// Contains the lexed, parsed, and typed compilation stages of a program.
pub struct Programs {
    pub lexed: LexedProgram,
    pub parsed: ParseProgram,
    pub typed: Option<ty::TyProgram>,
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
    /// Returns a new `BuildFilter` that only builds scripts.
    pub fn only_scripts() -> Self {
        Self {
            build_contracts: false,
            build_scripts: true,
            build_predicates: false,
            build_libraries: false,
        }
    }

    /// Returns a new `BuildFilter` that only builds contracts.
    pub fn only_contracts() -> Self {
        Self {
            build_contracts: true,
            build_scripts: false,
            build_predicates: false,
            build_libraries: false,
        }
    }

    /// Returns a new `BuildFilter`, that only builds predicates.
    pub fn only_predicates() -> Self {
        Self {
            build_contracts: false,
            build_scripts: false,
            build_predicates: true,
            build_libraries: false,
        }
    }

    /// Filter given target of output nodes according to the this `BuildFilter`.
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
                // Since parser cannot recover for program type detection, for the scenerios that
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
                        TreeType::Library { .. } => self.build_libraries,
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

    /// Writes debug_info (source_map) of the BuiltPackage to the given `path`.
    pub fn write_debug_info(&self, path: &Path) -> Result<()> {
        let source_map_json =
            serde_json::to_vec(&self.source_map).expect("JSON serialization failed");
        fs::write(path, source_map_json)?;
        Ok(())
    }

    /// Writes BuiltPackage to `output_dir`.
    pub fn write_output(
        &self,
        minify: MinifyOpts,
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
        let program_abi_path = output_dir.join(program_abi_stem).with_extension("json");
        match &self.program_abi {
            ProgramABI::Fuel(program_abi) => {
                if !program_abi.functions.is_empty() {
                    let file = File::create(program_abi_path)?;
                    let res = if minify.json_abi {
                        serde_json::to_writer(&file, &program_abi)
                    } else {
                        serde_json::to_writer_pretty(&file, &program_abi)
                    };
                    res?
                }
            }
            ProgramABI::Evm(program_abi) => {
                if !program_abi.is_empty() {
                    let file = File::create(program_abi_path)?;
                    let res = if minify.json_abi {
                        serde_json::to_writer(&file, &program_abi)
                    } else {
                        serde_json::to_writer_pretty(&file, &program_abi)
                    };
                    res?
                }
            }
            // TODO?
            ProgramABI::MidenVM(_) => (),
        }

        info!("      Bytecode size: {} bytes", self.bytecode.bytes.len());
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
                let root = format!("0x{}", Contract::root_from_code(&self.bytecode.bytes));
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
                info!("      Bytecode hash: {}", bytecode_hash);
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
    ) -> Box<dyn Iterator<Item = (&Pinned, Arc<BuiltPackage>)> + 'a> {
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
    pub fn from_build_opts(build_options: &BuildOpts) -> Result<Self> {
        let path = &build_options.pkg.path;

        let manifest_dir = if let Some(ref path) = path {
            PathBuf::from(path)
        } else {
            std::env::current_dir()?
        };

        let manifest_file = ManifestFile::from_dir(&manifest_dir)?;
        let member_manifests = manifest_file.member_manifests()?;
        // Check if we have members to build so that we are not trying to build an empty workspace.
        if member_manifests.is_empty() {
            bail!("No member found to build")
        }
        let lock_path = manifest_file.lock_path()?;
        Self::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            build_options.pkg.locked,
            build_options.pkg.offline,
        )
    }

    /// Create a new build plan for the project by fetching and pinning all dependenies.
    ///
    /// To account for an existing lock file, use `from_lock_and_manifest` instead.
    pub fn from_manifests(manifests: &MemberManifestFiles, offline: bool) -> Result<Self> {
        // Check toolchain version
        validate_version(manifests)?;
        let mut graph = Graph::default();
        let mut manifest_map = ManifestMap::default();
        fetch_graph(manifests, offline, &mut graph, &mut manifest_map)?;
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
        let _added = fetch_graph(manifests, offline, &mut graph, &mut manifest_map)?;

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
            info!("  Creating a new `Forc.lock` file. (Cause: {})", cause);
            let member_names = manifests
                .iter()
                .map(|(_, manifest)| manifest.project.name.to_string())
                .collect();
            crate::lock::print_diff(&member_names, &lock_diff);
            let string = toml::ser::to_string_pretty(&new_lock)
                .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
            fs::write(lock_path, string)
                .map_err(|e| anyhow!("failed to write lock file: {}", e))?;
            info!("   Created new lock file at {}", lock_path.display());
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
    /// In the case that this `BuildPlan` was constructed for a single package,
    /// only that package's node will be yielded.
    pub fn member_nodes(&self) -> impl Iterator<Item = NodeIx> + '_ {
        self.compilation_order()
            .iter()
            .cloned()
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

    /// Return the salt for the given pinned pkg, for none contract members returns `None`.
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
}

impl Programs {
    pub fn new(
        lexed: LexedProgram,
        parsed: ParseProgram,
        typed: Option<ty::TyProgram>,
    ) -> Programs {
        Programs {
            lexed,
            parsed,
            typed,
        }
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
    match &pkg_manifest.project.forc_version {
        Some(min_forc_version) => {
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
        }
        None => {}
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
/// Part of dependency validation, any checks related to the depenency's manifest content.
fn validate_dep_manifest(
    dep: &Pinned,
    dep_manifest: &PackageManifestFile,
    dep_edge: &Edge,
) -> Result<()> {
    let dep_program_type = dep_manifest.program_type()?;
    // Check if the dependency is either a library or a contract declared as a contract dependency
    match (&dep_program_type, &dep_edge.kind) {
        (TreeType::Contract, DepKind::Contract { salt: _ })
        | (TreeType::Library { .. }, DepKind::Library) => {}
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
    let node_removal_order = match petgraph::algo::toposort(&*graph, None) {
        Ok(nodes) => nodes,
        Err(_) => {
            // If toposort fails the given graph is cyclic, so invalidate everything.
            graph.clear();
            return;
        }
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
/// performing a toposort of the graph with reversed weights. The resulting order ensures all
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
/// and visits childs to collect their manifest files.
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
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(node) => node,
        Err(_) => return Ok(manifest_map),
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
            source::Pinned::Git(_) | source::Pinned::Registry(_) | source::Pinned::Member(_) => {
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
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
) -> Result<HashSet<NodeIx>> {
    let mut added_nodes = HashSet::default();
    for member_pkg_manifest in member_manifests.values() {
        added_nodes.extend(&fetch_pkg_graph(
            member_pkg_manifest,
            offline,
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
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
    member_manifests: &MemberManifestFiles,
) -> Result<HashSet<NodeIx>> {
    // Retrieve the project node, or create one if it does not exist.
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(proj_node) => proj_node,
        Err(_) => {
            let name = proj_manifest.project.name.clone();
            let source = source::Pinned::MEMBER;
            let pkg = Pinned { name, source };
            let pkg_id = pkg.id();
            manifest_map.insert(pkg_id, proj_manifest.clone());
            graph.add_node(pkg)
        }
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
                DepKind::Contract { salt: d.salt },
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
                .context("Failed to source dependency")?;

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
            source::Pinned::Member(_) | source::Pinned::Git(_) | source::Pinned::Registry(_) => {
                dep_pkg_id
            }
            source::Pinned::Path(_) => path_root,
        };

        // Fetch the children.
        added.extend(fetch_deps(
            fetch_id,
            offline,
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

/// Given a path to a directory we wish to lock, produce a path for an associated lock file.
///
/// Note that the lock file itself is simply a placeholder for co-ordinating access. As a result,
/// we want to create the lock file if it doesn't exist, but we can never reliably remove it
/// without risking invalidation of an existing lock. As a result, we use a dedicated, hidden
/// directory with a lock file named after the checkout path.
///
/// Note: This has nothing to do with `Forc.lock` files, rather this is about fd locks for
/// coordinating access to particular paths (e.g. git checkout directories).
fn fd_lock_path(path: &Path) -> PathBuf {
    const LOCKS_DIR_NAME: &str = ".locks";
    const LOCK_EXT: &str = "forc-lock";

    // Hash the path to produce a file-system friendly lock file name.
    // Append the file stem for improved readability.
    let mut hasher = hash_map::DefaultHasher::default();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    let file_name = match path.file_stem().and_then(|s| s.to_str()) {
        None => format!("{hash:X}"),
        Some(stem) => format!("{hash:X}-{stem}"),
    };

    user_forc_directory()
        .join(LOCKS_DIR_NAME)
        .join(file_name)
        .with_extension(LOCK_EXT)
}

/// Create an advisory lock over the given path.
///
/// See [fd_lock_path] for details.
pub(crate) fn path_lock(path: &Path) -> Result<fd_lock::RwLock<File>> {
    let lock_path = fd_lock_path(path);
    let lock_dir = lock_path
        .parent()
        .expect("lock path has no parent directory");
    std::fs::create_dir_all(lock_dir).context("failed to create forc advisory lock directory")?;
    let lock_file = File::create(&lock_path).context("failed to create advisory lock file")?;
    Ok(fd_lock::RwLock::new(lock_file))
}

/// Given a `forc_pkg::BuildProfile`, produce the necessary `sway_core::BuildConfig` required for
/// compilation.
pub fn sway_build_config(
    manifest_dir: &Path,
    entry_path: &Path,
    build_target: BuildTarget,
    build_profile: &BuildProfile,
) -> Result<sway_core::BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let file_name = find_file_name(manifest_dir, entry_path)?;
    let build_config = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.to_path_buf(),
        build_target,
    )
    .print_dca_graph(build_profile.print_dca_graph)
    .print_finalized_asm(build_profile.print_finalized_asm)
    .print_intermediate_asm(build_profile.print_intermediate_asm)
    .print_ir(build_profile.print_ir)
    .include_tests(build_profile.include_tests);
    Ok(build_config)
}

/// The name of the constant holding the contract's id.
pub const CONTRACT_ID_CONSTANT_NAME: &str = "CONTRACT_ID";

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
///
/// This function ensures that if `core` exists in the graph (the vastly common case) it is also
/// present within the namespace. This is a necessity for operators to work for example.
///
/// This function also ensures that if `std` exists in the graph,
/// then the std prelude will also be added.
///
/// `contract_id_value` should only be Some when producing the `dependency_namespace` for a contract with tests enabled.
/// This allows us to provide a contract's `CONTRACT_ID` constant to its own unit tests.
pub fn dependency_namespace(
    lib_namespace_map: &HashMap<NodeIx, namespace::Module>,
    compiled_contract_deps: &CompiledContractDeps,
    graph: &Graph,
    node: NodeIx,
    engines: Engines<'_>,
    contract_id_value: Option<ContractIdConst>,
) -> Result<namespace::Module, vec1::Vec1<CompileError>> {
    // TODO: Clean this up when config-time constants v1 are removed.
    let node_idx = &graph[node];
    let name = Some(Ident::new_no_span(node_idx.name.clone()));
    let mut namespace = if let Some(contract_id_value) = contract_id_value {
        namespace::Module::default_with_contract_id(engines, name.clone(), contract_id_value)?
    } else {
        namespace::Module::default()
    };

    namespace.is_external = true;
    namespace.name = name;

    // Add direct dependencies.
    let mut core_added = false;
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_node = edge.target();
        let dep_name = kebab_to_snake_case(&edge.weight().name);
        let dep_edge = edge.weight();
        let dep_namespace = match dep_edge.kind {
            DepKind::Library => lib_namespace_map
                .get(&dep_node)
                .cloned()
                .expect("no namespace module"),
            DepKind::Contract { salt } => {
                let dep_contract_id = compiled_contract_deps
                    .get(&dep_node)
                    .map(|dep| contract_id(dep.bytecode.clone(), dep.storage_slots.clone(), &salt))
                    // On `check` we don't compile contracts, so we use a placeholder.
                    .unwrap_or_default();
                // Construct namespace with contract id
                let contract_id_value = format!("0x{dep_contract_id}");
                let node_idx = &graph[dep_node];
                let name = Some(Ident::new_no_span(node_idx.name.clone()));
                let mut ns = namespace::Module::default_with_contract_id(
                    engines,
                    name.clone(),
                    contract_id_value,
                )?;
                ns.is_external = true;
                ns.name = name;
                ns
            }
        };
        namespace.insert_submodule(dep_name, dep_namespace);
        let dep = &graph[dep_node];
        if dep.name == CORE {
            core_added = true;
        }
    }

    // Add `core` if not already added.
    if !core_added {
        if let Some(core_node) = find_core_dep(graph, node) {
            let core_namespace = &lib_namespace_map[&core_node];
            namespace.insert_submodule(CORE.to_string(), core_namespace.clone());
        }
    }

    namespace.star_import_with_reexports(
        &[CORE, PRELUDE].map(|s| Ident::new_no_span(s.into())),
        &[],
        engines,
    );

    if has_std_dep(graph, node) {
        namespace.star_import_with_reexports(
            &[STD, PRELUDE].map(|s| Ident::new_no_span(s.into())),
            &[],
            engines,
        );
    }

    Ok(namespace)
}

/// Find the `std` dependency, if it is a direct one, of the given node.
fn has_std_dep(graph: &Graph, node: NodeIx) -> bool {
    // If we are `std`, do nothing.
    let pkg = &graph[node];
    if pkg.name == STD {
        return false;
    }

    // If we have `std` as a direct dep, use it.
    graph.edges_directed(node, Direction::Outgoing).any(|edge| {
        let dep_node = edge.target();
        let dep = &graph[dep_node];
        matches!(&dep.name[..], STD)
    })
}

/// Find the `core` dependency (whether direct or transitive) for the given node if it exists.
fn find_core_dep(graph: &Graph, node: NodeIx) -> Option<NodeIx> {
    // If we are `core`, do nothing.
    let pkg = &graph[node];
    if pkg.name == CORE {
        return None;
    }

    // If we have `core` as a direct dep, use it.
    let mut maybe_std = None;
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_node = edge.target();
        let dep = &graph[dep_node];
        match &dep.name[..] {
            CORE => return Some(dep_node),
            STD => maybe_std = Some(dep_node),
            _ => {}
        }
    }

    // If we have `std`, select `core` via `std`.
    if let Some(std) = maybe_std {
        return find_core_dep(graph, std);
    }

    // Otherwise, search from this node.
    for dep_node in Dfs::new(graph, node).iter(graph) {
        let dep = &graph[dep_node];
        if dep.name == CORE {
            return Some(dep_node);
        }
    }

    None
}

/// Compiles the package to an AST.
pub fn compile_ast(
    pkg: &PackageDescriptor,
    build_profile: &BuildProfile,
    engines: Engines<'_>,
    namespace: namespace::Module,
    package_name: &str,
) -> Result<CompileResult<ty::TyProgram>> {
    let source = pkg.manifest_file.entry_string()?;
    let sway_build_config = sway_build_config(
        pkg.manifest_file.dir(),
        &pkg.manifest_file.entry_path(),
        pkg.target,
        build_profile,
    )?;
    let ast_res = sway_core::compile_to_ast(
        engines,
        source,
        namespace,
        Some(&sway_build_config),
        package_name,
    );
    Ok(ast_res)
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
    engines: Engines<'_>,
    namespace: namespace::Module,
    source_map: &mut SourceMap,
) -> Result<CompiledPackage> {
    // Time the given expression and print the result if `build_config.time_phases` is true.
    macro_rules! time_expr {
        ($description:expr, $expression:expr) => {{
            if profile.time_phases {
                let expr_start = std::time::Instant::now();
                let output = { $expression };
                println!(
                    "  Time elapsed to {}: {:?}",
                    $description,
                    expr_start.elapsed()
                );
                output
            } else {
                $expression
            }
        }};
    }

    let entry_path = pkg.manifest_file.entry_path();
    let sway_build_config = time_expr!(
        "produce `sway_core::BuildConfig`",
        sway_build_config(pkg.manifest_file.dir(), &entry_path, pkg.target, profile)?
    );
    let terse_mode = profile.terse;
    let fail = |warnings, errors| {
        print_on_failure(terse_mode, warnings, errors);
        bail!("Failed to compile {}", pkg.name);
    };

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = time_expr!(
        "compile to ast",
        compile_ast(pkg, profile, engines, namespace, &pkg.name)?
    );
    let typed_program = match ast_res.value.as_ref() {
        None => return fail(&ast_res.warnings, &ast_res.errors),
        Some(typed_program) => typed_program,
    };

    if profile.print_ast {
        tracing::info!("{:#?}", typed_program);
    }

    let storage_slots = typed_program.storage_slots.clone();
    let tree_type = typed_program.kind.tree_type();

    let namespace = typed_program.root.namespace.clone().into();

    if !ast_res.errors.is_empty() {
        return fail(&ast_res.warnings, &ast_res.errors);
    }

    let asm_res = time_expr!(
        "compile ast to asm",
        sway_core::ast_to_asm(engines, &ast_res, &sway_build_config)
    );

    let mut program_abi = match pkg.target {
        BuildTarget::Fuel => {
            let mut types = vec![];
            ProgramABI::Fuel(time_expr!(
                "generate JSON ABI program",
                fuel_json_abi::generate_json_abi_program(
                    &mut JsonAbiContext {
                        program: typed_program,
                        json_abi_with_callpaths: profile.json_abi_with_callpaths,
                    },
                    engines.te(),
                    engines.de(),
                    &mut types
                )
            ))
        }
        BuildTarget::EVM => {
            // Merge the ABI output of ASM gen with ABI gen to handle internal constructors
            // generated by the ASM backend.
            let mut ops = match &asm_res.value {
                Some(ref asm) => match &asm.0.abi {
                    Some(ProgramABI::Evm(ops)) => ops.clone(),
                    _ => vec![],
                },
                _ => vec![],
            };

            let abi = time_expr!(
                "generate JSON ABI program",
                evm_json_abi::generate_json_abi_program(typed_program, &engines)
            );

            ops.extend(abi.into_iter());

            ProgramABI::Evm(ops)
        }

        BuildTarget::MidenVM => ProgramABI::MidenVM(()),
    };

    let entries = asm_res
        .value
        .as_ref()
        .map(|asm| asm.0.entries.clone())
        .unwrap_or_default();
    let decl_engine = engines.de();
    let entries = entries
        .iter()
        .map(|finalized_entry| PkgEntry::from_finalized_entry(finalized_entry, decl_engine))
        .collect::<anyhow::Result<_>>()?;
    let bc_res = time_expr!(
        "compile asm to bytecode",
        sway_core::asm_to_bytecode(asm_res, source_map)
    );

    let errored =
        !bc_res.errors.is_empty() || (!bc_res.warnings.is_empty() && profile.error_on_warnings);

    let compiled = match bc_res.value {
        Some(compiled) if !errored => compiled,
        _ => return fail(&bc_res.warnings, &bc_res.errors),
    };

    print_warnings(terse_mode, &pkg.name, &bc_res.warnings, &tree_type);

    // TODO: This should probably be in `fuel_abi_json::generate_json_abi_program`?
    // If ABI requires knowing config offsets, they should be inputs to ABI gen.
    if let ProgramABI::Fuel(ref mut program_abi) = program_abi {
        if let Some(ref mut configurables) = program_abi.configurables {
            // Filter out all dead configurables (i.e. ones without offsets in the bytecode)
            configurables.retain(|c| compiled.config_const_offsets.contains_key(&c.name));
            // Set the actual offsets in the JSON object
            for (config, offset) in compiled.config_const_offsets {
                if let Some(idx) = configurables.iter().position(|c| c.name == config) {
                    configurables[idx].offset = offset
                }
            }
        }
    }

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
        namespace,
        warnings: bc_res.warnings,
    };
    Ok(compiled_package)
}

impl PkgEntry {
    /// Returns whether this `PkgEntry` corresponds to a test.
    pub fn is_test(&self) -> bool {
        self.kind.test().is_some()
    }

    fn from_finalized_entry(
        finalized_entry: &FinalizedEntry,
        decl_engine: &DeclEngine,
    ) -> Result<Self> {
        let pkg_entry_kind = match &finalized_entry.test_decl_ref {
            Some(test_decl_ref) => {
                let pkg_test_entry = PkgTestEntry::from_decl(test_decl_ref.clone(), decl_engine)?;
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
    fn from_decl(decl_ref: DeclRefFunction, decl_engine: &DeclEngine) -> Result<Self> {
        let span = decl_ref.span();
        let test_function_decl = decl_engine.get_function(&decl_ref);

        let test_args: HashSet<String> = test_function_decl
            .attributes
            .get(&AttributeKind::Test)
            .expect("test declaration is missing test attribute")
            .iter()
            .flat_map(|attr| attr.args.iter().map(|arg| arg.name.to_string()))
            .collect();

        let pass_condition = if test_args.is_empty() {
            anyhow::Ok(TestPassCondition::ShouldNotRevert)
        } else if test_args.get("should_revert").is_some() {
            anyhow::Ok(TestPassCondition::ShouldRevert)
        } else {
            let test_name = &test_function_decl.name;
            bail!("Invalid test argument(s) for test: {test_name}.")
        }?;

        Ok(Self {
            pass_condition,
            span,
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
) -> Result<(String, BuildProfile)> {
    let BuildOpts {
        pkg,
        print,
        build_profile,
        release,
        time_phases,
        tests,
        error_on_warnings,
        ..
    } = build_options;
    let mut selected_build_profile = BuildProfile::DEBUG;

    match &build_profile {
        Some(build_profile) => {
            if *release {
                warn!(
                    "You specified both {} and 'release' profiles. Using the 'release' profile",
                    build_profile
                );
                selected_build_profile = BuildProfile::RELEASE;
            } else {
                selected_build_profile = build_profile;
            }
        }
        None => {
            if *release {
                selected_build_profile = BuildProfile::RELEASE;
            }
        }
    }

    // Retrieve the specified build profile
    let mut profile = build_profiles
        .get(selected_build_profile)
        .cloned()
        .unwrap_or_else(|| {
            warn!(
                "provided profile option {} is not present in the manifest file. \
            Using default profile.",
                selected_build_profile
            );
            Default::default()
        });
    profile.print_ast |= print.ast;
    profile.print_dca_graph |= print.dca_graph;
    profile.print_ir |= print.ir;
    profile.print_finalized_asm |= print.finalized_asm;
    profile.print_intermediate_asm |= print.intermediate_asm;
    profile.terse |= pkg.terse;
    profile.time_phases |= time_phases;
    profile.include_tests |= tests;
    profile.json_abi_with_callpaths |= pkg.json_abi_with_callpaths;
    profile.error_on_warnings |= error_on_warnings;

    Ok((selected_build_profile.to_string(), profile))
}

/// Check if the given node is a contract dependency of any node in the graph.
fn is_contract_dependency(graph: &Graph, node: NodeIx) -> bool {
    graph
        .edges_directed(node, Direction::Incoming)
        .any(|e| matches!(e.weight().kind, DepKind::Contract { .. }))
}

/// Builds a project with given BuildOptions.
pub fn build_with_options(build_options: BuildOpts) -> Result<Built> {
    let BuildOpts {
        minify,
        binary_outfile,
        debug_outfile,
        pkg,
        build_target,
        member_filter,
        ..
    } = &build_options;

    let current_dir = std::env::current_dir()?;
    let path = &build_options
        .pkg
        .path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| current_dir);

    let build_plan = BuildPlan::from_build_opts(&build_options)?;
    let graph = build_plan.graph();
    let manifest_map = build_plan.manifest_map();

    // Check if manifest used to create the build plan is one of the member manifests or a
    // workspace manifest.
    let curr_manifest = manifest_map
        .values()
        .find(|&pkg_manifest| pkg_manifest.dir() == path);
    let build_profiles: HashMap<String, BuildProfile> = build_plan.build_profiles().collect();
    // Get the selected build profile using build options
    let (profile_name, build_profile) = build_profile_from_opts(&build_profiles, &build_options)?;
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
    let built_packages = build(&build_plan, *build_target, &build_profile, &outputs)?;
    let output_dir = pkg.output_directory.as_ref().map(PathBuf::from);

    let finished = ansi_term::Colour::Green.bold().paint("Finished");
    info!("  {finished} {profile_name} in {:?}", build_start.elapsed());
    for (node_ix, built_package) in built_packages.into_iter() {
        print_pkg_summary_header(&built_package);
        let pinned = &graph[node_ix];
        let pkg_manifest = manifest_map
            .get(&pinned.id())
            .ok_or_else(|| anyhow!("Couldn't find member manifest for {}", pinned.name))?;
        let output_dir = output_dir
            .clone()
            .unwrap_or_else(|| default_output_directory(pkg_manifest.dir()).join(&profile_name));
        // Output artifacts for the built package
        if let Some(outfile) = &binary_outfile {
            built_package.write_bytecode(outfile.as_ref())?;
        }
        if let Some(outfile) = &debug_outfile {
            built_package.write_debug_info(outfile.as_ref())?;
        }
        built_package.write_output(minify.clone(), &pkg_manifest.project.name, &output_dir)?;
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
    // The ansi_term formatters ignore the `std::fmt` right-align
    // formatter, so we manually calculate the padding to align the program
    // type and name around the 10th column ourselves.
    let padded_ty_str = format!("{prog_ty_str:>10}");
    let padding = &padded_ty_str[..padded_ty_str.len() - prog_ty_str.len()];
    let ty_ansi = ansi_term::Colour::Green.bold().paint(prog_ty_str);
    let name_ansi = ansi_term::Style::new()
        .bold()
        .paint(&built_pkg.descriptor.name);
    info!("{padding}{ty_ansi} {name_ansi}");
}

/// Returns the ContractId of a built_package contract with specified `salt`.
pub fn contract_id(
    bytecode: Vec<u8>,
    mut storage_slots: Vec<StorageSlot>,
    salt: &fuel_tx::Salt,
) -> ContractId {
    // Construct the contract ID
    let contract = Contract::from(bytecode);
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    contract.id(salt, &contract.root(), &state_root)
}

/// Checks if there are conficting `Salt` declarations for the contract dependencies in the graph.
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
) -> anyhow::Result<Vec<(NodeIx, BuiltPackage)>> {
    let mut built_packages = Vec::new();

    let required: HashSet<NodeIx> = outputs
        .iter()
        .flat_map(|output_node| plan.node_deps(*output_node))
        .collect();

    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine);
    let include_tests = profile.include_tests;

    // This is the Contract ID of the current contract being compiled.
    // We will need this for `forc test`.
    let mut contract_id_value: Option<ContractIdConst> = None;

    let mut lib_namespace_map = Default::default();
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

        print_compiling(
            program_ty.as_ref(),
            &pkg.name,
            &pkg.source.display_compiling(manifest.dir()),
        );

        let descriptor = PackageDescriptor {
            name: pkg.name.clone(),
            target,
            pinned: pkg.clone(),
            manifest_file: manifest.clone(),
        };

        let fail = |warnings, errors| {
            print_on_failure(profile.terse, warnings, errors);
            bail!("Failed to compile {}", pkg.name);
        };

        let is_contract_dependency = is_contract_dependency(plan.graph(), node);
        // If we are building a contract and tests are enabled or we are building a contract
        // dependency, we need the tests exlcuded bytecode.
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

            // `ContractIdConst` is a None here since we do not yet have a
            // contract ID value at this point.
            let dep_namespace = match dependency_namespace(
                &lib_namespace_map,
                &compiled_contract_deps,
                plan.graph(),
                node,
                engines,
                None,
            ) {
                Ok(o) => o,
                Err(errs) => return fail(&[], &errs),
            };
            let compiled_without_tests = compile(
                &descriptor,
                &profile,
                engines,
                dep_namespace,
                &mut source_map,
            )?;

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
                    compiled_without_tests.bytecode.bytes.clone(),
                    compiled_without_tests.storage_slots,
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

        // Note that the contract ID value here is only Some if tests are enabled.
        let dep_namespace = match dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            plan.graph(),
            node,
            engines,
            contract_id_value.clone(),
        ) {
            Ok(o) => o,
            Err(errs) => return fail(&[], &errs),
        };

        let mut compiled = compile(
            &descriptor,
            &profile,
            engines,
            dep_namespace,
            &mut source_map,
        )?;

        if let TreeType::Library = compiled.tree_type {
            let mut namespace = namespace::Module::from(compiled.namespace);
            namespace.name = Some(Ident::new_no_span(pkg.name.clone()));
            lib_namespace_map.insert(node, namespace);
        }
        source_map.insert_dependency(descriptor.manifest_file.dir());

        // TODO: This should probably be in `fuel_abi_json::generate_json_abi_program`?
        if let ProgramABI::Fuel(ref mut program_abi) = compiled.program_abi {
            standardize_json_abi_types(program_abi);
        }

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

/// Standardize the JSON ABI data structure by eliminating duplicate types. This is an iterative
/// process because every time two types are merged, new opportunities for more merging arise.
fn standardize_json_abi_types(json_abi_program: &mut program_abi::ProgramABI) {
    loop {
        // If type with id_1 is a duplicate of type with id_2, then keep track of the mapping
        // between id_1 and id_2 in the HashMap below.
        let mut old_to_new_id: HashMap<usize, usize> = HashMap::new();

        // A vector containing unique `program_abi::TypeDeclaration`s.
        //
        // Two `program_abi::TypeDeclaration` are deemed the same if the have the same
        // `type_field`, `components`, and `type_parameters` (even if their `type_id`s are
        // different).
        let mut deduped_types: Vec<program_abi::TypeDeclaration> = Vec::new();

        // Insert values in `deduped_types` if they haven't been inserted before. Otherwise, create
        // an appropriate mapping between type IDs in the HashMap `old_to_new_id`.
        for decl in json_abi_program.types.iter() {
            if let Some(ty) = deduped_types.iter().find(|d| {
                d.type_field == decl.type_field
                    && d.components == decl.components
                    && d.type_parameters == decl.type_parameters
            }) {
                old_to_new_id.insert(decl.type_id, ty.type_id);
            } else {
                deduped_types.push(decl.clone());
            }
        }

        // Nothing to do if the hash map is empty as there are not merge opportunities. We can now
        // exit the loop.
        if old_to_new_id.is_empty() {
            break;
        }

        json_abi_program.types = deduped_types;

        // Update all `program_abi::TypeApplication`s and all `program_abi::TypeDeclaration`s
        update_all_types(json_abi_program, &old_to_new_id);
    }

    // Sort the `program_abi::TypeDeclaration`s
    json_abi_program
        .types
        .sort_by(|t1, t2| t1.type_field.cmp(&t2.type_field));

    // Standardize IDs (i.e. change them to 0,1,2,... according to the alphabetical order above
    let mut old_to_new_id: HashMap<usize, usize> = HashMap::new();
    for (ix, decl) in json_abi_program.types.iter_mut().enumerate() {
        old_to_new_id.insert(decl.type_id, ix);
        decl.type_id = ix;
    }

    // Update all `program_abi::TypeApplication`s and all `program_abi::TypeDeclaration`s
    update_all_types(json_abi_program, &old_to_new_id);
}

/// Recursively updates the type IDs used in a program_abi::ProgramABI
fn update_all_types(
    json_abi_program: &mut program_abi::ProgramABI,
    old_to_new_id: &HashMap<usize, usize>,
) {
    // Update all `program_abi::TypeApplication`s in every function
    for func in json_abi_program.functions.iter_mut() {
        for input in func.inputs.iter_mut() {
            update_json_type_application(input, old_to_new_id);
        }

        update_json_type_application(&mut func.output, old_to_new_id);
    }

    // Update all `program_abi::TypeDeclaration`
    for decl in json_abi_program.types.iter_mut() {
        update_json_type_declaration(decl, old_to_new_id);
    }
    if let Some(logged_types) = &mut json_abi_program.logged_types {
        for logged_type in logged_types.iter_mut() {
            update_json_type_application(&mut logged_type.application, old_to_new_id);
        }
    }
    if let Some(messages_types) = &mut json_abi_program.messages_types {
        for logged_type in messages_types.iter_mut() {
            update_json_type_application(&mut logged_type.application, old_to_new_id);
        }
    }
    if let Some(configurables) = &mut json_abi_program.configurables {
        for logged_type in configurables.iter_mut() {
            update_json_type_application(&mut logged_type.application, old_to_new_id);
        }
    }
}

/// Recursively updates the type IDs used in a `program_abi::TypeApplication` given a HashMap from
/// old to new IDs
fn update_json_type_application(
    type_application: &mut program_abi::TypeApplication,
    old_to_new_id: &HashMap<usize, usize>,
) {
    if let Some(new_id) = old_to_new_id.get(&type_application.type_id) {
        type_application.type_id = *new_id;
    }

    if let Some(args) = &mut type_application.type_arguments {
        for arg in args.iter_mut() {
            update_json_type_application(arg, old_to_new_id);
        }
    }
}

/// Recursively updates the type IDs used in a `program_abi::TypeDeclaration` given a HashMap from
/// old to new IDs
fn update_json_type_declaration(
    type_declaration: &mut program_abi::TypeDeclaration,
    old_to_new_id: &HashMap<usize, usize>,
) {
    if let Some(params) = &mut type_declaration.type_parameters {
        for param in params.iter_mut() {
            if let Some(new_id) = old_to_new_id.get(param) {
                *param = *new_id;
            }
        }
    }

    if let Some(components) = &mut type_declaration.components {
        for component in components.iter_mut() {
            update_json_type_application(component, old_to_new_id);
        }
    }
}

/// Compile the entire forc package and return the lexed, parsed and typed programs
/// of the dependancies and project.
/// The final item in the returned vector is the project.
pub fn check(
    plan: &BuildPlan,
    build_target: BuildTarget,
    terse_mode: bool,
    include_tests: bool,
    engines: Engines<'_>,
) -> anyhow::Result<Vec<CompileResult<Programs>>> {
    let mut lib_namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    // During `check`, we don't compile so this stays empty.
    let compiled_contract_deps = HashMap::new();

    let mut results = vec![];
    for &node in plan.compilation_order.iter() {
        let pkg = &plan.graph[node];
        let manifest = &plan.manifest_map()[&pkg.id()];
        let dep_namespace = dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            &plan.graph,
            node,
            engines,
            None,
        )
        .expect("failed to create dependency namespace");

        let CompileResult {
            value,
            mut warnings,
            mut errors,
        } = parse(manifest, build_target, terse_mode, include_tests, engines)?;

        let (lexed, parsed) = match value {
            None => {
                results.push(CompileResult::new(None, warnings, errors));
                return Ok(results);
            }
            Some(modules) => modules,
        };

        let ast_result = sway_core::parsed_to_ast(engines, &parsed, dep_namespace, None, &pkg.name);
        warnings.extend(ast_result.warnings);
        errors.extend(ast_result.errors);

        let typed_program = match ast_result.value {
            None => {
                let value = Some(Programs::new(lexed, parsed, None));
                results.push(CompileResult::new(value, warnings, errors));
                return Ok(results);
            }
            Some(typed_program) => typed_program,
        };

        if let TreeType::Library = typed_program.kind.tree_type() {
            let mut namespace = typed_program.root.namespace.clone();
            namespace.name = Some(Ident::new_no_span(pkg.name.clone()));
            namespace.span = Some(
                Span::new(
                    manifest.entry_string()?,
                    0,
                    0,
                    Some(manifest.entry_path().into()),
                )
                .unwrap(),
            );
            lib_namespace_map.insert(node, namespace);
        }

        source_map.insert_dependency(manifest.dir());

        let value = Some(Programs::new(lexed, parsed, Some(typed_program)));
        results.push(CompileResult::new(value, warnings, errors));
    }

    if results.is_empty() {
        bail!("unable to check sway program: build plan contains no packages")
    }

    Ok(results)
}

/// Returns a parsed AST from the supplied [PackageManifestFile]
pub fn parse(
    manifest: &PackageManifestFile,
    build_target: BuildTarget,
    terse_mode: bool,
    include_tests: bool,
    engines: Engines<'_>,
) -> anyhow::Result<CompileResult<(LexedProgram, ParseProgram)>> {
    let profile = BuildProfile {
        terse: terse_mode,
        ..BuildProfile::debug()
    };
    let source = manifest.entry_string()?;
    let sway_build_config = sway_build_config(
        manifest.dir(),
        &manifest.entry_path(),
        build_target,
        &profile,
    )?
    .include_tests(include_tests);
    Ok(sway_core::parse(source, engines, Some(&sway_build_config)))
}

/// Format an error message for an absent `Forc.toml`.
pub fn manifest_file_missing(dir: &Path) -> anyhow::Error {
    let message = format!(
        "could not find `{}` in `{}` or any parent directory",
        constants::MANIFEST_FILE_NAME,
        dir.display()
    );
    Error::msg(message)
}

/// Format an error message for failed parsing of a manifest.
pub fn parsing_failed(project_name: &str, errors: Vec<CompileError>) -> anyhow::Error {
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
    expected_types: Vec<TreeType>,
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

#[test]
fn test_root_pkg_order() {
    let current_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(current_dir)
        .parent()
        .unwrap()
        .join("test/src/e2e_vm_tests/test_programs/should_pass/forc/workspace_building/");
    let manifest_file = ManifestFile::from_dir(&manifest_dir).unwrap();
    let member_manifests = manifest_file.member_manifests().unwrap();
    let lock_path = manifest_file.lock_path().unwrap();
    let build_plan =
        BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, false, false).unwrap();
    let graph = build_plan.graph();
    let order: Vec<String> = build_plan
        .member_nodes()
        .map(|order| graph[order].name.clone())
        .collect();
    assert_eq!(order, vec!["test_lib", "test_contract", "test_script"])
}
