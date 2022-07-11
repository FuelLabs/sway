use crate::{
    lock::Lock,
    manifest::{BuildProfile, Dependency, Manifest, ManifestFile},
    CORE, STD,
};
use anyhow::{anyhow, bail, Context, Error, Result};
use forc_util::{
    find_file_name, git_checkouts_directory, kebab_to_snake_case, print_on_failure,
    print_on_success, print_on_success_library,
};
use fuel_tx::StorageSlot;
use petgraph::{
    self,
    visit::{Bfs, Dfs, EdgeRef, Walker},
    Directed, Direction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeSet, HashMap, HashSet},
    fmt, fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};
use sway_core::{
    semantic_analysis::namespace, source_map::SourceMap, types::*, BytecodeCompilationResult,
    CompileAstResult, CompileError, CompileResult, ParseProgram, TreeType,
};
use sway_types::JsonABI;
use sway_utils::constants;
use tracing::info;
use url::Url;

type GraphIx = u32;
type Node = Pinned;
type Edge = DependencyName;
pub type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;
pub type EdgeIx = petgraph::graph::EdgeIndex<GraphIx>;
pub type NodeIx = petgraph::graph::NodeIndex<GraphIx>;
pub type ManifestMap = HashMap<PinnedId, ManifestFile>;

/// A unique ID for a pinned package.
///
/// The internal value is produced by hashing the package's name and `SourcePinned`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PinnedId(u64);

/// The result of successfully compiling a package.
pub struct Compiled {
    pub json_abi: JsonABI,
    pub storage_slots: Vec<StorageSlot>,
    pub bytecode: Vec<u8>,
    pub tree_type: TreeType,
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
    Root(PathBuf),
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
    Root,
    Git(SourceGitPinned),
    Path(SourcePathPinned),
    Registry(SourceRegistryPinned),
}

