use crate::{
    lock::Lock,
    manifest::{
        BuildProfile, ConfigTimeConstant, Dependency, ManifestFile, MemberManifestFiles,
        PackageManifest, PackageManifestFile,
    },
    WorkspaceManifestFile, CORE, PRELUDE, STD,
};
use anyhow::{anyhow, bail, Context, Error, Result};
use forc_util::{
    default_output_directory, find_file_name, git_checkouts_directory, kebab_to_snake_case,
    print_on_failure, print_on_success,
};
use petgraph::{
    self,
    visit::{Bfs, Dfs, EdgeRef, Walker},
    Directed, Direction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeMap, BTreeSet, HashMap, HashSet},
    fmt,
    fs::{self, File},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};
use sway_core::fuel_prelude::{
    fuel_crypto,
    fuel_tx::{self, Contract, ContractId, StorageSlot},
};
use sway_core::{
    language::{
        parsed::{ParseProgram, TreeType},
        ty,
    },
    semantic_analysis::namespace,
    source_map::SourceMap,
    CompileResult, CompiledBytecode, FinalizedEntry,
};
use sway_error::error::CompileError;
use sway_types::{Ident, JsonABIProgram, JsonTypeApplication, JsonTypeDeclaration};
use sway_utils::constants;
use tracing::{info, warn};
use url::Url;

type GraphIx = u32;
type Node = Pinned;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Edge {
    /// The name of the dependency as declared under `[dependencies]` or `[contract-dependencies]`.
    /// This may differ from the package name as declared under the dependency package's manifest.
    pub name: String,
    pub kind: DepKind,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DepKind {
    /// The dependency is a library and declared under `[dependencies]`.
    Library,
    /// The dependency is a contract and declared under `[contract-dependencies]`.
    Contract,
}
pub type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;
pub type EdgeIx = petgraph::graph::EdgeIndex<GraphIx>;
pub type NodeIx = petgraph::graph::NodeIndex<GraphIx>;
pub type ManifestMap = HashMap<PinnedId, PackageManifestFile>;

/// A unique ID for a pinned package.
///
/// The internal value is produced by hashing the package's name and `SourcePinned`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PinnedId(u64);

/// The result of successfully compiling a package.
#[derive(Debug, Clone)]
pub struct BuiltPackage {
    pub json_abi_program: JsonABIProgram,
    pub storage_slots: Vec<StorageSlot>,
    pub bytecode: Vec<u8>,
    pub entries: Vec<FinalizedEntry>,
    pub tree_type: TreeType,
}

#[derive(Debug)]
pub enum Built {
    Package(Box<BuiltPackage>),
    Workspace(Vec<BuiltPackage>),
}

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
    pub source: SourcePinned,
}

/// Specifies a base source for a package.
///
/// - For registry packages, this includes a base version.
/// - For git packages, this includes a base git reference like a branch or tag.
///
/// Note that a `Source` does not specify a specific, pinned version. Rather, it specifies a source
/// at which the current latest version may be located.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Source {
    /// Used to refer to the root project.
    Member(PathBuf),
    /// A git repo with a `Forc.toml` manifest at its root.
    Git(SourceGit),
    /// A path to a directory with a `Forc.toml` manifest at its root.
    Path(PathBuf),
    /// A forc project hosted on the official registry.
    Registry(SourceRegistry),
}

/// A git repo with a `Forc.toml` manifest at its root.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SourceGit {
    /// The URL at which the repository is located.
    pub repo: Url,
    /// A git reference, e.g. a branch or tag.
    pub reference: GitReference,
}

/// Used to distinguish between types of git references.
///
/// For the most part, `GitReference` is useful to refine the `refspecs` used to fetch remote
/// repositories.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum GitReference {
    Branch(String),
    Tag(String),
    Rev(String),
    DefaultBranch,
}

/// A package from the official registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SourceRegistry {
    /// The base version specified for the package.
    pub version: semver::Version,
}

/// A pinned instance of a git source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SourceGitPinned {
    /// The git source that is being pinned.
    pub source: SourceGit,
    /// The hash to which we have pinned the source.
    pub commit_hash: String,
}

/// A pinned instance of a path source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SourcePathPinned {
    /// The ID of the package that is the root of the subgraph of path dependencies that this
    /// package is a part of.
    ///
    /// In other words, when traversing the parents of this package, this is the ID of the first
    /// non-path ancestor package.
    ///
    /// As a result, this will always be either a git package or the root package.
    ///
    /// This allows for disambiguating path dependencies of the same name that have different path
    /// roots.
    pub path_root: PinnedId,
}

/// A pinned instance of the registry source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SourceRegistryPinned {
    /// The registry package with base version.
    pub source: SourceRegistry,
    /// The pinned version.
    pub version: semver::Version,
}

/// A pinned instance of the package source.
///
/// Specifies an exact version to use, or an exact commit in the case of git dependencies. The
/// pinned version or commit is updated upon creation of the lock file and on `forc update`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum SourcePinned {
    Member,
    Git(SourceGitPinned),
    Path(SourcePathPinned),
    Registry(SourceRegistryPinned),
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

/// Error returned upon failed parsing of `SourcePathPinned::from_str`.
#[derive(Clone, Debug)]
pub struct SourcePathPinnedParseError;

/// Error returned upon failed parsing of `SourceGitPinned::from_str`.
#[derive(Clone, Debug)]
pub enum SourceGitPinnedParseError {
    Prefix,
    Url,
    Reference,
    CommitHash,
}

/// Represents the Head's commit hash and time (in seconds) from epoch
type HeadWithTime = (String, i64);

/// Everything needed to recognize a checkout in offline mode
///
/// Since we are omiting `.git` folder to save disk space, we need an indexing file
/// to recognize a checkout while searching local checkouts in offline mode
#[derive(Serialize, Deserialize)]
pub struct GitSourceIndex {
    /// Type of the git reference
    pub git_reference: GitReference,
    pub head_with_time: HeadWithTime,
}

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
}

#[derive(Default, Clone)]
pub struct PrintOpts {
    /// Print the generated Sway AST (Abstract Syntax Tree).
    pub ast: bool,
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

/// The set of options provided to the `build` functions.
#[derive(Default, Clone)]
pub struct BuildOpts {
    pub pkg: PkgOpts,
    pub print: PrintOpts,
    pub minify: MinifyOpts,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    pub debug_outfile: Option<String>,
    /// Name of the build profile to use.
    /// If it is not specified, forc will use debug build profile.
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    ///  If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    pub release: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
    /// Include all test functions within the build.
    pub tests: bool,
}

impl GitSourceIndex {
    pub fn new(time: i64, git_reference: GitReference, commit_hash: String) -> GitSourceIndex {
        GitSourceIndex {
            git_reference,
            head_with_time: (commit_hash, time),
        }
    }
}

impl Edge {
    pub fn new(name: String, kind: DepKind) -> Edge {
        Edge { name, kind }
    }
}

impl FromStr for DepKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "library" => Ok(DepKind::Library),
            "contract" => Ok(DepKind::Contract),
            _ => bail!("invalid dep kind"),
        }
    }
}

impl fmt::Display for DepKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DepKind::Library => write!(f, "library"),
            DepKind::Contract => write!(f, "contract"),
        }
    }
}

impl BuiltPackage {
    fn output_artifacts(
        &self,
        source_map: &SourceMap,
        build_options: BuildOpts,
        pkg_manifest: &PackageManifestFile,
        output_dir: &Path,
    ) -> Result<()> {
        let BuildOpts {
            minify,
            binary_outfile,
            debug_outfile,
            ..
        } = build_options;
        if let Some(outfile) = binary_outfile {
            fs::write(&outfile, &self.bytecode)?;
        }

        if let Some(outfile) = debug_outfile {
            let source_map_json =
                serde_json::to_vec(&source_map).expect("JSON serialization failed");
            fs::write(outfile, &source_map_json)?;
        }
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }
        // Place build artifacts into the output directory.
        let bin_path = output_dir
            .join(&pkg_manifest.project.name)
            .with_extension("bin");

        fs::write(&bin_path, &self.bytecode)?;

