use anyhow::{anyhow, bail, Result};
use crate::utils::{
    dependency::Dependency,
    helpers::{
        find_file_name, find_main_path, get_main_file, git_checkouts_directory, print_on_failure,
        print_on_success, print_on_success_library, read_manifest,
    },
    manifest::Manifest,
};
use petgraph::{self, Directed};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    path::PathBuf,
};
use sway_core::{
    source_map::SourceMap, BuildConfig, BytecodeCompilationResult, CompileAstResult, NamespaceRef,
    NamespaceWrapper, TreeType, TypedParseTree,
};
use sway_types::JsonABI;
use url::Url;

type GraphIx = u32;
type Node = PinnedFetched;
type Edge = ();
pub type Graph = petgraph::Graph<Node, Edge, Directed, GraphIx>;
pub type NodeIx = petgraph::graph::NodeIndex<GraphIx>;

/// The result of successfully compiling a package.
pub struct Compiled {
    pub json_abi: JsonABI,
    pub bytecode: Vec<u8>,
}

/// A package uniquely identified by name along with its source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
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

/// A package uniquely identified by name along with its pinned source and fetched path.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PinnedFetched {
    /// The pinned package that has been fetched.
    pub pkg: Pinned,
    /// Path to the fetched source.
    ///
    /// For dependencies specified via `Path`, the original path is used.
    pub path: PathBuf,
}

/// Specifies a base source for a package.
///
/// - For registry packages, this includes a base version.
/// - For git packages, this includes a base git reference like a branch or tag.
///
/// Note that a `Source` does not specify a specific, pinned version. Rather, it specifies a source
/// at which the current latest version may be located.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Source {
    /// A git repo with a `Forc.toml` manifest at its root.
    Git(SourceGit),
    /// A path to a directory with a `Forc.toml` manifest at its root.
    Path(PathBuf),
    /// A forc project hosted on the official registry.
    Registry(SourceRegistry),
}

/// A git repo with a `Forc.toml` manifest at its root.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SourceGit {
    /// The URL at which the repository is located.
    pub repo: Url,
    /// A git reference, e.g. a branch or tag.
    pub reference: String,
}

/// A package from the official registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
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
    Path(PathBuf),
    Registry(SourceRegistryPinned),
}

// Parameters to pass through to the `BuildConfig` during compilation.
pub(crate) struct BuildConf {
    pub(crate) use_ir: bool,
    pub(crate) print_ir: bool,
    pub(crate) print_finalized_asm: bool,
    pub(crate) print_intermediate_asm: bool,
}

impl std::ops::Deref for PinnedFetched {
    type Target = Pinned;
    fn deref(&self) -> &Self::Target {
        &self.pkg
    }
}

/// Fetch all depedencies and produce the dependency graph.
///
/// This will determine pinned versions and commits for remote dependencies during traversal.
pub(crate) fn fetch_deps(
    proj_manifest_dir: PathBuf,
    proj_manifest: &Manifest,
    offline_mode: bool,
) -> Result<Graph> {
    let mut graph = Graph::new();

    // Add the project to the graph as the root node.
    let name = proj_manifest.project.name.clone();
    let path = proj_manifest_dir;
    let source = SourcePinned::Path(path.clone());
    let pkg = Pinned { name, source };
    let fetched = PinnedFetched { pkg, path };
    let root = graph.add_node(fetched);

    // The set of visited packages, starting with the root.
    let mut visited = HashMap::new();
    visited.insert(graph[root].pkg.clone(), root);

    // Recursively fetch children and add them to the graph.
    // TODO: Convert this recursion to use loop & stack to ensure deps can't cause stack overflow.
    fetch_children(offline_mode, root, &mut graph, &mut visited)?;

    Ok(graph)
}