/// Represents the full build plan for a project.
#[derive(Clone)]
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
    pub fn from_manifest(
        manifest: &ManifestFile,
        sway_git_tag: &str,
        offline: bool,
    ) -> Result<Self> {
        let mut graph = Graph::default();
        let mut manifest_map = ManifestMap::default();
        fetch_graph(
            manifest,
            offline,
            sway_git_tag,
            &mut graph,
            &mut manifest_map,
        )?;
        let compilation_order = compilation_order(&graph)?;
        Ok(Self {
            graph,
            manifest_map,
            compilation_order,
        })
    }

    /// Create a new build plan taking into account the state of both the Manifest and the existing
    /// lock file if there is one.
    ///
    /// This will first attempt to load a build plan from the lock file and validate the resulting
    /// graph using the current state of the Manifest.
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
    pub fn from_lock_and_manifest(
        manifest: &ManifestFile,
        locked: bool,
        offline: bool,
        sway_git_tag: &str,
    ) -> Result<Self> {
        // Keep track of the cause for the new lock file if it turns out we need one.
        let mut new_lock_cause = None;

        // First, attempt to load the lock.
        let lock_path = forc_util::lock_path(manifest.dir());
        let lock = Lock::from_path(&lock_path).unwrap_or_else(|e| {
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
        let invalid_deps = validate_graph(&graph, manifest, sway_git_tag);
        remove_deps(&mut graph, &manifest.project.name, &invalid_deps);

        // We know that the remaining nodes have valid paths, otherwise they would have been
        // removed. We can safely produce an initial `manifest_map`.
        let mut manifest_map = graph_to_manifest_map(manifest.clone(), &graph, sway_git_tag)?;

        // Attempt to fetch the remainder of the graph.
        let _added = fetch_graph(
            manifest,
            offline,
            sway_git_tag,
            &mut graph,
            &mut manifest_map,
        )?;

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
            crate::lock::print_diff(&manifest.project.name, &lock_diff);
            let string = toml::ser::to_string_pretty(&new_lock)
                .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
            fs::write(&lock_path, &string)
                .map_err(|e| anyhow!("failed to write lock file: {}", e))?;
            info!("   Created new lock file at {}", lock_path.display());
        }

        Ok(plan)
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
}

/// Given a graph and the known project name retrieved from the manifest, produce an iterator
/// yielding any nodes from the graph that might potentially be the project node.
fn potential_proj_nodes<'a>(g: &'a Graph, proj_name: &'a str) -> impl 'a + Iterator<Item = NodeIx> {
    g.node_indices()
        .filter(|&n| g.edges_directed(n, Direction::Incoming).next().is_none())
        .filter(move |&n| g[n].name == proj_name)
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

/// Validates the state of the pinned package graph against the given project manifest.
///
/// Returns the set of invalid dependency edges.
fn validate_graph(
    graph: &Graph,
    proj_manifest: &ManifestFile,
    sway_git_tag: &str,
) -> BTreeSet<EdgeIx> {
    // If we don't have a project node, remove everything as we can't validate dependencies
    // without knowing where to start.
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(node) => node,
        Err(_) => return graph.edge_indices().collect(),
    };
    // Collect all invalid dependency nodes.
    let mut visited = HashSet::new();
    validate_deps(graph, proj_node, proj_manifest, sway_git_tag, &mut visited)
}

/// Recursively validate all dependencies of the given `node`.
///
/// Returns the set of invalid dependency edges.
fn validate_deps(
    graph: &Graph,
    node: NodeIx,
    node_manifest: &ManifestFile,
    sway_git_tag: &str,
    visited: &mut HashSet<NodeIx>,
) -> BTreeSet<EdgeIx> {
    let mut remove = BTreeSet::default();
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_name = edge.weight();
        let dep_node = edge.target();
        match validate_dep(graph, node_manifest, dep_name, dep_node, sway_git_tag) {
            Err(_) => {
                remove.insert(edge.id());
            }
            Ok(dep_manifest) => {
                if visited.insert(dep_node) {
                    let rm = validate_deps(graph, dep_node, &dep_manifest, sway_git_tag, visited);
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
    node_manifest: &ManifestFile,
    dep_name: &str,
    dep_node: NodeIx,
    sway_git_tag: &str,
) -> Result<ManifestFile> {
    // Check the validity of the dependency path, including its path root.
    let dep_path =
        dep_path(graph, node_manifest, dep_name, dep_node, sway_git_tag).map_err(|e| {
            anyhow!(
                "failed to construct path for dependency {:?}: {}",
                dep_name,
                e
            )
        })?;

    // Ensure the manifest is accessible.
    let dep_manifest = ManifestFile::from_dir(&dep_path, sway_git_tag)?;

    // Check that the dependency's source matches the entry in the parent manifest.
    let dep_entry = node_manifest
        .dep(dep_name)
        .ok_or_else(|| anyhow!("no entry in parent manifest"))?;
    let dep_source = dep_to_source_patched(node_manifest, dep_name, dep_entry)?;
    let dep_pkg = graph[dep_node].unpinned(&dep_path);
    if dep_pkg.source != dep_source {
        bail!("dependency node's source does not match manifest entry");
    }

    validate_dep_manifest(&graph[dep_node], &dep_manifest)?;

    Ok(dep_manifest)
}

/// Part of dependency validation, any checks related to the depenency's manifest content.
fn validate_dep_manifest(dep: &Pinned, dep_manifest: &ManifestFile) -> Result<()> {
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
    Ok(())
}

/// Returns the canonical, local path to the given dependency node if it exists, `None` otherwise.
///
/// Also returns `None` in the case that the dependency is a `Path` dependency and the path root is
/// invalid.
fn dep_path(
    graph: &Graph,
    node_manifest: &ManifestFile,
    dep_name: &str,
    dep_node: NodeIx,
    sway_git_tag: &str,
) -> Result<PathBuf> {
    let dep = &graph[dep_node];
    match &dep.source {
        SourcePinned::Git(git) => {
            let repo_path = git_commit_path(&dep.name, &git.source.repo, &git.commit_hash);
            find_dir_within(&repo_path, &dep.name, sway_git_tag).ok_or_else(|| {
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
        SourcePinned::Root => unreachable!("a `Root` node cannot be a dependency"),
    }
}

/// Remove the given set of dependency edges from the `graph`.
///
/// Also removes all nodes that are no longer connected to the project node as a result.
fn remove_deps(graph: &mut Graph, proj_name: &str, edges_to_remove: &BTreeSet<EdgeIx>) {
    // Retrieve the project node.
    let proj_node = match find_proj_node(graph, proj_name) {
        Ok(node) => node,
        Err(_) => {
            // If it fails, invalidate everything.
            graph.clear();
            return;
        }
    };

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

    // Remove all nodes that are no longer connected to the project node as a result.
    // Skip iteration over the project node.
    let mut nodes = node_removal_order.into_iter();
    assert_eq!(nodes.next(), Some(proj_node));
    for node in nodes {
        if !has_parent(graph, node) {
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
            let refname = format!("refs/remotes/origin/tags/{}", tag);
            let id = repo.refname_to_id(&refname)?;
            let obj = repo.find_object(id, None)?;
            let obj = obj.peel(git2::ObjectType::Commit)?;
            Ok(obj.id())
        }

        // Resolve to the target for the given branch.
        fn resolve_branch(repo: &git2::Repository, branch: &str) -> Result<git2::Oid> {
            let name = format!("origin/{}", branch);
            let b = repo
                .find_branch(&name, git2::BranchType::Remote)
                .with_context(|| format!("failed to find branch `{}`", branch))?;
            b.get()
                .target()
                .ok_or_else(|| anyhow::format_err!("branch `{}` did not have a target", branch))
        }

        // Use the HEAD commit when default branch is specified.
        fn resolve_default_branch(repo: &git2::Repository) -> Result<git2::Oid> {
            let head_id = repo.refname_to_id("refs/remotes/origin/HEAD")?;
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
            SourcePinned::Root => Source::Root(path.to_owned()),
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
            SourcePinned::Root => write!(f, "root"),
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
        let source = if s == "root" {
            SourcePinned::Root
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

/// Given a graph of pinned packages and the project manifest, produce a map containing the
/// manifest of for every node in the graph.
///
/// Assumes the given `graph` only contains valid dependencies (see `validate_graph`).
fn graph_to_manifest_map(
    proj_manifest: ManifestFile,
    graph: &Graph,
    sway_git_tag: &str,
) -> Result<ManifestMap> {
    let mut manifest_map = ManifestMap::new();

    // Traverse the graph from the project node.
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(node) => node,
        Err(_) => return Ok(manifest_map),
    };
    let proj_id = graph[proj_node].id();
    manifest_map.insert(proj_id, proj_manifest);

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
                let dep_name = edge.weight();
                let parent = &graph[parent_node];
                let parent_manifest = manifest_map.get(&parent.id())?;
                Some((parent_manifest, dep_name))
            })
            .next()
            .ok_or_else(|| anyhow!("more than one root package detected in graph"))?;
        let dep_path =
            dep_path(graph, parent_manifest, dep_name, dep_node, sway_git_tag).map_err(|e| {
                anyhow!(
                    "failed to construct path for dependency {:?}: {}",
                    dep_name,
                    e
                )
            })?;
        let dep_manifest = ManifestFile::from_dir(&dep_path, sway_git_tag)?;
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
            SourcePinned::Git(_) | SourcePinned::Registry(_) | SourcePinned::Root => {
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
fn fetch_graph(
    proj_manifest: &ManifestFile,
    offline: bool,
    sway_git_tag: &str,
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
) -> Result<HashSet<NodeIx>> {
    // Retrieve the project node, or create one if it does not exist.
    let proj_node = match find_proj_node(graph, &proj_manifest.project.name) {
        Ok(proj_node) => proj_node,
        Err(_) => {
            let name = proj_manifest.project.name.clone();
            let source = SourcePinned::Root;
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
        sway_git_tag,
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
    sway_git_tag: &str,
    graph: &mut Graph,
    manifest_map: &mut ManifestMap,
    fetched: &mut HashMap<Pkg, NodeIx>,
    visited: &mut HashSet<NodeIx>,
) -> Result<HashSet<NodeIx>> {
    let mut added = HashSet::default();
    let parent_id = graph[node].id();
    let deps: Vec<_> = manifest_map[&parent_id]
        .deps()
        .map(|(n, d)| (n.clone(), d.clone()))
        .collect();
    for (dep_name, dep) in deps {
        let name = dep.package().unwrap_or(&dep_name).to_string();
        let source = dep_to_source_patched(&manifest_map[&parent_id], &name, &dep)
            .context("Failed to source dependency")?;

        // If we haven't yet fetched this dependency, fetch it, pin it and add it to the graph.
        let dep_pkg = Pkg { name, source };
        let dep_node = match fetched.entry(dep_pkg) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let dep_pinned = pin_pkg(
                    fetch_id,
                    path_root,
                    entry.key(),
                    manifest_map,
                    offline,
                    sway_git_tag,
                )?;
                let dep_node = graph.add_node(dep_pinned);
                added.insert(dep_node);
                *entry.insert(dep_node)
            }
        };

        // Ensure we have an edge to the dependency.
        graph.update_edge(node, dep_node, dep_name.to_string());

        // If we've visited this node during this traversal already, no need to traverse it again.
        if !visited.insert(dep_node) {
            continue;
        }

        let dep_pinned = &graph[dep_node];
        let dep_pkg_id = dep_pinned.id();
        validate_dep_manifest(dep_pinned, &manifest_map[&dep_pkg_id]).map_err(|e| {
            let parent = &graph[node];
            anyhow!(
                "dependency of {:?} named {:?} is invalid: {}",
                parent.name,
                dep_name,
                e
            )
        })?;

        let path_root = match dep_pinned.source {
            SourcePinned::Root | SourcePinned::Git(_) | SourcePinned::Registry(_) => dep_pkg_id,
            SourcePinned::Path(_) => path_root,
        };

        // Fetch the children.
        added.extend(fetch_deps(
            fetch_id,
            offline,
            dep_node,
            path_root,
            sway_git_tag,
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
            refspecs.push(format!("+refs/heads/{0}:refs/remotes/origin/{0}", s));
        }
        GitReference::Tag(s) => {
            refspecs.push(format!("+refs/tags/{0}:refs/remotes/origin/tags/{0}", s));
        }
        GitReference::Rev(s) => {
            if s.starts_with("refs/") {
                refspecs.push(format!("+{0}:{0}", s));
            } else {
                // We can't fetch the commit directly, so we fetch all branches and tags in order
                // to find it.
                refspecs.push("+refs/heads/*:refs/remotes/origin/*".to_string());
                refspecs.push("+HEAD:refs/remotes/origin/HEAD".to_string());
                tags = true;
            }
        }
        GitReference::DefaultBranch => {
            refspecs.push("+HEAD:refs/remotes/origin/HEAD".to_string());
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
        .with_context(|| format!("failed to fetch `{}`", &source.repo))?;

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
    sway_git_tag: &str,
) -> Result<Pinned> {
    let name = pkg.name.clone();
    let pinned = match &pkg.source {
        Source::Root(path) => {
            let source = SourcePinned::Root;
            let pinned = Pinned { name, source };
            let id = pinned.id();
            let manifest = ManifestFile::from_dir(path, sway_git_tag)?;
            manifest_map.insert(id, manifest);
            pinned
        }
        Source::Path(path) => {
            let path_pinned = SourcePathPinned { path_root };
            let source = SourcePinned::Path(path_pinned);
            let pinned = Pinned { name, source };
            let id = pinned.id();
            let manifest = ManifestFile::from_dir(path, sway_git_tag)?;
            manifest_map.insert(id, manifest);
            pinned
        }
        Source::Git(ref git_source) => {
            // TODO: If the git source directly specifies a full commit hash, we should first check
            // to see if we have a local copy. Otherwise we cannot know what commit we should pin
            // to without fetching the repo into a temporary directory.
            if offline {
                bail!(
                    "Unable to fetch pkg {:?} from {:?} in offline mode",
                    name,
                    git_source.repo
                );
            }
            let pinned_git = pin_git(fetch_id, &name, git_source.clone())?;
            let repo_path =
                git_commit_path(&name, &pinned_git.source.repo, &pinned_git.commit_hash);
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
                let path =
                    find_dir_within(&repo_path, &pinned.name, sway_git_tag).ok_or_else(|| {
                        anyhow!(
                            "failed to find package `{}` in {}",
                            pinned.name,
                            pinned_git.to_string()
                        )
                    })?;
                let manifest = ManifestFile::from_dir(&path, sway_git_tag)?;
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
        Ok(())
    })?;

    Ok(path)
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
                Source::Path(canonical_path)
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
    manifest: &'manifest ManifestFile,
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
fn apply_patch(manifest: &ManifestFile, dep_name: &str, dep_source: &Source) -> Result<Source> {
    match dep_source_patch(manifest, dep_name, dep_source) {
        Some(patch) => dep_to_source(manifest.dir(), patch),
        None => Ok(dep_source.clone()),
    }
}

/// Converts the `Dependency` to a `Source` with any relevant patches in the given manifest
/// applied.
fn dep_to_source_patched(
    manifest: &ManifestFile,
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
    .print_ir(build_profile.print_ir);
    Ok(build_config)
}

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
///
/// This function ensures that if `core` exists in the graph (the vastly common case) it is also
/// present within the namespace. This is a necessity for operators to work for example.
pub fn dependency_namespace(
    namespace_map: &HashMap<NodeIx, namespace::Module>,
    graph: &Graph,
    node: NodeIx,
) -> namespace::Module {
    let mut namespace = namespace::Module::default();

    // Add direct dependencies.
    let mut core_added = false;
    for edge in graph.edges_directed(node, Direction::Outgoing) {
        let dep_node = edge.target();
        let dep_namespace = &namespace_map[&dep_node];
        let dep_name = kebab_to_snake_case(edge.weight());
        namespace.insert_submodule(dep_name, dep_namespace.clone());
        let dep = &graph[dep_node];
        if dep.name == CORE {
            core_added = true;
        }
    }

    // Add `core` if not already added.
    if !core_added {
        if let Some(core_node) = find_core_dep(graph, node) {
            let core_namespace = &namespace_map[&core_node];
            namespace.insert_submodule(CORE.to_string(), core_namespace.clone());
        }
    }

    namespace
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
            _ => (),
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
    manifest: &ManifestFile,
    build_profile: &BuildProfile,
    namespace: namespace::Module,
) -> Result<CompileAstResult> {
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
    manifest: &ManifestFile,
    build_profile: &BuildProfile,
    namespace: namespace::Module,
    source_map: &mut SourceMap,
) -> Result<(Compiled, Option<namespace::Root>)> {
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
        sway_build_config(manifest.dir(), &entry_path, build_profile)?
    );
    let silent_mode = build_profile.silent;

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = time_expr!(
        "compile to ast",
        compile_ast(manifest, build_profile, namespace)?
    );
    match &ast_res {
        CompileAstResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            bail!("Failed to compile {}", pkg.name);
        }
        CompileAstResult::Success {
            typed_program,
            warnings,
        } => {
            let json_abi = time_expr!("generate JSON ABI", typed_program.kind.generate_json_abi());
            let storage_slots = typed_program.storage_slots.clone();
            let tree_type = typed_program.kind.tree_type();
            match tree_type {
                // If we're compiling a library, we don't need to compile any further.
                // Instead, we update the namespace with the library's top-level module.
                TreeType::Library { .. } => {
                    print_on_success_library(silent_mode, &pkg.name, warnings);
                    let bytecode = vec![];
                    let lib_namespace = typed_program.root.namespace.clone();
                    let compiled = Compiled {
                        json_abi,
                        storage_slots,
                        bytecode,
                        tree_type,
                    };
                    Ok((compiled, Some(lib_namespace.into())))
                }

                // For all other program types, we'll compile the bytecode.
                TreeType::Contract | TreeType::Predicate | TreeType::Script => {
                    let asm_res = time_expr!(
                        "compile ast to asm",
                        sway_core::ast_to_asm(ast_res, &sway_build_config)
                    );
                    let bc_res = time_expr!(
                        "compile asm to bytecode",
                        sway_core::asm_to_bytecode(asm_res, source_map)
                    );
                    sway_core::clear_lazy_statics();
                    match bc_res {
                        BytecodeCompilationResult::Success { bytes, warnings } => {
                            print_on_success(silent_mode, &pkg.name, &warnings, &tree_type);
                            let bytecode = bytes;
                            let compiled = Compiled {
                                json_abi,
                                storage_slots,
                                bytecode,
                                tree_type,
                            };
                            Ok((compiled, None))
                        }
                        BytecodeCompilationResult::Library { .. } => {
                            unreachable!("compilation of library program types is handled above")
                        }
                        BytecodeCompilationResult::Failure { errors, warnings } => {
                            print_on_failure(silent_mode, &warnings, &errors);
                            bail!("Failed to compile {}", pkg.name);
                        }
                    }
                }
            }
        }
    }
}

/// Build an entire forc package and return the compiled output.
///
/// This compiles all packages (including dependencies) in the order specified by the `BuildPlan`.
///
/// Also returns the resulting `sway_core::SourceMap` which may be useful for debugging purposes.
pub fn build(plan: &BuildPlan, profile: &BuildProfile) -> anyhow::Result<(Compiled, SourceMap)> {
    let mut namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut storage_slots = vec![];
    let mut bytecode = vec![];
    let mut tree_type = None;
    for &node in &plan.compilation_order {
        let dep_namespace = dependency_namespace(&namespace_map, &plan.graph, node);
        let pkg = &plan.graph()[node];
        let manifest = &plan.manifest_map()[&pkg.id()];
        let res = compile(pkg, manifest, profile, dep_namespace, &mut source_map)?;
        let (compiled, maybe_namespace) = res;
        if let Some(namespace) = maybe_namespace {
            namespace_map.insert(node, namespace.into());
        }
        json_abi.extend(compiled.json_abi);
        storage_slots.extend(compiled.storage_slots);
        bytecode = compiled.bytecode;
        tree_type = Some(compiled.tree_type);
        source_map.insert_dependency(manifest.dir());
    }
    let tree_type =
        tree_type.ok_or_else(|| anyhow!("build plan must contain at least one package"))?;
    let compiled = Compiled {
        bytecode,
        json_abi,
        storage_slots,
        tree_type,
    };
    Ok((compiled, source_map))
}

/// Compile the entire forc package and return a CompileAstResult.
pub fn check(
    plan: &BuildPlan,
    silent_mode: bool,
) -> anyhow::Result<(CompileResult<ParseProgram>, CompileAstResult)> {
    let mut namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    for (i, &node) in plan.compilation_order.iter().enumerate() {
        let dep_namespace = dependency_namespace(&namespace_map, &plan.graph, node);
        let pkg = &plan.graph[node];
        let manifest = &plan.manifest_map()[&pkg.id()];
        let parsed_result = parse(manifest, silent_mode)?;

        let parse_program = match &parsed_result.value {
            None => bail!("unable to parse"),
            Some(program) => program,
        };

        let ast_result = sway_core::parsed_to_ast(parse_program, dep_namespace);

        let typed_program = match &ast_result {
            CompileAstResult::Failure { .. } => bail!("unable to type check"),
            CompileAstResult::Success { typed_program, .. } => typed_program,
        };

        if let TreeType::Library { .. } = typed_program.kind.tree_type() {
            namespace_map.insert(node, typed_program.root.namespace.clone());
        }

        source_map.insert_dependency(manifest.dir());

        // We only need to return the final CompileAstResult
        if i == plan.compilation_order.len() - 1 {
            return Ok((parsed_result, ast_result));
        }
    }
    bail!("unable to check sway program: build plan contains no packages")
}

/// Returns a parsed AST from the supplied [ManifestFile]
pub fn parse(
    manifest: &ManifestFile,
    silent_mode: bool,
) -> anyhow::Result<CompileResult<ParseProgram>> {
    let profile = BuildProfile {
        silent: silent_mode,
        ..BuildProfile::debug()
    };
    let source = manifest.entry_string()?;
    let sway_build_config = sway_build_config(manifest.dir(), &manifest.entry_path(), &profile)?;
    Ok(sway_core::parse(source, Some(&sway_build_config)))
}

/// Attempt to find a `Forc.toml` with the given project name within the given directory.
///
/// Returns the path to the package on success, or `None` in the case it could not be found.
pub fn find_within(dir: &Path, pkg_name: &str, sway_git_tag: &str) -> Option<PathBuf> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().ends_with(constants::MANIFEST_FILE_NAME))
        .find_map(|entry| {
            let path = entry.path();
            let manifest = Manifest::from_file(path, sway_git_tag).ok()?;
            if manifest.project.name == pkg_name {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
}

/// The same as [find_within], but returns the package's project directory.
pub fn find_dir_within(dir: &Path, pkg_name: &str, sway_git_tag: &str) -> Option<PathBuf> {
    find_within(dir, pkg_name, sway_git_tag).and_then(|path| path.parent().map(Path::to_path_buf))
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