        if !self.json_abi_program.functions.is_empty() {
            let json_abi_program_stem = format!("{}-abi", pkg_manifest.project.name);
            let json_abi_program_path = output_dir
                .join(&json_abi_program_stem)
                .with_extension("json");
            let file = File::create(json_abi_program_path)?;
            let res = if minify.json_abi {
                serde_json::to_writer(&file, &self.json_abi_program)
            } else {
                serde_json::to_writer_pretty(&file, &self.json_abi_program)
            };
            res?
        }
        info!("  Bytecode size is {} bytes.", self.bytecode.len());
        // Additional ops required depending on the program type
        match self.tree_type {
            TreeType::Contract => {
                // For contracts, emit a JSON file with all the initialized storage slots.
                let json_storage_slots_stem =
                    format!("{}-storage_slots", pkg_manifest.project.name);
                let json_storage_slots_path = output_dir
                    .join(&json_storage_slots_stem)
                    .with_extension("json");
                let storage_slots_file = File::create(json_storage_slots_path)?;
                let res = if minify.json_storage_slots {
                    serde_json::to_writer(&storage_slots_file, &self.storage_slots)
                } else {
                    serde_json::to_writer_pretty(&storage_slots_file, &self.storage_slots)
                };

                res?;
            }
            TreeType::Predicate => {
                // get the root hash of the bytecode for predicates and store the result in a file in the output directory
                let root = format!("0x{}", Contract::root_from_code(&self.bytecode));
                let root_file_name =
                    format!("{}{}", &pkg_manifest.project.name, SWAY_BIN_ROOT_SUFFIX);
                let root_path = output_dir.join(root_file_name);
                fs::write(root_path, &root)?;
                info!("  Predicate root: {}", root);
            }
            TreeType::Script => {
                // hash the bytecode for scripts and store the result in a file in the output directory
                let bytecode_hash = format!("0x{}", fuel_crypto::Hasher::hash(&self.bytecode));
                let hash_file_name =
                    format!("{}{}", &pkg_manifest.project.name, SWAY_BIN_HASH_SUFFIX);
                let hash_path = output_dir.join(hash_file_name);
                fs::write(hash_path, &bytecode_hash)?;
                info!("  Script bytecode hash: {}", bytecode_hash);
            }
            _ => (),
        }

        Ok(())
    }
}
const DEFAULT_REMOTE_NAME: &str = "origin";

/// Error returned upon failed parsing of `SourcePinned::from_str`.
#[derive(Clone, Debug)]
pub struct SourcePinnedParseError;

/// The name specified on the left hand side of the `=` in a depenedency declaration under
/// `[dependencies]` within a forc manifest.
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
pub type DependencyName = String;

impl BuildPlan {
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
                .map(|(_, manifest)| manifest.project.name.clone())
                .collect();
            crate::lock::print_diff(&member_names, &lock_diff);
            let string = toml::ser::to_string_pretty(&new_lock)
                .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
            fs::write(lock_path, &string)
                .map_err(|e| anyhow!("failed to write lock file: {}", e))?;
            info!("   Created new lock file at {}", lock_path.display());
        }

        Ok(plan)
    }

    /// Produce an iterator yielding all workspace member nodes in order of compilation.
    ///
    /// In the case that this `BuildPlan` was constructed for a single package,
    /// only that package's node will be yielded.
    pub fn member_nodes(&self) -> impl Iterator<Item = NodeIx> + '_ {
        self.compilation_order()
            .iter()
            .cloned()
            .filter(|&n| self.graph[n].source == SourcePinned::Member)
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
        // Collect visitable nodes from the given node in the graph.
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
        .filter(|&n| g[n].source == SourcePinned::Member)
}

/// Validates the state of the pinned package graph against the given ManifestFile.
///
/// Returns the set of invalid dependency edges.
fn validate_graph(graph: &Graph, manifests: &MemberManifestFiles) -> Result<BTreeSet<EdgeIx>> {
    let mut member_pkgs: HashMap<&String, &PackageManifestFile> = manifests.iter().collect();
    let member_nodes: Vec<_> = member_nodes(graph)
        .filter_map(|n| member_pkgs.remove(&graph[n].name).map(|pkg| (n, pkg)))
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
    let dep_source = dep_to_source_patched(node_manifest, dep_name, dep_entry)?;
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
        (TreeType::Contract, DepKind::Contract) | (TreeType::Library { .. }, DepKind::Library) => {}
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
/// Also returns `None` in the case that the dependency is a `Path` dependency and the path root is
/// invalid.
fn dep_path(
    graph: &Graph,
    node_manifest: &PackageManifestFile,
    dep_node: NodeIx,
    manifests: &MemberManifestFiles,
) -> Result<PathBuf> {
    let dep = &graph[dep_node];
    let dep_name = &dep.name;
    match &dep.source {
        SourcePinned::Git(git) => {
            let repo_path = git_commit_path(&dep.name, &git.source.repo, &git.commit_hash);
            find_dir_within(&repo_path, &dep.name).ok_or_else(|| {
                anyhow!(
                    "failed to find package `{}` in {}",
                    dep.name,
                    git.to_string()
                )
            })
        }
        SourcePinned::Path(src) => {
            validate_path_root(graph, dep_node, src.path_root)?;

            // Check if the path is directly from the dependency.
            if let Some(path) = node_manifest.dep_path(dep_name) {
                if path.exists() {
                    return Ok(path);
                }
            }

            // Otherwise, check if it comes from a patch.
            for (_, patch_map) in node_manifest.patches() {
                if let Some(Dependency::Detailed(details)) = patch_map.get(dep_name) {
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
        SourcePinned::Registry(_reg) => unreachable!("registry dependencies not yet supported"),
        SourcePinned::Member => {
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
        .filter(|&n| member_names.contains(&graph[n].name))
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

impl GitReference {
    /// Resolves the parsed forc git reference to the associated git ID.
    pub fn resolve(&self, repo: &git2::Repository) -> Result<git2::Oid> {
        // Find the commit associated with this tag.
        fn resolve_tag(repo: &git2::Repository, tag: &str) -> Result<git2::Oid> {
            let refname = format!("refs/remotes/{}/tags/{}", DEFAULT_REMOTE_NAME, tag);
            let id = repo.refname_to_id(&refname)?;
            let obj = repo.find_object(id, None)?;
            let obj = obj.peel(git2::ObjectType::Commit)?;
            Ok(obj.id())
        }

        // Resolve to the target for the given branch.
        fn resolve_branch(repo: &git2::Repository, branch: &str) -> Result<git2::Oid> {
            let name = format!("{}/{}", DEFAULT_REMOTE_NAME, branch);
            let b = repo
                .find_branch(&name, git2::BranchType::Remote)
                .with_context(|| format!("failed to find branch `{}`", branch))?;
            b.get()
                .target()
                .ok_or_else(|| anyhow::format_err!("branch `{}` did not have a target", branch))
        }

        // Use the HEAD commit when default branch is specified.
        fn resolve_default_branch(repo: &git2::Repository) -> Result<git2::Oid> {
            let head_id =
                repo.refname_to_id(&format!("refs/remotes/{}/HEAD", DEFAULT_REMOTE_NAME))?;
            let head = repo.find_object(head_id, None)?;
            Ok(head.peel(git2::ObjectType::Commit)?.id())
        }

        // Find the commit for the given revision.
        fn resolve_rev(repo: &git2::Repository, rev: &str) -> Result<git2::Oid> {
            let obj = repo.revparse_single(rev)?;
            match obj.as_tag() {
                Some(tag) => Ok(tag.target_id()),
                None => Ok(obj.id()),
            }
        }

        match self {
            GitReference::Tag(s) => {
                resolve_tag(repo, s).with_context(|| format!("failed to find tag `{}`", s))
            }
            GitReference::Branch(s) => resolve_branch(repo, s),
            GitReference::DefaultBranch => resolve_default_branch(repo),
            GitReference::Rev(s) => resolve_rev(repo, s),
        }
    }
}

impl Pinned {
    /// Retrieve the unique ID for the pinned package.
    ///
    /// The internal value is produced by hashing the package's name and `SourcePinned`.
    pub fn id(&self) -> PinnedId {
        PinnedId::new(&self.name, &self.source)
    }

    /// Retrieve the unpinned version of this source.
    pub fn unpinned(&self, path: &Path) -> Pkg {
        let source = match &self.source {
            SourcePinned::Member => Source::Member(path.to_owned()),
            SourcePinned::Git(git) => Source::Git(git.source.clone()),
            SourcePinned::Path(_) => Source::Path(path.to_owned()),
            SourcePinned::Registry(reg) => Source::Registry(reg.source.clone()),
        };
        let name = self.name.clone();
        Pkg { name, source }
    }
}

impl PinnedId {
    /// Hash the given name and pinned source to produce a unique pinned package ID.
    pub fn new(name: &str, source: &SourcePinned) -> Self {
        let mut hasher = hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        source.hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl SourcePathPinned {
    pub const PREFIX: &'static str = "path";
}

impl SourceGitPinned {
    pub const PREFIX: &'static str = "git";
}

impl fmt::Display for PinnedId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format the inner `u64` as hex.
        write!(f, "{:016X}", self.0)
    }
}

impl fmt::Display for SourcePathPinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // path+from-root-<id>
        write!(f, "{}+from-root-{}", Self::PREFIX, self.path_root)
    }
}

impl fmt::Display for SourceGitPinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // git+<url/to/repo>?<ref_kind>=<ref_string>#<commit>
        write!(
            f,
            "{}+{}?{}#{}",
            Self::PREFIX,
            self.source.repo,
            self.source.reference,
            self.commit_hash
        )
    }
}

impl fmt::Display for GitReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitReference::Branch(ref s) => write!(f, "branch={}", s),
            GitReference::Tag(ref s) => write!(f, "tag={}", s),
            GitReference::Rev(ref _s) => write!(f, "rev"),
            GitReference::DefaultBranch => write!(f, "default-branch"),
        }
    }
}