/// Fetch children nodes of the given node and add unvisited nodes to the graph.
fn fetch_children(
    offline_mode: bool,
    node: NodeIx,
    graph: &mut Graph,
    visited: &mut HashMap<Pinned, NodeIx>,
) -> Result<()> {
    let fetched = &graph[node];
    let manifest = read_manifest(&fetched.path)?;
    let deps = match &manifest.dependencies {
        None => return Ok(()),
        Some(deps) => deps,
    };
    for (name, dep) in deps {
        let name = name.clone();
        let source = dep_to_source(dep)?;
        if offline_mode && !matches!(source, Source::Path(_)) {
            bail!("Unable to fetch pkg {:?} in offline mode", source);
        }
        let pkg = Pkg { name, source };
        let pinned = pin_pkg(&pkg)?;
        let dep_node = if let Entry::Vacant(entry) = visited.entry(pinned.clone()) {
            let fetched = fetch_pinned(pinned)?;
            let node = graph.add_node(fetched);
            entry.insert(node);
            fetch_children(offline_mode, node, graph, visited)?;
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
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
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
            name, source.repo, e
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
                    source.reference, name, e
                )
            })?;

        // Follow the reference until we find the latest commit and retrieve its hash.
        let commit = reference.peel_to_commit().map_err(|e| {
            anyhow!(
                "failed to obtain commit for ref '{}' of package '{}': {}",
                source.reference, name, e
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
fn pin_pkg(pkg: &Pkg) -> Result<Pinned> {
    let source = match &pkg.source {
        Source::Path(path) => SourcePinned::Path(path.clone()),
        Source::Git(ref source) => {
            let pinned = pin_git(&pkg.name, source.clone())?;
            SourcePinned::Git(pinned)
        }
        Source::Registry(ref _source) => {
            unimplemented!("determine registry pkg git URL, fetch to determine latest available semver-compatible version")
        }
    };
    let name = pkg.name.clone();
    let pinned = Pinned { name, source };
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

        // If it already exists, remove it as we're about to replace it.
        // In theory we could just leave it and use the existing directory as it *should* match what
        // we're about to clone into it, but we replace it just in case the directory has been tampered
        // with or is corrupted for whatever reason.
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

/// Given a package's pinned source ensure we have a copy of the source on the local filesystem.
fn fetch_pinned(pkg: Pinned) -> Result<PinnedFetched> {
    let path = match &pkg.source {
        SourcePinned::Git(pinned) => fetch_git(&pkg.name, pinned)?,
        SourcePinned::Path(path) => path.clone(),
        SourcePinned::Registry(_pinned) => {
            unimplemented!("fetch pinned package from registry");
        }
    };
    let fetched = PinnedFetched { pkg, path };
    Ok(fetched)
}

fn dep_to_source(dep: &Dependency) -> Result<Source> {
    let source = match dep {
        Dependency::Simple(ref _ver_str) => unimplemented!(),
        Dependency::Detailed(ref det) => match (&det.path, &det.version, &det.git, &det.branch) {
            (Some(path), _, _, _) => Source::Path(PathBuf::from(path)),
            (_, _, Some(repo), branch) => {
                let reference = match branch {
                    None => "master".to_string(),
                    Some(branch) => branch.clone(),
                };
                let repo = Url::parse(repo)?;
                let source = SourceGit { repo, reference };
                Source::Git(source)
            }
            _ => {
                bail!("unsupported set of arguments for dependency: {:?}", dep);
            }
        },
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

/// Compiles the given package.
///
/// ## Program Types
///
/// Behaviour differs slightly based on the package's program type.
///
/// ### Library Packages
///
/// A Library package will have JSON ABI generated for all publicly exposed `abi`s. The parsed AST
/// will be added as a module to the given overall namespace so that its items are accessible to
/// successively compiled packages. NOTE: This namespace is currently global, so be aware that
/// calling this multiple times for the same package will result in duplicate/shadowed name
/// conflicts.
///
/// ### Contract
///
/// Contracts will output both their JSON ABI and compiled bytecode.
///
/// ### Script, Predicate
///
/// Scripts and Predicates will be compiled to bytecode and will not emit any JSON ABI.
pub(crate) fn compile(
    pkg: &PinnedFetched,
    build_conf: &BuildConf,
    namespace: NamespaceRef,
    source_map: &mut SourceMap,
    silent_mode: bool,
) -> Result<Compiled> {
    let manifest = read_manifest(&pkg.path)?;
    let source = get_main_file(&manifest, &pkg.path)?;
    let build_config = build_config(pkg.path.clone(), &manifest, build_conf)?;

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
                    namespace.insert_module_ref(pkg.name.clone(), lib_namespace);
                    Ok(Compiled { json_abi, bytecode })
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
                            Ok(Compiled { json_abi, bytecode })
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

// TODO: Update this to match behaviour described in `compile_pkg` above.
fn generate_json_abi(ast: &TypedParseTree) -> JsonABI {
    match ast {
        TypedParseTree::Contract { abi_entries, .. } => {
            abi_entries.iter().map(|x| x.generate_json_abi()).collect()
        }
        _ => vec![],
    }
}
