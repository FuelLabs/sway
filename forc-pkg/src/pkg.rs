use crate::{
    lock::Lock,
    manifest::{Dependency, Manifest, ManifestFile},
};
use anyhow::{anyhow, bail, Context, Error, Result};
use forc_util::{
    find_file_name, git_checkouts_directory, kebab_to_snake_case, print_on_failure,
    print_on_success, print_on_success_library, println_yellow_err,
};
use petgraph::{self, visit::EdgeRef, Directed, Direction};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};
use sway_core::{
    source_map::SourceMap, BytecodeCompilationResult, CompileAstResult, CompileError, NamespaceRef,
    NamespaceWrapper, TreeType, TypedParseTree,
};
use sway_types::JsonABI;
use sway_utils::constants;
use url::Url;

type GraphIx = u32;
type Node = Pinned;
type Edge = DependencyName;
pub type Graph = petgraph::Graph<Node, Edge, Directed, GraphIx>;
pub type NodeIx = petgraph::graph::NodeIndex<GraphIx>;
pub type PathMap = HashMap<PinnedId, PathBuf>;

/// A unique ID for a pinned package.
///
/// The internal value is produced by hashing the package's name and `SourcePinned`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct PinnedId(u64);

/// The result of successfully compiling a package.
pub struct Compiled {
    pub json_abi: JsonABI,
    pub bytecode: Vec<u8>,
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
    Git(SourceGitPinned),
    Path,
    Registry(SourceRegistryPinned),
}

/// Represents the full build plan for a project.
#[derive(Clone)]
pub struct BuildPlan {
    graph: Graph,
    path_map: PathMap,
    compilation_order: Vec<NodeIx>,
}

/// Parameters to pass through to the `sway_core::BuildConfig` during compilation.
pub struct BuildConfig {
    pub use_orig_asm: bool,
    pub use_orig_parser: bool,
    pub print_ir: bool,
    pub print_finalized_asm: bool,
    pub print_intermediate_asm: bool,
    pub silent: bool,
}