impl fmt::Display for SourcePinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SourcePinned::Member => write!(f, "member"),
            SourcePinned::Path(src) => src.fmt(f),
            SourcePinned::Git(src) => src.fmt(f),
            SourcePinned::Registry(_reg) => unimplemented!("pkg registries not yet implemented"),
        }
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

impl FromStr for SourcePathPinned {
    type Err = SourcePathPinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // path+from-root-<id>
        let s = s.trim();

        // Check for prefix at the start.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus) != Some(0) {
            return Err(SourcePathPinnedParseError);
        }
        let s = &s[prefix_plus.len()..];

        // Parse the `from-root-*` section.
        let path_root = s
            .split("from-root-")
            .nth(1)
            .ok_or(SourcePathPinnedParseError)?
            .parse()
            .map_err(|_| SourcePathPinnedParseError)?;

        Ok(Self { path_root })
    }
}

impl FromStr for SourceGitPinned {
    type Err = SourceGitPinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // git+<url/to/repo>?<reference>#<commit>
        let s = s.trim();

        // Check for "git+" at the start.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus) != Some(0) {
            return Err(SourceGitPinnedParseError::Prefix);
        }
        let s = &s[prefix_plus.len()..];

        // Parse the `repo` URL.
        let repo_str = s.split('?').next().ok_or(SourceGitPinnedParseError::Url)?;
        let repo = Url::parse(repo_str).map_err(|_| SourceGitPinnedParseError::Url)?;
        let s = &s[repo_str.len() + "?".len()..];

        // Parse the git reference and commit hash. This can be any of either:
        // - `branch=<branch-name>#<commit-hash>`
        // - `tag=<tag-name>#<commit-hash>`
        // - `rev#<commit-hash>`
        // - `default#<commit-hash>`
        let mut s_iter = s.split('#');
        let reference = s_iter.next().ok_or(SourceGitPinnedParseError::Reference)?;
        let commit_hash = s_iter
            .next()
            .ok_or(SourceGitPinnedParseError::CommitHash)?
            .to_string();
        validate_git_commit_hash(&commit_hash)
            .map_err(|_| SourceGitPinnedParseError::CommitHash)?;

        const BRANCH: &str = "branch=";
        const TAG: &str = "tag=";
        let reference = if reference.find(BRANCH) == Some(0) {
            GitReference::Branch(reference[BRANCH.len()..].to_string())
        } else if reference.find(TAG) == Some(0) {
            GitReference::Tag(reference[TAG.len()..].to_string())
        } else if reference == "rev" {
            GitReference::Rev(commit_hash.to_string())
        } else if reference == "default-branch" {
            GitReference::DefaultBranch
        } else {
            return Err(SourceGitPinnedParseError::Reference);
        };

        let source = SourceGit { repo, reference };
        Ok(Self {
            source,
            commit_hash,
        })
    }
}

impl FromStr for SourcePinned {
    type Err = SourcePinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = if s == "root" || s == "member" {
            // Also check `"root"` to support reading the legacy `Forc.lock` format and to
            // avoid breaking old projects.
            SourcePinned::Member
        } else if let Ok(src) = SourcePathPinned::from_str(s) {
            SourcePinned::Path(src)
        } else if let Ok(src) = SourceGitPinned::from_str(s) {
            SourcePinned::Git(src)
        } else {
            // TODO: Try parse registry source.
            return Err(SourcePinnedParseError);
        };
        Ok(source)
    }
}

fn validate_git_commit_hash(commit_hash: &str) -> Result<()> {
    const LEN: usize = 40;
    if commit_hash.len() != LEN {
        bail!(
            "invalid hash length: expected {}, found {}",
            LEN,
            commit_hash.len()
        );
    }
    if !commit_hash.chars().all(|c| c.is_ascii_alphanumeric()) {
        bail!("hash contains one or more non-ascii-alphanumeric characters");
    }
    Ok(())
}

impl Default for GitReference {
    fn default() -> Self {
        Self::DefaultBranch
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
            .filter_map(|edge| {
                let parent_node = edge.source();
                let dep_name = &edge.weight().name;
                let parent = &graph[parent_node];
                let parent_manifest = manifest_map.get(&parent.id())?;
                Some((parent_manifest, dep_name))
            })
            .next()
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
        match &pkg.source {
            SourcePinned::Path(src) => {
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
            SourcePinned::Git(_) | SourcePinned::Registry(_) | SourcePinned::Member => {
                return Ok(node);
            }
        }
    }
}

/// Produce a unique ID for a particular fetch pass.
///
/// This is used in the temporary git directory and allows for avoiding contention over the git
/// repo directory.
pub fn fetch_id(path: &Path, timestamp: std::time::Instant) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    hasher.finish()
}

/// Given an empty or partially completed `graph`, complete the graph.
///
/// If the given `manifest` is of type ManifestFile::Workspace resulting graph will have multiple
/// root nodes, each representing a member of the workspace. Otherwise resulting graph will only
/// have a single root node, representing the package that is described by the ManifestFile::Package
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
        )?);
    }
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
) -> Result<HashSet<NodeIx>> {
    // Retrieve the project node, or create one if it does not exist.
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(proj_node) => proj_node,
        Err(_) => {
            let name = proj_manifest.project.name.clone();
            let source = SourcePinned::Member;
            let pkg = Pinned { name, source };
            let pkg_id = pkg.id();
            manifest_map.insert(pkg_id, proj_manifest.clone());
            graph.add_node(pkg)
        }
    };

    // Traverse the rest of the graph from the root.
    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(proj_manifest.dir(), fetch_ts);
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
) -> Result<HashSet<NodeIx>> {
    let mut added = HashSet::default();
    let parent_id = graph[node].id();
    let package_manifest = &manifest_map[&parent_id];
    // If the current package is a contract, we need to first get the deployment dependencies
    let deps: Vec<(String, Dependency, DepKind)> = package_manifest
        .contract_deps()
        .map(|(n, d)| (n.clone(), d.clone(), DepKind::Contract))
        .chain(
            package_manifest
                .deps()
                .map(|(n, d)| (n.clone(), d.clone(), DepKind::Library)),
        )
        .collect();
    for (dep_name, dep, dep_kind) in deps {
        let name = dep.package().unwrap_or(&dep_name).to_string();
        let parent_manifest = &manifest_map[&parent_id];
        let source = dep_to_source_patched(parent_manifest, &name, &dep)
            .context("Failed to source dependency")?;

        // If we haven't yet fetched this dependency, fetch it, pin it and add it to the graph.
        let dep_pkg = Pkg { name, source };
        let dep_node = match fetched.entry(dep_pkg) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let dep_pinned = pin_pkg(fetch_id, path_root, entry.key(), manifest_map, offline)?;
                let dep_node = graph.add_node(dep_pinned);
                added.insert(dep_node);
                *entry.insert(dep_node)
            }
        };

        let dep_edge = Edge::new(dep_name.to_string(), dep_kind);
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
            SourcePinned::Member | SourcePinned::Git(_) | SourcePinned::Registry(_) => dep_pkg_id,
            SourcePinned::Path(_) => path_root,
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
        )?);
    }
    Ok(added)
}

