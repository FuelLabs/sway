use crate::{
    lock::Lock,
    utils::{
        dependency::Dependency,
        helpers::{
            find_file_name, find_main_path, get_main_file, git_checkouts_directory,
            print_on_failure, print_on_success, print_on_success_library, read_manifest,
        },
        manifest::Manifest,
    },
};
use anyhow::{anyhow, bail, Result};
use petgraph::{self, visit::EdgeRef, Directed, Direction};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};
use sway_core::{
    source_map::SourceMap, BuildConfig, BytecodeCompilationResult, CompileAstResult, NamespaceRef,
    NamespaceWrapper, TreeType, TypedParseTree,
};
use sway_types::JsonABI;
use url::Url;

type GraphIx = u32;
type Node = Pinned;
type Edge = ();
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
    /// The unique name of the package.
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
    pub reference: String,
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
pub(crate) struct BuildPlan {
    pub(crate) graph: Graph,
    pub(crate) path_map: PathMap,
    pub(crate) compilation_order: Vec<NodeIx>,
}

// Parameters to pass through to the `BuildConfig` during compilation.
pub(crate) struct BuildConf {
    pub(crate) use_ir: bool,
    pub(crate) print_ir: bool,
    pub(crate) print_finalized_asm: bool,
    pub(crate) print_intermediate_asm: bool,
}

/// Error returned upon failed parsing of `SourceGitPinned::from_str`.
#[derive(Clone, Debug)]
pub enum SourceGitPinnedParseError {
    Prefix,
    Url,
    Reference,
    CommitHash,
}

impl BuildPlan {
    /// Create a new build plan for the project by fetching and pinning dependenies.
    pub fn new(manifest_dir: &Path, offline: bool) -> Result<Self> {
        let manifest = read_manifest(manifest_dir)?;
        let (graph, path_map) = fetch_deps(manifest_dir.to_path_buf(), &manifest, offline)?;
        let compilation_order = compilation_order(&graph)?;
        Ok(Self {
            graph,
            path_map,
            compilation_order,
        })
    }

    /// Attempt to load the build plan from the `Lock`.
    pub fn from_lock(proj_path: &Path, lock: &Lock) -> Result<Self> {
        let graph = lock.to_graph()?;
        let compilation_order = compilation_order(&graph)?;
        let path_map = graph_to_path_map(proj_path, &graph, &compilation_order)?;
        Ok(Self {
            graph,
            path_map,
            compilation_order,
        })
    }

    /// Attempt to load the build plan from the `Forc.lock` file.
    pub fn from_lock_file(lock_path: &Path) -> Result<Self> {
        let proj_path = lock_path.parent().unwrap();
        let lock = Lock::from_path(lock_path)?;
        Self::from_lock(proj_path, &lock)
    }

    /// Ensure that the build plan is valid for the given manifest.
    pub fn validate(&self, manifest: &Manifest) -> Result<()> {
        // Retrieve project's graph node.
        let proj_node = *self
            .compilation_order
            .last()
            .ok_or_else(|| anyhow!("Invalid Graph"))?;

        // Collect dependency `Source`s from graph.
        let plan_dep_pkgs: BTreeSet<_> = self
            .graph
            .edges_directed(proj_node, Direction::Outgoing)
            .map(|e| self.graph[e.target()].unpinned(&self.path_map))
            .collect();

        // Collect dependency `Source`s from manifest.
        let proj_id = self.graph[proj_node].id();
        let proj_path = &self.path_map[&proj_id];
        let manifest_dep_pkgs = manifest
            .dependencies
            .as_ref()
            .into_iter()
            .flat_map(|deps| deps.iter())
            .map(|(name, dep)| {
                // NOTE: Temporarily warn about `version` until we have support for registries.
                if let Dependency::Detailed(det) = dep {
                    if det.version.is_some() {
                        crate::utils::helpers::println_yellow_err(&format!(
                            "  WARNING! Dependency \"{}\" specifies the unused `version` field: \
                            consider using `branch` or `tag` instead",
                            name
                        ))
                        .unwrap();
                    }
                }

                let name = name.clone();
                let source = dep_to_source(proj_path, dep)?;
                Ok(Pkg { name, source })
            })
            .collect::<Result<BTreeSet<_>>>()?;

        // Ensure both `pkg::Source` are equal. If not, error.
        if plan_dep_pkgs != manifest_dep_pkgs {
            bail!("Manifest dependencies do not match");
        }

        Ok(())
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
        // git+<url/to/repo>?reference=<reference>#<commit>
        format!(
            "git+{}?reference={}#{}",
            self.source.repo, self.source.reference, self.commit_hash,
        )
    }
}