/// Error returned upon failed parsing of `SourceGitPinned::from_str`.
#[derive(Clone, Debug)]
pub enum SourceGitPinnedParseError {
    Prefix,
    Url,
    Reference,
    CommitHash,
}

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
    /// Create a new build plan for the project by fetching and pinning dependenies.
    pub fn new(manifest: &ManifestFile, sway_git_tag: &str, offline: bool) -> Result<Self> {
        let path = manifest.dir().to_path_buf();
        let (graph, path_map) = fetch_deps(path, manifest, sway_git_tag, offline)?;
        let compilation_order = compilation_order(&graph)?;
        Ok(Self {
            graph,
            path_map,
            compilation_order,
        })
    }

    /// Attempt to load the build plan from the `Lock`.
    pub fn from_lock(proj_path: &Path, lock: &Lock, sway_git_tag: &str) -> Result<Self> {
        let graph = lock.to_graph()?;
        let compilation_order = compilation_order(&graph)?;
        let path_map = graph_to_path_map(proj_path, &graph, &compilation_order, sway_git_tag)?;
        Ok(Self {
            graph,
            path_map,
            compilation_order,
        })
    }

    /// Attempt to load the build plan from the `Forc.lock` file.
    pub fn from_lock_file(lock_path: &Path, sway_git_tag: &str) -> Result<Self> {
        let proj_path = lock_path.parent().unwrap();
        let lock = Lock::from_path(lock_path)?;
        Self::from_lock(proj_path, &lock, sway_git_tag)
    }

    /// Ensure that the build plan is valid for the given manifest.
    pub fn validate(&self, manifest: &Manifest, sway_git_tag: &str) -> Result<()> {
        // Retrieve project's graph node.
        let proj_node = *self
            .compilation_order
            .last()
            .ok_or_else(|| anyhow!("Invalid Graph"))?;

        // Collect dependency `Source`s from graph.
        let plan_dep_pkgs: BTreeSet<_> = self
            .graph
            .edges_directed(proj_node, Direction::Outgoing)
            .map(|e| {
                let dep_name = e.weight();
                let dep_pkg = self.graph[e.target()].unpinned(&self.path_map);
                (dep_name, dep_pkg)
            })
            .collect();

        // Collect dependency `Source`s from manifest.
        let proj_id = self.graph[proj_node].id();
        let proj_path = &self.path_map[&proj_id];
        let manifest_dep_pkgs = manifest
            .deps()
            .map(|(dep_name, dep)| {
                // NOTE: Temporarily warn about `version` until we have support for registries.
                if let Dependency::Detailed(det) = dep {
                    if det.version.is_some() {
                        println_yellow_err(&format!(
                            "  WARNING! Dependency \"{}\" specifies the unused `version` field: \
                            consider using `branch` or `tag` instead",
                            dep_name
                        ));
                    }
                }

                let name = dep.package().unwrap_or(dep_name).to_string();
                let source = dep_to_source(proj_path, dep)?;
                let dep_pkg = Pkg { name, source };
                Ok((dep_name, dep_pkg))
            })
            .collect::<Result<BTreeSet<_>>>()?;

        // Ensure both `pkg::Source` are equal. If not, error.
        if plan_dep_pkgs != manifest_dep_pkgs {
            bail!("Manifest dependencies do not match");
        }

        // Ensure the pkg names of all nodes match their associated manifests.
        for node in self.graph.node_indices() {
            let pkg = &self.graph[node];
            let id = pkg.id();
            let path = &self.path_map[&id];
            let manifest = ManifestFile::from_dir(path, sway_git_tag)?;
            if pkg.name != manifest.project.name {
                bail!(
                    "package name {:?} does not match the associated manifest project name {:?}",
                    pkg.name,
                    manifest.project.name,
                );
            }
        }

        Ok(())
    }

    /// View the build plan's compilation graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// View the build plan's map of pinned package IDs to the path containing a local copy of
    /// their source.
    pub fn path_map(&self) -> &PathMap {
        &self.path_map
    }

    /// The order in which nodes are compiled, determined via a toposort of the package graph.
    pub fn compilation_order(&self) -> &[NodeIx] {
        &self.compilation_order
    }
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
    pub fn unpinned(&self, path_map: &PathMap) -> Pkg {
        let id = self.id();
        let source = match &self.source {
            SourcePinned::Git(git) => Source::Git(git.source.clone()),
            SourcePinned::Path => Source::Path(path_map[&id].to_path_buf()),
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

impl ToString for SourceGitPinned {
    fn to_string(&self) -> String {
        // git+<url/to/repo>?<ref_kind>=<ref_string>#<commit>
        let reference = self.source.reference.to_string();
        format!(
            "git+{}?{}#{}",
            self.source.repo, reference, self.commit_hash
        )
    }
}

impl ToString for GitReference {
    fn to_string(&self) -> String {
        match self {
            GitReference::Branch(ref s) => format!("branch={}", s),
            GitReference::Tag(ref s) => format!("tag={}", s),
            GitReference::Rev(ref _s) => "rev".to_string(),
            GitReference::DefaultBranch => "default-branch".to_string(),
        }
    }
}

impl FromStr for SourceGitPinned {
    type Err = SourceGitPinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // git+<url/to/repo>?<reference>#<commit>
        let s = s.trim();

        // Check for "git+" at the start.
        const PREFIX: &str = "git+";
        if s.find(PREFIX) != Some(0) {
            return Err(SourceGitPinnedParseError::Prefix);
        }
        let s = &s[PREFIX.len()..];

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
    petgraph::algo::toposort(rev_pkg_graph, None)
        // TODO: Show full list of packages that cycle.
        .map_err(|e| anyhow!("dependency cycle detected: {:?}", e))
}

/// Given graph of pinned dependencies and the directory for the root node, produce a path map
/// containing the path to the local source for every node in the graph.
pub fn graph_to_path_map(
    proj_manifest_dir: &Path,
    graph: &Graph,
    compilation_order: &[NodeIx],
    sway_git_tag: &str,
) -> Result<PathMap> {
    let mut path_map = PathMap::new();

    // We resolve all paths in reverse compilation order.
    // That is, we follow paths starting from the project root.
    let mut path_resolve_order = compilation_order.iter().cloned().rev();

    // Add the project's package to the map.
    let proj_node = path_resolve_order
        .next()
        .ok_or_else(|| anyhow!("graph must contain at least the project node"))?;
    let proj_id = graph[proj_node].id();
    path_map.insert(proj_id, proj_manifest_dir.to_path_buf());

    // Produce the unique `fetch_id` in case we need to fetch a missing git dep.
    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(&path_map[&proj_id], fetch_ts);

    // Resolve all following dependencies, knowing their parents' paths will already be resolved.
    for dep_node in path_resolve_order {
        let dep = &graph[dep_node];
        let dep_path = match &dep.source {
            SourcePinned::Git(git) => {
                let repo_path = git_commit_path(&dep.name, &git.source.repo, &git.commit_hash);
                if !repo_path.exists() {
                    println!("  Fetching {}", git.to_string());
                    fetch_git(fetch_id, &dep.name, git)?;
                }
                find_dir_within(&repo_path, &dep.name, sway_git_tag).ok_or_else(|| {
                    anyhow!(
                        "failed to find package `{}` in {}",
                        dep.name,
                        git.to_string()
                    )
                })?
            }
            SourcePinned::Path => {
                let (parent_node, dep_name) = graph
                    .edges_directed(dep_node, Direction::Incoming)
                    .next()
                    .map(|edge| (edge.source(), edge.weight().clone()))
                    .ok_or_else(|| anyhow!("more than one root package detected in graph"))?;
                let parent = &graph[parent_node];
                let parent_path = &path_map[&parent.id()];
                let parent_manifest = ManifestFile::from_dir(parent_path, sway_git_tag)?;
                let detailed = parent_manifest
                    .dependencies
                    .as_ref()
                    .and_then(|deps| deps.get(&dep_name))
                    .ok_or_else(|| {
                        anyhow!(
                            "dependency required for path reconstruction \
                            has been removed from the manifest"
                        )
                    })
                    .and_then(|dep| match dep {
                        Dependency::Detailed(detailed) => Ok(detailed),
                        Dependency::Simple(_) => {
                            bail!("missing path info for dependency: {}", &dep_name);
                        }
                    })?;
                let rel_dep_path = detailed
                    .path
                    .as_ref()
                    .ok_or_else(|| anyhow!("missing path info for dependency: {}", dep.name))?;
                let path = parent_path.join(rel_dep_path);
                if !path.exists() {
                    bail!("pinned `path` dependency \"{}\" source missing", dep.name);
                }
                path
            }
            SourcePinned::Registry(_reg) => {
                bail!("registry dependencies are not yet supported");
            }
        };
        path_map.insert(dep.id(), dep_path);
    }

    Ok(path_map)
}

/// Fetch all depedencies and produce the dependency graph along with a map from each node's unique
/// ID to its local fetched path.
///
/// This will determine pinned versions and commits for remote dependencies during traversal.
pub(crate) fn fetch_deps(
    proj_manifest_dir: PathBuf,
    proj_manifest: &Manifest,
    sway_git_tag: &str,
    offline_mode: bool,
) -> Result<(Graph, PathMap)> {
    let mut graph = Graph::new();
    let mut path_map = PathMap::new();

    // Add the project to the graph as the root node.
    let name = proj_manifest.project.name.clone();
    let path = proj_manifest_dir;
    let source = SourcePinned::Path;
    let pkg = Pinned { name, source };
    let pkg_id = pkg.id();
    path_map.insert(pkg_id, path);
    let root = graph.add_node(pkg);

    // The set of visited packages, starting with the root.
    let mut visited = HashMap::new();
    visited.insert(graph[root].clone(), root);

    // Recursively fetch children and add them to the graph.
    // TODO: Convert this recursion to use loop & stack to ensure deps can't cause stack overflow.
    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(&path_map[&pkg_id], fetch_ts);
    let manifest = Manifest::from_dir(&path_map[&pkg_id], sway_git_tag)?;
    fetch_children(
        fetch_id,
        offline_mode,
        root,
        &manifest,
        sway_git_tag,
        &mut graph,
        &mut path_map,
        &mut visited,
    )?;

    Ok((graph, path_map))
}

/// Produce a unique ID for a particular fetch pass.
///
/// This is used in the temporary git directory and allows for avoiding contention over the git repo directory.
fn fetch_id(path: &Path, timestamp: std::time::Instant) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    hasher.finish()
}

/// Fetch children nodes of the given node and add unvisited nodes to the graph.
#[allow(clippy::too_many_arguments)]
fn fetch_children(
    fetch_id: u64,
    offline_mode: bool,
    node: NodeIx,
    manifest: &Manifest,
    sway_git_tag: &str,
    graph: &mut Graph,
    path_map: &mut PathMap,
    visited: &mut HashMap<Pinned, NodeIx>,
) -> Result<()> {
    let parent = &graph[node];
    let parent_path = path_map[&parent.id()].clone();
    for (dep_name, dep) in manifest.deps() {
        let name = dep.package().unwrap_or(dep_name).to_string();
        let source = dep_to_source(&parent_path, dep)?;
        if offline_mode && !matches!(source, Source::Path(_)) {
            bail!("Unable to fetch pkg {:?} in offline mode", source);
        }
        let pkg = Pkg { name, source };
        let pinned = pin_pkg(fetch_id, &pkg, path_map, sway_git_tag)?;
        let pkg_id = pinned.id();
        let manifest = Manifest::from_dir(&path_map[&pkg_id], sway_git_tag)?;
        if pinned.name != manifest.project.name {
            bail!(
                "dependency name {:?} must match the manifest project name {:?} \
                unless `package = {:?}` is specified in the dependency declaration",
                pinned.name,
                manifest.project.name,
                manifest.project.name,
            );
        }
        let dep_node = if let hash_map::Entry::Vacant(entry) = visited.entry(pinned.clone()) {
            let node = graph.add_node(pinned);
            entry.insert(node);
            fetch_children(
                fetch_id,
                offline_mode,
                node,
                &manifest,
                sway_git_tag,
                graph,
                path_map,
                visited,
            )?;
            node
        } else {
            visited[&pinned]
        };
        graph.add_edge(node, dep_node, dep_name.to_string());
    }
    Ok(())
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
fn pin_git(fetch_id: u64, name: &str, source: SourceGit) -> Result<SourceGitPinned> {
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
fn pin_pkg(fetch_id: u64, pkg: &Pkg, path_map: &mut PathMap, sway_git_tag: &str) -> Result<Pinned> {
    let name = pkg.name.clone();
    let pinned = match &pkg.source {
        Source::Path(path) => {
            let source = SourcePinned::Path;
            let pinned = Pinned { name, source };
            let id = pinned.id();
            path_map.insert(id, path.clone());
            pinned
        }
        Source::Git(ref git_source) => {
            let pinned_git = pin_git(fetch_id, &name, git_source.clone())?;
            let repo_path =
                git_commit_path(&name, &pinned_git.source.repo, &pinned_git.commit_hash);
            let source = SourcePinned::Git(pinned_git.clone());
            let pinned = Pinned { name, source };
            let id = pinned.id();
            if let hash_map::Entry::Vacant(entry) = path_map.entry(id) {
                // TODO: Here we assume that if the local path already exists, that it contains the full and
                // correct source for that commit and hasn't been tampered with. This is probably fine for most
                // cases as users should never be touching these directories, however we should add some code
                // to validate this. E.g. can we recreate the git hash by hashing the directory or something
                // along these lines using git?
                if !repo_path.exists() {
                    println!("  Fetching {}", pinned_git.to_string());
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
                entry.insert(path);
            }
            pinned
        }
        Source::Registry(ref _source) => {
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
fn git_commit_path(name: &str, repo: &Url, commit_hash: &str) -> PathBuf {
    let repo_dir_name = git_repo_dir_name(name, repo);
    git_checkouts_directory()
        .join(repo_dir_name)
        .join(commit_hash)
}

/// Fetch the repo at the given git package's URL and checkout the pinned commit.
///
/// Returns the location of the checked out commit.
fn fetch_git(fetch_id: u64, name: &str, pinned: &SourceGitPinned) -> Result<PathBuf> {
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
                Source::Path(path)
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

/// Given a `forc_pkg::BuildConfig`, produce the necessary `sway_core::BuildConfig` required for
/// compilation.
pub fn sway_build_config(
    manifest_dir: &Path,
    entry_path: &Path,
    build_conf: &BuildConfig,
) -> Result<sway_core::BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let file_name = find_file_name(manifest_dir, entry_path)?;
    let build_config = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        manifest_dir.to_path_buf(),
    )
    .use_orig_asm(build_conf.use_orig_asm)
    .use_orig_parser(build_conf.use_orig_parser)
    .print_finalized_asm(build_conf.print_finalized_asm)
    .print_intermediate_asm(build_conf.print_intermediate_asm)
    .print_ir(build_conf.print_ir);
    Ok(build_config)
}

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
pub fn dependency_namespace(
    namespace_map: &HashMap<NodeIx, NamespaceRef>,
    graph: &Graph,
    compilation_order: &[NodeIx],
    node: NodeIx,
) -> NamespaceRef {
    use petgraph::visit::{Dfs, Walker};

    // Find all nodes that are a dependency of this one with a depth-first search.
    let deps: HashSet<NodeIx> = Dfs::new(graph, node).iter(graph).collect();

    // In order of compilation, accumulate dependency namespace refs.
    let namespace = sway_core::create_module();
    for &dep_node in compilation_order.iter().filter(|n| deps.contains(n)) {
        if dep_node == node {
            break;
        }
        // Add the namespace once for each of its names.
        let namespace_ref = namespace_map[&dep_node];
        let dep_names: BTreeSet<_> = graph
            .edges_directed(dep_node, Direction::Incoming)
            .map(|e| e.weight())
            .collect();
        for dep_name in dep_names {
            let dep_name = kebab_to_snake_case(dep_name);
            namespace.insert_module_ref(dep_name.to_string(), namespace_ref);
        }
    }

    namespace
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
    build_config: &BuildConfig,
    namespace: NamespaceRef,
    source_map: &mut SourceMap,
) -> Result<(Compiled, Option<NamespaceRef>)> {
    let entry_path = manifest.entry_path();
    let source = manifest.entry_string()?;
    let sway_build_config = sway_build_config(manifest.dir(), &entry_path, build_config)?;
    let silent_mode = build_config.silent;

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = sway_core::compile_to_ast(source, namespace, &sway_build_config);
    match &ast_res {
        CompileAstResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            bail!("Failed to compile {}", pkg.name);
        }
        CompileAstResult::Success {
            parse_tree,
            tree_type,
            warnings,
        } => {
            let json_abi = generate_json_abi(&*parse_tree);
            match tree_type {
                // If we're compiling a library, we don't need to compile any further.
                // Instead, we update the namespace with the library's top-level module.
                TreeType::Library { .. } => {
                    print_on_success_library(silent_mode, &pkg.name, warnings);
                    let bytecode = vec![];
                    let lib_namespace = parse_tree.clone().get_namespace_ref();
                    let compiled = Compiled { json_abi, bytecode };
                    Ok((compiled, Some(lib_namespace)))
                }

                // For all other program types, we'll compile the bytecode.
                TreeType::Contract | TreeType::Predicate | TreeType::Script => {
                    let tree_type = tree_type.clone();
                    let asm_res = sway_core::ast_to_asm(ast_res, &sway_build_config);
                    let bc_res = sway_core::asm_to_bytecode(asm_res, source_map);
                    match bc_res {
                        BytecodeCompilationResult::Success { bytes, warnings } => {
                            print_on_success(silent_mode, &pkg.name, &warnings, &tree_type);
                            let bytecode = bytes;
                            let compiled = Compiled { json_abi, bytecode };
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
pub fn build(
    plan: &BuildPlan,
    conf: &BuildConfig,
    sway_git_tag: &str,
) -> anyhow::Result<(Compiled, SourceMap)> {
    let mut namespace_map = Default::default();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut bytecode = vec![];
    for &node in &plan.compilation_order {
        let dep_namespace =
            dependency_namespace(&namespace_map, &plan.graph, &plan.compilation_order, node);
        let pkg = &plan.graph[node];
        let path = &plan.path_map[&pkg.id()];
        let manifest = ManifestFile::from_dir(path, sway_git_tag)?;
        let res = compile(pkg, &manifest, conf, dep_namespace, &mut source_map)?;
        let (compiled, maybe_namespace) = res;
        if let Some(namespace) = maybe_namespace {
            namespace_map.insert(node, namespace);
        }
        json_abi.extend(compiled.json_abi);
        bytecode = compiled.bytecode;
        source_map.insert_dependency(path.clone());
    }
    let compiled = Compiled { bytecode, json_abi };
    Ok((compiled, source_map))
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

// TODO: Update this to match behaviour described in the `compile` doc comment above.
fn generate_json_abi(ast: &TypedParseTree) -> JsonABI {
    match ast {
        TypedParseTree::Contract { abi_entries, .. } => {
            abi_entries.iter().map(|x| x.generate_json_abi()).collect()
        }
        _ => vec![],
    }
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
        .map(|e| e.to_friendly_error_string())
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("Parsing {} failed: \n{}", project_name, error);
    Error::msg(message)
}

/// Format an error message if an incorrect program type is present.
pub fn wrong_program_type(
    project_name: &str,
    expected_type: TreeType,
    parse_type: TreeType,
) -> anyhow::Error {
    let message = format!(
        "{} is not a '{:?}' it is a '{:?}'",
        project_name, expected_type, parse_type
    );
    Error::msg(message)
}

/// Format an error message if a given URL fails to produce a working node.
pub fn fuel_core_not_running(node_url: &str) -> anyhow::Error {
    let message = format!("could not get a response from node at the URL {}. Start a node with `fuel-core`. See https://github.com/FuelLabs/fuel-core#running for more information", node_url);
    Error::msg(message)
}