/// The name to use for a package's git repository under the user's forc directory.
fn git_repo_dir_name(name: &str, repo: &Url) -> String {
    let repo_url_hash = hash_url(repo);
    format!("{}-{:x}", name, repo_url_hash)
}

fn hash_url(url: &Url) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    hasher.finish()
}

/// A temporary directory that we can use for cloning a git-sourced package's repo and discovering
/// the current HEAD for the given git reference.
///
/// The resulting directory is:
///
/// ```ignore
/// $HOME/.forc/git/checkouts/tmp/<fetch_id>-name-<repo_url_hash>
/// ```
///
/// A unique `fetch_id` may be specified to avoid contention over the git repo directory in the
/// case that multiple processes or threads may be building different projects that may require
/// fetching the same dependency.
fn tmp_git_repo_dir(fetch_id: u64, name: &str, repo: &Url) -> PathBuf {
    let repo_dir_name = format!("{:x}-{}", fetch_id, git_repo_dir_name(name, repo));
    git_checkouts_directory().join("tmp").join(repo_dir_name)
}

/// Given a git reference, build a list of `refspecs` required for the fetch opration.
///
/// Also returns whether or not our reference implies we require fetching tags.
fn git_ref_to_refspecs(reference: &GitReference) -> (Vec<String>, bool) {
    let mut refspecs = vec![];
    let mut tags = false;
    match reference {
        GitReference::Branch(s) => {
            refspecs.push(format!(
                "+refs/heads/{1}:refs/remotes/{0}/{1}",
                DEFAULT_REMOTE_NAME, s
            ));
        }
        GitReference::Tag(s) => {
            refspecs.push(format!(
                "+refs/tags/{1}:refs/remotes/{0}/tags/{1}",
                DEFAULT_REMOTE_NAME, s
            ));
        }
        GitReference::Rev(s) => {
            if s.starts_with("refs/") {
                refspecs.push(format!("+{0}:{0}", s));
            } else {
                // We can't fetch the commit directly, so we fetch all branches and tags in order
                // to find it.
                refspecs.push(format!(
                    "+refs/heads/*:refs/remotes/{}/*",
                    DEFAULT_REMOTE_NAME
                ));
                refspecs.push(format!("+HEAD:refs/remotes/{}/HEAD", DEFAULT_REMOTE_NAME));
                tags = true;
            }
        }
        GitReference::DefaultBranch => {
            refspecs.push(format!("+HEAD:refs/remotes/{}/HEAD", DEFAULT_REMOTE_NAME));
        }
    }
    (refspecs, tags)
}

/// Initializes a temporary git repo for the package and fetches only the reference associated with
/// the given source.
fn with_tmp_git_repo<F, O>(fetch_id: u64, name: &str, source: &SourceGit, f: F) -> Result<O>
where
    F: FnOnce(git2::Repository) -> Result<O>,
{
    // Clear existing temporary directory if it exists.
    let repo_dir = tmp_git_repo_dir(fetch_id, name, &source.repo);
    if repo_dir.exists() {
        let _ = std::fs::remove_dir_all(&repo_dir);
    }

    // Initialise the repository.
    let repo = git2::Repository::init(&repo_dir)
        .map_err(|e| anyhow!("failed to init repo at \"{}\": {}", repo_dir.display(), e))?;

    // Fetch the necessary references.
    let (refspecs, tags) = git_ref_to_refspecs(&source.reference);

    // Fetch the refspecs.
    let mut fetch_opts = git2::FetchOptions::new();
    if tags {
        fetch_opts.download_tags(git2::AutotagOption::All);
    }
    repo.remote_anonymous(source.repo.as_str())?
        .fetch(&refspecs, Some(&mut fetch_opts), None)
        .with_context(|| {
            format!(
                "failed to fetch `{}`. Check your connection or run in `--offline` mode",
                &source.repo
            )
        })?;

    // Call the user function.
    let output = f(repo)?;

    // Clean up the temporary directory.
    let _ = std::fs::remove_dir_all(&repo_dir);
    Ok(output)
}

/// Pin the given git-sourced package.
///
/// This clones the repository to a temporary directory in order to determine the commit at the
/// HEAD of the given git reference.
pub fn pin_git(fetch_id: u64, name: &str, source: SourceGit) -> Result<SourceGitPinned> {
    let commit_hash = with_tmp_git_repo(fetch_id, name, &source, |repo| {
        // Resolve the reference to the commit ID.
        let commit_id = source
            .reference
            .resolve(&repo)
            .with_context(|| "failed to resolve reference".to_string())?;
        Ok(format!("{}", commit_id))
    })?;
    Ok(SourceGitPinned {
        source,
        commit_hash,
    })
}

/// Given a package source, attempt to determine the pinned version or commit.
///
/// Also updates the `path_map` with a path to the local copy of the source.
///
/// The `path_root` is required for `Path` dependencies and must specify the package that is the
/// root of the current subgraph of path dependencies.
fn pin_pkg(
    fetch_id: u64,
    path_root: PinnedId,
    pkg: &Pkg,
    manifest_map: &mut ManifestMap,
    offline: bool,
) -> Result<Pinned> {
    let name = pkg.name.clone();
    let pinned = match &pkg.source {
        Source::Member(path) => {
            let source = SourcePinned::Member;
            let pinned = Pinned { name, source };
            let id = pinned.id();
            let manifest = PackageManifestFile::from_dir(path)?;
            manifest_map.insert(id, manifest);
            pinned
        }
        Source::Path(path) => {
            let path_pinned = SourcePathPinned { path_root };
            let source = SourcePinned::Path(path_pinned);
            let pinned = Pinned { name, source };
            let id = pinned.id();
            let manifest = PackageManifestFile::from_dir(path)?;
            manifest_map.insert(id, manifest);
            pinned
        }
        Source::Git(ref git_source) => {
            // If the git source directly specifies a full commit hash, we should check
            // to see if we have a local copy. Otherwise we cannot know what commit we should pin
            // to without fetching the repo into a temporary directory.
            let (pinned_git, repo_path) = if offline {
                let (local_path, commit_hash) = search_git_source_locally(&name, git_source)?
                    .ok_or_else(|| {
                        anyhow!(
                            "Unable to fetch pkg {:?} from  {:?} in offline mode",
                            name,
                            git_source.repo
                        )
                    })?;
                let pinned_git = SourceGitPinned {
                    source: git_source.clone(),
                    commit_hash,
                };
                (pinned_git, local_path)
            } else if let GitReference::DefaultBranch | GitReference::Branch(_) =
                git_source.reference
            {
                // If the reference is to a branch or to the default branch we need to fetch
                // from remote even though we may have it locally. Because remote may contain a
                // newer commit.
                let pinned_git = pin_git(fetch_id, &name, git_source.clone())?;
                let repo_path =
                    git_commit_path(&name, &pinned_git.source.repo, &pinned_git.commit_hash);
                (pinned_git, repo_path)
            } else {
                // If we are in online mode and the reference is to a specific commit (tag or
                // rev) we can first search it locally and re-use it.
                match search_git_source_locally(&name, git_source) {
                    Ok(Some((local_path, commit_hash))) => {
                        let pinned_git = SourceGitPinned {
                            source: git_source.clone(),
                            commit_hash,
                        };
                        (pinned_git, local_path)
                    }
                    _ => {
                        // If the checkout we are looking for does not exists locally or an
                        // error happened during the search fetch it
                        let pinned_git = pin_git(fetch_id, &name, git_source.clone())?;
                        let repo_path = git_commit_path(
                            &name,
                            &pinned_git.source.repo,
                            &pinned_git.commit_hash,
                        );
                        (pinned_git, repo_path)
                    }
                }
            };
            let source = SourcePinned::Git(pinned_git.clone());
            let pinned = Pinned { name, source };
            let id = pinned.id();
            if let hash_map::Entry::Vacant(entry) = manifest_map.entry(id) {
                // TODO: Here we assume that if the local path already exists, that it contains the
                // full and correct source for that commit and hasn't been tampered with. This is
                // probably fine for most cases as users should never be touching these
                // directories, however we should add some code to validate this. E.g. can we
                // recreate the git hash by hashing the directory or something along these lines
                // using git?
                if !repo_path.exists() {
                    info!("  Fetching {}", pinned_git.to_string());
                    fetch_git(fetch_id, &pinned.name, &pinned_git)?;
                }
                let path = find_dir_within(&repo_path, &pinned.name).ok_or_else(|| {
                    anyhow!(
                        "failed to find package `{}` in {}",
                        pinned.name,
                        pinned_git.to_string()
                    )
                })?;
                let manifest = PackageManifestFile::from_dir(&path)?;
                entry.insert(manifest);
            }
            pinned
        }
        Source::Registry(ref _source) => {
            if offline {
                bail!("Unable to fetch pkg {:?} in offline mode", name);
            }
            // TODO: determine registry pkg git URL, fetch to determine latest available
            // semver-compatible version
            bail!("registry dependencies are not yet supported");
        }
    };
    Ok(pinned)
}