impl FromStr for SourceGitPinned {
    type Err = SourceGitPinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // git+<url/to/repo>?reference=<reference>#<commit>
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

        // Parse the "reference=" string.
        // TODO: This will need updating if we want to support omitting a git reference and allow
        // for specifying commit hashes directly in `Forc.toml` git dependencies.
        const REFERENCE: &str = "reference=";
        if s.find(REFERENCE) != Some(0) {
            return Err(SourceGitPinnedParseError::Reference);
        }
        let s = &s[REFERENCE.len()..];

        // And now retrieve the `reference` and `commit_hash` values.
        let mut s_iter = s.split('#');
        let reference = s_iter
            .next()
            .ok_or(SourceGitPinnedParseError::Reference)?
            .to_string();
        let commit_hash = s_iter
            .next()
            .ok_or(SourceGitPinnedParseError::CommitHash)?
            .to_string();

        let source = SourceGit { repo, reference };
        Ok(Self {
            source,
            commit_hash,
        })
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

    // Resolve all following dependencies, knowing their parents' paths will already be resolved.
    for dep_node in path_resolve_order {
        let dep = &graph[dep_node];
        let dep_path = match &dep.source {
            SourcePinned::Git(git) => {
                git_commit_path(&dep.name, &git.source.repo, &git.commit_hash)
            }
            SourcePinned::Path => {
                let parent_node = graph
                    .edges_directed(dep_node, Direction::Incoming)
                    .next()
                    .ok_or_else(|| anyhow!("more than one root package detected in graph"))?
                    .source();
                let parent = &graph[parent_node];
                let parent_path = &path_map[&parent.id()];
                let parent_manifest = read_manifest(parent_path)?;
                let detailed = parent_manifest
                    .dependencies
                    .as_ref()
                    .and_then(|deps| match &deps[&dep.name] {
                        Dependency::Detailed(detailed) => Some(detailed),
                        Dependency::Simple(_) => None,
                    })
                    .ok_or_else(|| anyhow!("missing path info for dependency: {}", dep.name))?;
                let rel_dep_path = detailed
                    .path
                    .as_ref()
                    .ok_or_else(|| anyhow!("missing path info for dependency: {}", dep.name))?;
                parent_path.join(rel_dep_path)
            }
            SourcePinned::Registry(_reg) => {
                bail!("registry dependencies are not yet supported");
            }
        };
        if !dep_path.exists() {
            match &dep.source {
                SourcePinned::Path => {
                    bail!("pinned `path` dependency \"{}\" source missing", dep.name);
                }
                SourcePinned::Git(git) => {
                    println!("  Fetching {}", git.to_string());
                    fetch_git(&dep.name, git)?;
                }
                SourcePinned::Registry(_reg) => {
                    bail!("registry dependencies are not yet supported");
                }
            }
        }
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
    offline_mode: bool,
) -> Result<(Graph, PathMap)> {
    let mut graph = Graph::new();
    let mut path_map = PathMap::new();

    // Add the project to the graph as the root node.
    let name = proj_manifest.project.name.clone();
    let path = proj_manifest_dir;
    let source = SourcePinned::Path;
    let pkg = Pinned { name, source };
    path_map.insert(pkg.id(), path);
    let root = graph.add_node(pkg);

    // The set of visited packages, starting with the root.
    let mut visited = HashMap::new();
    visited.insert(graph[root].clone(), root);

    // Recursively fetch children and add them to the graph.
    // TODO: Convert this recursion to use loop & stack to ensure deps can't cause stack overflow.
    fetch_children(offline_mode, root, &mut graph, &mut path_map, &mut visited)?;

    Ok((graph, path_map))
}

/// Fetch children nodes of the given node and add unvisited nodes to the graph.
fn fetch_children(
    offline_mode: bool,
    node: NodeIx,
    graph: &mut Graph,
    path_map: &mut PathMap,
    visited: &mut HashMap<Pinned, NodeIx>,
) -> Result<()> {
    let parent = &graph[node];
    let parent_path = path_map[&parent.id()].clone();
    let manifest = read_manifest(&parent_path)?;
    let deps = match &manifest.dependencies {
        None => return Ok(()),
        Some(deps) => deps,
    };
    for (name, dep) in deps {
        let name = name.clone();
        let source = dep_to_source(&parent_path, dep)?;
        if offline_mode && !matches!(source, Source::Path(_)) {
            bail!("Unable to fetch pkg {:?} in offline mode", source);
        }
        let pkg = Pkg { name, source };
        let pinned = pin_pkg(&pkg, path_map)?;
        let dep_node = if let hash_map::Entry::Vacant(entry) = visited.entry(pinned.clone()) {
            let node = graph.add_node(pinned);
            entry.insert(node);
            fetch_children(offline_mode, node, graph, path_map, visited)?;
            node
        } else {
            visited[&pinned]
        };
        graph.add_edge(node, dep_node, ());
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
/// $HOME/.forc/git/checkouts/tmp/name-<repo_url_hash>
/// ```
fn tmp_git_repo_dir(name: &str, repo: &Url) -> PathBuf {
    let repo_dir_name = git_repo_dir_name(name, repo);
    git_checkouts_directory().join("tmp").join(repo_dir_name)
}

/// Clones the package git repo into a temporary directory and applies the given function.
fn with_tmp_git_repo<F, O>(name: &str, source: &SourceGit, f: F) -> Result<O>
where
    F: FnOnce(git2::Repository) -> Result<O>,
{
    // Clear existing temporary directory if it exists.
    let repo_dir = tmp_git_repo_dir(name, &source.repo);
    if repo_dir.exists() {
        let _ = std::fs::remove_dir_all(&repo_dir);
    }

    // Clone repo into temporary directory.
    let repo_url_string = format!("{}", source.repo);
    let repo = git2::Repository::clone(&repo_url_string, &repo_dir).map_err(|e| {
        anyhow!(
            "failed to clone package '{}' from '{}': {}",
            name,
            source.repo,
            e
        )
    })?;

    // Do something with the repo.
    let output = f(repo)?;

    // Clean up the temporary directory.
    if repo_dir.exists() {
        let _ = std::fs::remove_dir_all(&repo_dir);
    }

    Ok(output)
}

/// Pin the given git-sourced package.
///
/// This clones the repository to a temporary directory in order to determine the commit at the
/// HEAD of the given git reference.
fn pin_git(name: &str, source: SourceGit) -> Result<SourceGitPinned> {
    let commit_hash = with_tmp_git_repo(name, &source, |repo| {
        // Find specified reference in repo.
        let reference = repo
            .resolve_reference_from_short_name(&source.reference)
            .map_err(|e| {
                anyhow!(
                    "failed to find git ref '{}' for package '{}': {}",
                    source.reference,
                    name,
                    e
                )
            })?;

        // Follow the reference until we find the latest commit and retrieve its hash.
        let commit = reference.peel_to_commit().map_err(|e| {
            anyhow!(
                "failed to obtain commit for ref '{}' of package '{}': {}",
                source.reference,
                name,
                e
            )
        })?;
        Ok(format!("{}", commit.id()))
    })?;
    Ok(SourceGitPinned {
        source,
        commit_hash,
    })
}

/// Given a package source, attempt to determine the pinned version or commit.
///
/// Also updates the `path_map` with a path to the local copy of the source.
fn pin_pkg(pkg: &Pkg, path_map: &mut PathMap) -> Result<Pinned> {
    let name = pkg.name.clone();
    let pinned = match &pkg.source {
        Source::Path(path) => {
            let source = SourcePinned::Path;
            let pinned = Pinned { name, source };
            let id = pinned.id();
            path_map.insert(id, path.clone());
            pinned
        }
        Source::Git(ref source) => {
            let pinned_git = pin_git(&name, source.clone())?;
            let path = git_commit_path(&name, &pinned_git.source.repo, &pinned_git.commit_hash);
            let source = SourcePinned::Git(pinned_git.clone());
            let pinned = Pinned { name, source };
            let id = pinned.id();
            if let hash_map::Entry::Vacant(entry) = path_map.entry(id) {
                // TODO: Here we assume that if the local path already exists, that it contains the full and
                // correct source for that commit and hasn't been tampered with. This is probably fine for most
                // cases as users should never be touching these directories, however we should add some code
                // to validate this. E.g. can we recreate the git hash by hashing the directory or something
                // along these lines using git?
                if !path.exists() {
                    println!("  Fetching {}", pinned_git.to_string());
                    fetch_git(&pinned.name, &pinned_git)?;
                }
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
fn fetch_git(name: &str, pinned: &SourceGitPinned) -> Result<PathBuf> {
    let path = git_commit_path(name, &pinned.source.repo, &pinned.commit_hash);

    // Checkout the pinned hash to the path.
    with_tmp_git_repo(name, &pinned.source, |repo| {
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
        Dependency::Simple(ref _ver_str) => unimplemented!(),
        Dependency::Detailed(ref det) => {
            match (&det.path, &det.version, &det.git, &det.branch, &det.tag) {
                (Some(relative_path), _, _, _, _) => {
                    let path = pkg_path.join(relative_path);
                    Source::Path(path)
                }
                (_, _, Some(repo), branch, tag) => {
                    let reference = match (branch, tag) {
                        (Some(branch), None) => branch.clone(),
                        (None, Some(tag)) => tag.clone(),
                        // TODO: Consider "main" or having no default at all.
                        _ => "master".to_string(),
                    };
                    let repo = Url::parse(repo)?;
                    let source = SourceGit { repo, reference };
                    Source::Git(source)
                }
                _ => {
                    bail!("unsupported set of arguments for dependency: {:?}", dep);
                }
            }
        }
    };
    Ok(source)
}

pub(crate) fn build_config(
    path: PathBuf,
    manifest: &Manifest,
    build_conf: &BuildConf,
) -> Result<BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let main_path = find_main_path(&path, manifest);
    let file_name = find_file_name(&path, &main_path)?;
    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        path.to_path_buf(),
    )
    .use_ir(build_conf.use_ir || build_conf.print_ir) // --print-ir implies --use-ir.
    .print_finalized_asm(build_conf.print_finalized_asm)
    .print_intermediate_asm(build_conf.print_intermediate_asm)
    .print_ir(build_conf.print_ir);
    Ok(build_config)
}

/// Builds the dependency namespace for the package at the given node index within the graph.
///
/// This function is designed to be called for each node in order of compilation.
pub(crate) fn dependency_namespace(
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
    for dep_node in compilation_order.iter().filter(|n| deps.contains(n)) {
        if *dep_node == node {
            break;
        }
        namespace.insert_module_ref(graph[*dep_node].name.clone(), namespace_map[dep_node]);
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
pub(crate) fn compile(
    pkg: &Pinned,
    pkg_path: &Path,
    build_conf: &BuildConf,
    namespace: NamespaceRef,
    source_map: &mut SourceMap,
    silent_mode: bool,
) -> Result<(Compiled, Option<NamespaceRef>)> {
    let manifest = read_manifest(pkg_path)?;
    let source = get_main_file(&manifest, pkg_path)?;
    let build_config = build_config(pkg_path.to_path_buf(), &manifest, build_conf)?;

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = sway_core::compile_to_ast(source, namespace, &build_config);
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
                    let asm_res = sway_core::ast_to_asm(ast_res, &build_config);
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
        "git+https://github.com/foo/bar?reference=baz#64092602dd6158f3e41d775ed889389440a2cd86",
        "git+https://github.com/fuellabs/sway-lib-std?reference=v0.1.0#0000000000000000000000000000000000000000",
        "git+https://github.com/fuellabs/sway-lib-core?reference=v0.0.1#0000000000000000000000000000000000000000",
    ];

    let expected = [
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/foo/bar").unwrap(),
                reference: "baz".to_string(),
            },
            commit_hash: "64092602dd6158f3e41d775ed889389440a2cd86".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/fuellabs/sway-lib-std").unwrap(),
                reference: "v0.1.0".to_string(),
            },
            commit_hash: "0000000000000000000000000000000000000000".to_string(),
        },
        SourceGitPinned {
            source: SourceGit {
                repo: Url::parse("https://github.com/fuellabs/sway-lib-core").unwrap(),
                reference: "v0.0.1".to_string(),
            },
            commit_hash: "0000000000000000000000000000000000000000".to_string(),
        },
    ];

    for (&string, expected) in strings.iter().zip(&expected) {
        let parsed = SourceGitPinned::from_str(string).unwrap();
        assert_eq!(&parsed, expected);
        let serialized = expected.to_string();
        assert_eq!(&serialized, string);
    }
}