/// The path to which a git package commit should be checked out.
///
/// The resulting directory is:
///
/// ```ignore
/// $HOME/.forc/git/checkouts/name-<repo_url_hash>/<commit_hash>
/// ```
///
/// where `<repo_url_hash>` is a hash of the source repository URL.
pub fn git_commit_path(name: &str, repo: &Url, commit_hash: &str) -> PathBuf {
    let repo_dir_name = git_repo_dir_name(name, repo);
    git_checkouts_directory()
        .join(repo_dir_name)
        .join(commit_hash)
}

/// Fetch the repo at the given git package's URL and checkout the pinned commit.
///
/// Returns the location of the checked out commit.
pub fn fetch_git(fetch_id: u64, name: &str, pinned: &SourceGitPinned) -> Result<PathBuf> {
    let path = git_commit_path(name, &pinned.source.repo, &pinned.commit_hash);
    // Checkout the pinned hash to the path.
    with_tmp_git_repo(fetch_id, name, &pinned.source, |repo| {
        // Change HEAD to point to the pinned commit.
        let id = git2::Oid::from_str(&pinned.commit_hash)?;
        repo.set_head_detached(id)?;
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path)?;

        // Checkout HEAD to the target directory.
        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.force().target_dir(&path);
        repo.checkout_head(Some(&mut checkout))?;

        // Fetch HEAD time and create an index
        let current_head = repo.revparse_single("HEAD")?;
        let head_commit = current_head
            .as_commit()
            .ok_or_else(|| anyhow!("Cannot get commit from {}", current_head.id().to_string()))?;
        let head_time = head_commit.time().seconds();
        let source_index = GitSourceIndex::new(
            head_time,
            pinned.source.reference.clone(),
            pinned.commit_hash.clone(),
        );
        // Write the index file
        fs::write(
            path.join(".forc_index"),
            serde_json::to_string(&source_index)?,
        )?;
        Ok(())
    })?;
    Ok(path)
}

/// Search local checkout dir for git sources, for non-branch git references tries to find the
/// exact match. For branch references, tries to find the most recent repo present locally with the given repo
fn search_git_source_locally(
    name: &str,
    git_source: &SourceGit,
) -> Result<Option<(PathBuf, String)>> {
    // In the checkouts dir iterate over dirs whose name starts with `name`
    let checkouts_dir = git_checkouts_directory();
    match &git_source.reference {
        GitReference::Branch(branch) => {
            // Collect repos from this branch with their HEAD time
            let repos_from_branch = collect_local_repos_with_branch(checkouts_dir, name, branch)?;
            // Get the newest repo by their HEAD commit times
            let newest_branch_repo = repos_from_branch
                .into_iter()
                .max_by_key(|&(_, (_, time))| time)
                .map(|(repo_path, (hash, _))| (repo_path, hash));
            Ok(newest_branch_repo)
        }
        _ => find_exact_local_repo_with_reference(checkouts_dir, name, &git_source.reference),
    }
}

/// Search and collect repos from checkouts_dir that are from given branch and for the given package
fn collect_local_repos_with_branch(
    checkouts_dir: PathBuf,
    package_name: &str,
    branch_name: &str,
) -> Result<Vec<(PathBuf, HeadWithTime)>> {
    let mut list_of_repos = Vec::new();
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Check if the repo's HEAD commit to verify it is from desired branch
        if let GitReference::Branch(branch) = repo_index.git_reference {
            if branch == branch_name {
                list_of_repos.push((repo_dir_path, repo_index.head_with_time));
            }
        }
        Ok(())
    })?;
    Ok(list_of_repos)
}

/// Search an exact reference in locally available repos
fn find_exact_local_repo_with_reference(
    checkouts_dir: PathBuf,
    package_name: &str,
    git_reference: &GitReference,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    if let GitReference::Tag(tag) = git_reference {
        found_local_repo = find_repo_with_tag(tag, package_name, checkouts_dir)?;
    } else if let GitReference::Rev(rev) = git_reference {
        found_local_repo = find_repo_with_rev(rev, package_name, checkouts_dir)?;
    }
    Ok(found_local_repo)
}

/// Search and find the match repo between the given tag and locally available options
fn find_repo_with_tag(
    tag: &str,
    package_name: &str,
    checkouts_dir: PathBuf,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Get current head of the repo
        let current_head = repo_index.head_with_time.0;
        if let GitReference::Tag(curr_repo_tag) = repo_index.git_reference {
            if curr_repo_tag == tag {
                found_local_repo = Some((repo_dir_path, current_head))
            }
        }
        Ok(())
    })?;
    Ok(found_local_repo)
}

/// Search and find the match repo between the given rev and locally available options
fn find_repo_with_rev(
    rev: &str,
    package_name: &str,
    checkouts_dir: PathBuf,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Get current head of the repo
        let current_head = repo_index.head_with_time.0;
        if let GitReference::Rev(curr_repo_rev) = repo_index.git_reference {
            if curr_repo_rev == rev {
                found_local_repo = Some((repo_dir_path, current_head));
            }
        }
        Ok(())
    })?;
    Ok(found_local_repo)
}

/// Search local checkouts directory and apply the given function. This is used for iterating over
/// possible options of a given package.
fn with_search_checkouts<F>(checkouts_dir: PathBuf, package_name: &str, mut f: F) -> Result<()>
where
    F: FnMut(GitSourceIndex, PathBuf) -> Result<()>,
{
    for entry in fs::read_dir(checkouts_dir)? {
        let entry = entry?;
        let folder_name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("invalid folder name"))?;
        if folder_name.starts_with(package_name) {
            // Search if the dir we are looking starts with the name of our package
            for repo_dir in fs::read_dir(entry.path())? {
                // Iterate over all dirs inside the `name-***` directory and try to open repo from
                // each dirs inside this one
                let repo_dir = repo_dir
                    .map_err(|e| anyhow!("Cannot find local repo at checkouts dir {}", e))?;
                if repo_dir.file_type()?.is_dir() {
                    // Get the path of the current repo
                    let repo_dir_path = repo_dir.path();
                    // Get the index file from the found path
                    if let Ok(index_file) = fs::read_to_string(repo_dir_path.join(".forc_index")) {
                        let index = serde_json::from_str(&index_file)?;
                        f(index, repo_dir_path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Given the path to a package and a `Dependency` parsed from one of its forc dependencies,
/// produce the `Source` for that dependendency.
fn dep_to_source(pkg_path: &Path, dep: &Dependency) -> Result<Source> {
    let source = match dep {
        Dependency::Simple(ref ver_str) => {
            bail!(
                "Unsupported dependency declaration in \"{}\": `{}` - \
                currently only `git` and `path` dependencies are supported",
                pkg_path.display(),
                ver_str
            )
        }
        Dependency::Detailed(ref det) => match (&det.path, &det.version, &det.git) {
            (Some(relative_path), _, _) => {
                let path = pkg_path.join(relative_path);
                let canonical_path = path.canonicalize().map_err(|e| {
                    anyhow!("Failed to canonicalize dependency path {:?}: {}", path, e)
                })?;
                // Check if path is a member of a workspace.
                let workspace_manifest = canonical_path
                    .parent()
                    .and_then(|parent_dir| WorkspaceManifestFile::from_dir(parent_dir).ok());

                match workspace_manifest {
                    Some(ws) if ws.is_member_path(&canonical_path)? => {
                        Source::Member(canonical_path)
                    }
                    _ => Source::Path(canonical_path),
                }
            }
            (_, _, Some(repo)) => {
                let reference = match (&det.branch, &det.tag, &det.rev) {
                    (Some(branch), None, None) => GitReference::Branch(branch.clone()),
                    (None, Some(tag), None) => GitReference::Tag(tag.clone()),
                    (None, None, Some(rev)) => GitReference::Rev(rev.clone()),
                    (None, None, None) => GitReference::DefaultBranch,
                    _ => bail!(
                        "git dependencies support at most one reference: \
                            either `branch`, `tag` or `rev`"
                    ),
                };
                let repo = Url::parse(repo)?;
                let source = SourceGit { repo, reference };
                Source::Git(source)
            }
            _ => {
                bail!("unsupported set of fields for dependency: {:?}", dep);
            }
        },
    };
    Ok(source)
}

/// If a patch exists for the given dependency source within the given project manifest, this
/// returns the patch.
fn dep_source_patch<'manifest>(
    manifest: &'manifest PackageManifestFile,
    dep_name: &str,
    dep_source: &Source,
) -> Option<&'manifest Dependency> {
    if let Source::Git(git) = dep_source {
        if let Some(patches) = manifest.patch(git.repo.as_str()) {
            if let Some(patch) = patches.get(dep_name) {
                return Some(patch);
            }
        }
    }
    None
}

/// If a patch exists for the given dependency within the given manifest, this returns a new
/// `Source` with the patch applied.
///
/// If no patch exists, this returns the original `Source`.
fn apply_patch(
    manifest: &PackageManifestFile,
    dep_name: &str,
    dep_source: &Source,
) -> Result<Source> {
    match dep_source_patch(manifest, dep_name, dep_source) {
        Some(patch) => dep_to_source(manifest.dir(), patch),
        None => Ok(dep_source.clone()),
    }
}

/// Converts the `Dependency` to a `Source` with any relevant patches in the given manifest
/// applied.
fn dep_to_source_patched(
    manifest: &PackageManifestFile,
    dep_name: &str,
    dep: &Dependency,
) -> Result<Source> {
    let unpatched = dep_to_source(manifest.dir(), dep)?;
    apply_patch(manifest, dep_name, &unpatched)
}

/// Given a `forc_pkg::BuildProfile`, produce the necessary `sway_core::BuildConfig` required for
/// compilation.
pub fn sway_build_config(
    manifest_dir: &Path,
    entry_path: &Path,
    build_profile: &BuildProfile,
) -> Result<sway_core::BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let file_name = find_file_name(manifest_dir, entry_path)?;
    let build_config = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.to_path_buf(),
    )
    .print_finalized_asm(build_profile.print_finalized_asm)
    .print_intermediate_asm(build_profile.print_intermediate_asm)
    .print_ir(build_profile.print_ir)
    .include_tests(build_profile.include_tests);
    Ok(build_config)
}

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
///
/// This function ensures that if `core` exists in the graph (the vastly common case) it is also
/// present within the namespace. This is a necessity for operators to work for example.
///
/// This function also ensures that if `std` exists in the graph,
/// then the std prelude will also be added.
pub fn dependency_namespace(
    lib_namespace_map: &HashMap<NodeIx, namespace::Module>,
    compiled_contract_deps: &HashMap<NodeIx, BuiltPackage>,
    graph: &Graph,
    node: NodeIx,
    constants: BTreeMap<String, ConfigTimeConstant>,
) -> Result<namespace::Module, vec1::Vec1<CompileError>> {
    let mut namespace = namespace::Module::default_with_constants(constants)?;

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
            DepKind::Contract => {
                let mut constants = BTreeMap::default();
                let compiled_dep = compiled_contract_deps.get(&dep_node);
                let dep_contract_id = match compiled_dep {
                    Some(dep_contract_compiled) => contract_id(dep_contract_compiled),
                    // On `check` we don't compile contracts, so we use a placeholder.
                    None => ContractId::default(),
                };

                // Construct namespace with contract id
                let contract_dep_constant_name = "CONTRACT_ID";
                let contract_id_value = format!("\"{dep_contract_id}\"");
                let contract_id_constant = ConfigTimeConstant {
                    r#type: "b256".to_string(),
                    value: contract_id_value,
                    public: true,
                };
                constants.insert(contract_dep_constant_name.to_string(), contract_id_constant);
                namespace::Module::default_with_constants(constants)?
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

    namespace.star_import_with_reexports(&[CORE, PRELUDE].map(Ident::new_no_span), &[]);

    if has_std_dep(graph, node) {
        namespace.star_import_with_reexports(&[STD, PRELUDE].map(Ident::new_no_span), &[]);
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
    manifest: &PackageManifestFile,
    build_profile: &BuildProfile,
    namespace: namespace::Module,
) -> Result<CompileResult<ty::TyProgram>> {
    let source = manifest.entry_string()?;
    let sway_build_config =
        sway_build_config(manifest.dir(), &manifest.entry_path(), build_profile)?;
    let ast_res = sway_core::compile_to_ast(source, namespace, Some(&sway_build_config));
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
    pkg: &Pinned,
    manifest: &PackageManifestFile,
    build_profile: &BuildProfile,
    namespace: namespace::Module,
    source_map: &mut SourceMap,
) -> Result<(BuiltPackage, namespace::Root)> {
    // Time the given expression and print the result if `build_config.time_phases` is true.
    macro_rules! time_expr {
        ($description:expr, $expression:expr) => {{
            if build_profile.time_phases {
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

    let entry_path = manifest.entry_path();
    let sway_build_config = time_expr!(
        "produce `sway_core::BuildConfig`",
        sway_build_config(manifest.dir(), &entry_path, build_profile,)?
    );
    let terse_mode = build_profile.terse;
    let fail = |warnings, errors| {
        print_on_failure(terse_mode, warnings, errors);
        bail!("Failed to compile {}", pkg.name);
    };

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = time_expr!(
        "compile to ast",
        compile_ast(manifest, build_profile, namespace,)?
    );
    let typed_program = match ast_res.value.as_ref() {
        None => return fail(&ast_res.warnings, &ast_res.errors),
        Some(typed_program) => typed_program,
    };

    if build_profile.print_ast {
        tracing::info!("{:#?}", typed_program);
    }

    let mut types = vec![];
    let json_abi_program = time_expr!(
        "generate JSON ABI program",
        typed_program.generate_json_abi_program(&mut types)
    );

    let storage_slots = typed_program.storage_slots.clone();
    let tree_type = typed_program.kind.tree_type();

    let namespace = typed_program.root.namespace.clone().into();

    if !ast_res.errors.is_empty() {
        return fail(&ast_res.warnings, &ast_res.errors);
    }

    let asm_res = time_expr!(
        "compile ast to asm",
        sway_core::ast_to_asm(ast_res, &sway_build_config)
    );
    let entries = asm_res
        .value
        .as_ref()
        .map(|asm| asm.0.entries.clone())
        .unwrap_or_default();
    let bc_res = time_expr!(
        "compile asm to bytecode",
        sway_core::asm_to_bytecode(asm_res, source_map)
    );

    match bc_res.value {
        Some(CompiledBytecode(bytes)) if bc_res.errors.is_empty() => {
            print_on_success(terse_mode, &pkg.name, &bc_res.warnings, &tree_type);
            let bytecode = bytes;
            let built_package = BuiltPackage {
                json_abi_program,
                storage_slots,
                bytecode,
                tree_type,
                entries,
            };
            Ok((built_package, namespace))
        }
        _ => fail(&bc_res.warnings, &bc_res.errors),
    }
}

/// The suffix that helps identify the file which contains the hash of the binary file created when
/// scripts are built_package.
pub const SWAY_BIN_HASH_SUFFIX: &str = "-bin-hash";

/// The suffix that helps identify the file which contains the root hash of the binary file created
/// when predicates are built_package.
pub const SWAY_BIN_ROOT_SUFFIX: &str = "-bin-root";

/// Returns the implied build profile by the build opts
fn build_profile_from_opts(
    build_profiles: &HashMap<String, BuildProfile>,
    build_options: &BuildOpts,
) -> Result<(String, BuildProfile)> {
    let key_debug = "debug".to_string();
    let key_release = "release".to_string();

    let BuildOpts {
        pkg,
        print,
        build_profile,
        release,
        time_phases,
        tests,
        ..
    } = build_options.to_owned();
    let mut selected_build_profile = key_debug;

    match &build_profile {
        Some(build_profile) => {
            if release {
                warn!(
                    "You specified both {} and 'release' profiles. Using the 'release' profile",
                    build_profile
                );
                selected_build_profile = key_release;
            } else {
                selected_build_profile = build_profile.clone();
            }
        }
        None => {
            if release {
                selected_build_profile = key_release;
            }
        }
    }

    // Retrieve the specified build profile
    let mut profile = build_profiles
        .get(&selected_build_profile)
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
    profile.print_ir |= print.ir;
    profile.print_finalized_asm |= print.finalized_asm;
    profile.print_intermediate_asm |= print.intermediate_asm;
    profile.terse |= pkg.terse;
    profile.time_phases |= time_phases;
    profile.include_tests |= tests;

    Ok((selected_build_profile, profile))
}

/// Builds a project with given BuildOptions
pub fn build_with_options(build_options: BuildOpts) -> Result<Built> {
    let path = &build_options.pkg.path;

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let manifest_file = ManifestFile::from_dir(&this_dir)?;
    let member_manifests = manifest_file.member_manifests()?;
    let lock_path = manifest_file.lock_path()?;
    let build_plan = BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        build_options.pkg.locked,
        build_options.pkg.offline,
    )?;
    let graph = build_plan.graph();
    let manifest_map = build_plan.manifest_map();
    let build_profiles: HashMap<String, BuildProfile> = build_plan.build_profiles().collect();
    // Get the selected build profile using build options
    let (profile_name, build_profile) = build_profile_from_opts(&build_profiles, &build_options)?;

    // If this is a workspace we want to have all members in the output.
    let outputs = match &manifest_file {
        ManifestFile::Package(pkg_manifest) => std::iter::once(
            build_plan
                .find_member_index(&pkg_manifest.project.name)
                .ok_or_else(|| anyhow!("Cannot found project node in the graph"))?,
        )
        .collect(),
        ManifestFile::Workspace(_) => build_plan.member_nodes().collect(),
    };
    // Build it!
    let built_packages_with_source_map = build(&build_plan, &build_profile, &outputs)?;
    let output_dir = build_options
        .clone()
        .pkg
        .output_directory
        .map(PathBuf::from);
    for (node_ix, (built_package, source_map)) in built_packages_with_source_map.iter() {
        let pinned = &graph[*node_ix];
        let pkg_manifest = manifest_map
            .get(&pinned.id())
            .ok_or_else(|| anyhow!("Couldn't find member manifest for {}", pinned.name))?;
        let output_dir = output_dir
            .clone()
            .unwrap_or_else(|| default_output_directory(pkg_manifest.dir()).join(&profile_name));
        built_package.output_artifacts(
            source_map,
            build_options.clone(),
            pkg_manifest,
            &output_dir,
        )?;
    }

    let built_packages: Vec<BuiltPackage> = built_packages_with_source_map
        .iter()
        .map(|(_, (built_package, _))| built_package)
        .cloned()
        .collect();

    match manifest_file {
        ManifestFile::Package(_) => {
            let built_pkg = built_packages
                .last()
                .ok_or_else(|| anyhow!("Couldn't find any built package"))?;
            Ok(Built::Package(Box::new(built_pkg.clone())))
        }
        ManifestFile::Workspace(_) => Ok(Built::Workspace(built_packages)),
    }
}

/// Returns the ContractId of a built_package contract with specified `salt`.
fn contract_id(built_package: &BuiltPackage) -> ContractId {
    // Construct the contract ID
    let contract = Contract::from(built_package.bytecode.clone());
    let salt = fuel_tx::Salt::new([0; 32]);
    let mut storage_slots = built_package.storage_slots.clone();
    storage_slots.sort();
    let state_root = Contract::initial_state_root(storage_slots.iter());
    contract.id(&salt, &contract.root(), &state_root)
}

/// Build an entire forc package and return the built_package output.
///
/// This compiles all packages (including dependencies) in the order specified by the `BuildPlan`.
///
/// Also returns the resulting `sway_core::SourceMap` which may be useful for debugging purposes.
pub fn build(
    plan: &BuildPlan,
    profile: &BuildProfile,
    outputs: &HashSet<NodeIx>,
) -> anyhow::Result<Vec<(NodeIx, (BuiltPackage, SourceMap))>> {
    //TODO remove once type engine isn't global anymore.
    sway_core::clear_lazy_statics();
    let mut built_packages = Vec::new();

    let required: HashSet<NodeIx> = outputs
        .iter()
        .flat_map(|output_node| plan.node_deps(*output_node))
        .collect();

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
        let constants = manifest.config_time_constants();
        let dep_namespace = match dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            &plan.graph,
            node,
            constants,
        ) {
            Ok(o) => o,
            Err(errs) => {
                print_on_failure(profile.terse, &[], &errs);
                bail!("Failed to compile {}", pkg.name);
            }
        };
        let res = compile(pkg, manifest, profile, dep_namespace, &mut source_map)?;
        let (mut built_package, namespace) = res;
        // If the current node is a contract dependency, collect the contract_id
        if plan
            .graph()
            .edges_directed(node, Direction::Incoming)
            .any(|e| e.weight().kind == DepKind::Contract)
        {
            compiled_contract_deps.insert(node, built_package.clone());
        }
        if let TreeType::Library { .. } = built_package.tree_type {
            lib_namespace_map.insert(node, namespace.into());
        }
        source_map.insert_dependency(manifest.dir());
        standardize_json_abi_types(&mut built_package.json_abi_program);
        built_packages.push((node, (built_package, source_map)));
    }

    Ok(built_packages)
}

/// Standardize the JSON ABI data structure by eliminating duplicate types. This is an iterative
/// process because every time two types are merged, new opportunities for more merging arise.
fn standardize_json_abi_types(json_abi_program: &mut JsonABIProgram) {
    loop {
        // If type with id_1 is a duplicate of type with id_2, then keep track of the mapping
        // between id_1 and id_2 in the HashMap below.
        let mut old_to_new_id: HashMap<usize, usize> = HashMap::new();

        // HashSet to eliminate duplicate type declarations.
        let mut types_set: HashSet<JsonTypeDeclaration> = HashSet::new();

        // Insert values in the HashSet `types_set` if they haven't been inserted before.
        // Otherwise, create an appropriate mapping in the HashMap `old_to_new_id`.
        for decl in json_abi_program.types.iter_mut() {
            if let Some(ty) = types_set.get(decl) {
                old_to_new_id.insert(decl.type_id, ty.type_id);
            } else {
                types_set.insert(decl.clone());
            }
        }

        // Nothing to do if the hash map is empty as there are not merge opportunities. We can now
        // exit the loop.
        if old_to_new_id.is_empty() {
            break;
        }

        // Convert the set into a vector and store it back in `json_abi_program.types`. We could
        // convert the HashSet *directly* into a vector using `collect()`, but the order would not
        // be deterministic. We could use `BTreeSet` instead of `HashSet` but the ordering in the
        // BTreeSet would have to depend on the original type ID and we're trying to avoid that.
        let mut filtered_types = vec![];
        for t in json_abi_program.types.iter() {
            if let Some(ty) = types_set.get(t) {
                if ty.type_id == t.type_id {
                    filtered_types.push((*ty).clone());
                    types_set.remove(t);
                }
            }
        }
        json_abi_program.types = filtered_types;

        // Update all `JsonTypeApplication`s and all `JsonTypeDeclaration`s
        update_all_types(json_abi_program, &old_to_new_id);
    }

    // Sort the `JsonTypeDeclaration`s
    json_abi_program
        .types
        .sort_by(|t1, t2| t1.type_field.cmp(&t2.type_field));

    // Standardize IDs (i.e. change them to 0,1,2,... according to the alphabetical order above
    let mut old_to_new_id: HashMap<usize, usize> = HashMap::new();
    for (ix, decl) in json_abi_program.types.iter_mut().enumerate() {
        old_to_new_id.insert(decl.type_id, ix);
        decl.type_id = ix;
    }

    // Update all `JsonTypeApplication`s and all `JsonTypeDeclaration`s
    update_all_types(json_abi_program, &old_to_new_id);
}

/// Recursively updates the type IDs used in a JsonABIProgram
fn update_all_types(json_abi_program: &mut JsonABIProgram, old_to_new_id: &HashMap<usize, usize>) {
    // Update all `JsonTypeApplication`s in every function
    for func in json_abi_program.functions.iter_mut() {
        for input in func.inputs.iter_mut() {
            update_json_type_application(input, old_to_new_id);
        }

        update_json_type_application(&mut func.output, old_to_new_id);
    }

    // Update all `JsonTypeDeclaration`
    for decl in json_abi_program.types.iter_mut() {
        update_json_type_declaration(decl, old_to_new_id);
    }

    for logged_type in json_abi_program.logged_types.iter_mut() {
        update_json_type_application(&mut logged_type.logged_type, old_to_new_id);
    }
}

/// Recursively updates the type IDs used in a `JsonTypeApplication` given a HashMap from old to
/// new IDs
fn update_json_type_application(
    type_application: &mut JsonTypeApplication,
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

/// Recursively updates the type IDs used in a `JsonTypeDeclaration` given a HashMap from old to
/// new IDs
fn update_json_type_declaration(
    type_declaration: &mut JsonTypeDeclaration,
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

/// A `CompileResult` thats type is a tuple containing a `ParseProgram` and `Option<ty::TyProgram>`
type ParseAndTypedPrograms = CompileResult<(ParseProgram, Option<ty::TyProgram>)>;

/// Compile the entire forc package and return the parse and typed programs
/// of the dependancies and project.
/// The final item in the returned vector is the project.
pub fn check(plan: &BuildPlan, terse_mode: bool) -> anyhow::Result<Vec<ParseAndTypedPrograms>> {
    //TODO remove once type engine isn't global anymore.
    sway_core::clear_lazy_statics();
    let mut lib_namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    // During `check`, we don't compile so this stays empty.
    let compiled_contract_deps = HashMap::new();

    let mut results = vec![];
    for &node in plan.compilation_order.iter() {
        let pkg = &plan.graph[node];
        let manifest = &plan.manifest_map()[&pkg.id()];
        let constants = manifest.config_time_constants();
        let dep_namespace = dependency_namespace(
            &lib_namespace_map,
            &compiled_contract_deps,
            &plan.graph,
            node,
            constants,
        )
        .expect("failed to create dependency namespace");
        let CompileResult {
            value,
            mut warnings,
            mut errors,
        } = parse(manifest, terse_mode)?;

        let parse_program = match value {
            None => {
                results.push(CompileResult::new(None, warnings, errors));
                return Ok(results);
            }
            Some(program) => program,
        };

        let ast_result = sway_core::parsed_to_ast(&parse_program, dep_namespace);
        warnings.extend(ast_result.warnings);
        errors.extend(ast_result.errors);

        let typed_program = match ast_result.value {
            None => {
                let value = Some((parse_program, None));
                results.push(CompileResult::new(value, warnings, errors));
                return Ok(results);
            }
            Some(typed_program) => typed_program,
        };

        if let TreeType::Library { .. } = typed_program.kind.tree_type() {
            lib_namespace_map.insert(node, typed_program.root.namespace.clone());
        }

        source_map.insert_dependency(manifest.dir());

        let value = Some((parse_program, Some(typed_program)));
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
    terse_mode: bool,
) -> anyhow::Result<CompileResult<ParseProgram>> {
    let profile = BuildProfile {
        terse: terse_mode,
        ..BuildProfile::debug()
    };
    let source = manifest.entry_string()?;
    let sway_build_config = sway_build_config(manifest.dir(), &manifest.entry_path(), &profile)?;
    Ok(sway_core::parse(source, Some(&sway_build_config)))
}

/// Attempt to find a `Forc.toml` with the given project name within the given directory.
///
/// Returns the path to the package on success, or `None` in the case it could not be found.
pub fn find_within(dir: &Path, pkg_name: &str) -> Option<PathBuf> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().ends_with(constants::MANIFEST_FILE_NAME))
        .find_map(|entry| {
            let path = entry.path();
            let manifest = PackageManifest::from_file(path).ok()?;
            if manifest.project.name == pkg_name {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
}

/// The same as [find_within], but returns the package's project directory.
pub fn find_dir_within(dir: &Path, pkg_name: &str) -> Option<PathBuf> {
    find_within(dir, pkg_name).and_then(|path| path.parent().map(Path::to_path_buf))
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

#[test]
fn test_source_git_pinned_parsing() {
    let strings = [
        "git+https://github.com/foo/bar?branch=baz#64092602dd6158f3e41d775ed889389440a2cd86",
        "git+https://github.com/fuellabs/sway-lib-std?tag=v0.1.0#0000000000000000000000000000000000000000",
        "git+https://github.com/fuellabs/sway-lib-core?tag=v0.0.1#0000000000000000000000000000000000000000",
        "git+https://some-git-host.com/owner/repo?rev#FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "git+https://some-git-host.com/owner/repo?default-branch#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ];

    let expected = [
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/foo/bar").unwrap(),
                reference: GitReference::Branch("baz".to_string()),
            },
            commit_hash: "64092602dd6158f3e41d775ed889389440a2cd86".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/fuellabs/sway-lib-std").unwrap(),
                reference: GitReference::Tag("v0.1.0".to_string()),
            },
            commit_hash: "0000000000000000000000000000000000000000".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/fuellabs/sway-lib-core").unwrap(),
                reference: GitReference::Tag("v0.0.1".to_string()),
            },
            commit_hash: "0000000000000000000000000000000000000000".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://some-git-host.com/owner/repo").unwrap(),
                reference: GitReference::Rev(
                    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string(),
                ),
            },
            commit_hash: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://some-git-host.com/owner/repo").unwrap(),
                reference: GitReference::DefaultBranch,
            },
            commit_hash: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        },
    ];

    for (&string, expected) in strings.iter().zip(&expected) {
        let parsed = SourceGitPinned::from_str(string).unwrap();
        assert_eq!(&parsed, expected);
        let serialized = expected.to_string();
        assert_eq!(&serialized, string);
    }
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
        .map(|e| format!("{}", e))
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("Parsing {} failed: \n{}", project_name, error);
    Error::msg(message)
}

/// Format an error message if an incorrect program type is present.
pub fn wrong_program_type(
    project_name: &str,
    expected_types: Vec<TreeType>,
    parse_type: TreeType,
) -> anyhow::Error {
    let message = format!(
        "{} is not a '{:?}' it is a '{:?}'",
        project_name, expected_types, parse_type
    );
    Error::msg(message)
}

/// Format an error message if a given URL fails to produce a working node.
pub fn fuel_core_not_running(node_url: &str) -> anyhow::Error {
    let message = format!("could not get a response from node at the URL {}. Start a node with `fuel-core`. See https://github.com/FuelLabs/fuel-core#running for more information", node_url);
    Error::msg(message)
}
