use crate::{
    cli::BuildCommand,
    utils::{
        dependency::Dependency,
        helpers::{
            default_output_directory, find_file_name, find_main_path, get_main_file,
            git_checkouts_directory, print_on_failure, print_on_success, print_on_success_library,
            read_manifest,
        },
        manifest::Manifest,
    },
};
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use sway_core::{
    create_module, source_map::SourceMap, BuildConfig, BytecodeCompilationResult, CompileAstResult,
    NamespaceRef, NamespaceWrapper, TreeType, TypedParseTree,
};
use sway_types::JsonABI;
use sway_utils::{find_manifest_dir, MANIFEST_FILE_NAME};
use url::Url;

/// The result of compiling a particular package.
pub struct CompiledPkg {
    pub json_abi: JsonABI,
    pub bytecode: Vec<u8>,
}

/// A package uniquely identified by name along with its source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pkg {
    /// The unique name of the package.
    pub name: String,
    /// Where the package is sourced from.
    pub source: PkgSource,
}

/// A package uniquely identified by name along with its pinned source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgPinned {
    pub name: String,
    pub source: PkgSourcePinned,
}

/// A package uniquely identified by name along with its pinned source and fetched path.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgPinnedFetched {
    /// The pinned package that has been fetched.
    pub pkg: PkgPinned,
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
/// Note that a `PkgSource` does not specify a specific, pinned version. Rather, it specifies a
/// source at which the current latest version may be located.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum PkgSource {
    /// A git repo with a `Forc.toml` manifest at its root.
    Git(PkgSourceGit),
    /// A path to a directory with a `Forc.toml` manifest at its root.
    Path(PathBuf),
    /// A forc project hosted on the official registry.
    Registry(PkgSourceRegistry),
}

/// A git repo with a `Forc.toml` manifest at its root.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgSourceGit {
    /// The URL at which the repository is located.
    pub repo: Url,
    /// A git reference, e.g. a branch or tag.
    pub reference: String,
}

/// A package from the official registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgSourceRegistry {
    /// The base version specified for the package.
    pub version: semver::Version,
}

/// A pinned instance of a git source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgSourceGitPinned {
    /// The git source that is being pinned.
    pub source: PkgSourceGit,
    /// The hash to which we have pinned the source.
    pub commit_hash: String,
}

/// A pinned instance of the registry source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PkgSourceRegistryPinned {
    /// The registry package with base version.
    pub source: PkgSourceRegistry,
    /// The pinned version.
    pub version: semver::Version,
}

/// A pinned instance of the package source.
///
/// Specifies an exact version to use, or an exact commit in the case of git dependencies. The
/// pinned version or commit is updated upon creation of the lock file and on `forc update`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum PkgSourcePinned {
    Git(PkgSourceGitPinned),
    Path(PathBuf),
    Registry(PkgSourceRegistryPinned),
}

// Parameters to pass through to the `BuildConfig` during compilation.
struct BuildConf {
    use_ir: bool,
    print_ir: bool,
    print_finalized_asm: bool,
    print_intermediate_asm: bool,
}

type GraphIx = u32;
type PkgGraph = petgraph::Graph<PkgPinnedFetched, (), petgraph::Directed, GraphIx>;
type NodeIx = petgraph::graph::NodeIndex<GraphIx>;

impl std::ops::Deref for PkgPinnedFetched {
    type Target = PkgPinned;
    fn deref(&self) -> &Self::Target {
        &self.pkg
    }
}

pub fn build(command: BuildCommand) -> Result<CompiledPkg> {
    let BuildCommand {
        path,
        binary_outfile,
        use_ir,
        debug_outfile,
        print_finalized_asm,
        print_intermediate_asm,
        print_ir,
        offline_mode,
        silent_mode,
        output_directory,
        minify_json_abi,
    } = command;

    let build_conf = BuildConf {
        use_ir,
        print_ir,
        print_finalized_asm,
        print_intermediate_asm,
    };

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(anyhow!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            ))
        }
    };
    let manifest = read_manifest(&manifest_dir)?;

    // Produce the graph of packages.
    // TODO: We should first try to load this from something like a `Forc.lock`.
    let pkg_graph = fetch_deps(manifest_dir.clone(), &manifest, offline_mode)?;

    // TODO: Warn about duplicate pkg names with differing versions/sources.

    // The `pkg_graph` is of *a -> b* where *a* depends on *b*. We can determine compilation order
    // by performing a toposort of the graph with reversed weights.
    let rev_pkg_graph = petgraph::visit::Reversed(&pkg_graph);
    let compilation_order = petgraph::algo::toposort(rev_pkg_graph, None)
        // TODO: Show full list of packages that cycle.
        .map_err(|e| anyhow!("dependency cycle detected: {:?}", e))?;

    // Iterate over and compile all packages.
    let namespace = create_module();
    let mut source_map = SourceMap::new();
    let mut json_abi = vec![];
    let mut bytecode = vec![];
    for node in compilation_order {
        let pkg = &pkg_graph[node];
        let compiled = compile_pkg(pkg, &build_conf, namespace, &mut source_map, silent_mode)?;
        json_abi.extend(compiled.json_abi);
        bytecode = compiled.bytecode;
        source_map.insert_dependency(pkg.path.clone());
    }

    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile)?;
        file.write_all(bytecode.as_slice())?;
    }

    if let Some(outfile) = debug_outfile {
        fs::write(
            outfile,
            &serde_json::to_vec(&source_map).expect("JSON serialization failed"),
        )
        .map_err(|e| e)?;
    }

    // TODO: We may support custom build profiles in the future.
    let profile = "debug";

    // Create the output directory for build artifacts.
    let output_dir = output_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output_directory(&manifest_dir).join(profile));
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e)?;
    }

    // Place build artifacts into the output directory.
    let bin_path = output_dir
        .join(&manifest.project.name)
        .with_extension("bin");
    std::fs::write(&bin_path, bytecode.as_slice())?;
    if !json_abi.is_empty() {
        let json_abi_stem = format!("{}-abi", manifest.project.name);
        let json_abi_path = output_dir.join(&json_abi_stem).with_extension("json");
        let file = File::create(json_abi_path).map_err(|e| e)?;
        let res = if minify_json_abi {
            serde_json::to_writer(&file, &json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &json_abi)
        };
        res.map_err(|e| e)?;
    }

    println!("  Bytecode size is {} bytes.", bytecode.len());

    Ok(CompiledPkg { bytecode, json_abi })
}

/// Fetch all depedencies and produce the dependency graph.
///
/// This will determine pinned versions and commits for remote dependencies during traversal.
fn fetch_deps(
    proj_manifest_dir: PathBuf,
    proj_manifest: &Manifest,
    offline_mode: bool,
) -> Result<PkgGraph> {
    let mut graph = PkgGraph::new();

    // Add the project to the graph as the root node.
    let name = proj_manifest.project.name.clone();
    let path = proj_manifest_dir;
    let source = PkgSourcePinned::Path(path.clone());
    let pkg = PkgPinned { name, source };
    let fetched = PkgPinnedFetched { pkg, path };
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
    graph: &mut PkgGraph,
    visited: &mut HashMap<PkgPinned, NodeIx>,
) -> Result<()> {
    let fetched = &graph[node];
    let manifest = read_manifest(&fetched.path)?;
    let deps = match &manifest.dependencies {
        None => return Ok(()),
        Some(deps) => deps,
    };
    for (name, dep) in deps {
        let name = name.clone();
        let source = dep_to_pkg_source(dep)?;
        if offline_mode && !matches!(source, PkgSource::Path(_)) {
            bail!("Unable to fetch pkg {:?} in offline mode", source);
        }
        let pkg = Pkg { name, source };
        let pinned = pin_pkg(&pkg)?;
        let dep_node = if let Entry::Vacant(entry) = visited.entry(pinned.clone()) {
            let fetched = fetch_pkg_pinned(pinned)?;
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
fn pkg_git_repo_dir_name(name: &str, repo: &Url) -> String {
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
    let repo_dir_name = pkg_git_repo_dir_name(name, repo);
    git_checkouts_directory().join("tmp").join(repo_dir_name)
}

/// Clones the package git repo into a temporary directory and applies the given function.
fn with_pkg_tmp_git_repo<F, O>(name: &str, source: &PkgSourceGit, f: F) -> Result<O>
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
fn pin_pkg_git(name: &str, source: PkgSourceGit) -> Result<PkgSourceGitPinned> {
    let commit_hash = with_pkg_tmp_git_repo(name, &source, |repo| {
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
    Ok(PkgSourceGitPinned {
        source,
        commit_hash,
    })
}

/// Given a package source, attempt to determine the pinned version or commit.
fn pin_pkg(pkg: &Pkg) -> Result<PkgPinned> {
    let source = match &pkg.source {
        PkgSource::Path(path) => PkgSourcePinned::Path(path.clone()),
        PkgSource::Git(ref source) => {
            let pinned = pin_pkg_git(&pkg.name, source.clone())?;
            PkgSourcePinned::Git(pinned)
        }
        PkgSource::Registry(ref _source) => {
            unimplemented!("determine registry pkg git URL, fetch to determine latest available semver-compatible version")
        }
    };
    let name = pkg.name.clone();
    let pinned = PkgPinned { name, source };
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
fn pkg_git_commit_path(name: &str, repo: &Url, commit_hash: &str) -> PathBuf {
    let repo_dir_name = pkg_git_repo_dir_name(name, repo);
    git_checkouts_directory()
        .join(repo_dir_name)
        .join(commit_hash)
}

/// Fetch the repo at the given git package's URL and checkout the pinned commit.
///
/// Returns the location of the checked out commit.
fn fetch_pkg_git(name: &str, pinned: &PkgSourceGitPinned) -> Result<PathBuf> {
    let path = pkg_git_commit_path(name, &pinned.source.repo, &pinned.commit_hash);

    // Checkout the pinned hash to the path.
    with_pkg_tmp_git_repo(name, &pinned.source, |repo| {
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
fn fetch_pkg_pinned(pkg: PkgPinned) -> Result<PkgPinnedFetched> {
    let path = match &pkg.source {
        PkgSourcePinned::Git(pinned) => fetch_pkg_git(&pkg.name, pinned)?,
        PkgSourcePinned::Path(path) => path.clone(),
        PkgSourcePinned::Registry(_pinned) => {
            unimplemented!("fetch pinned package from registry");
        }
    };
    let fetched = PkgPinnedFetched { pkg, path };
    Ok(fetched)
}

fn dep_to_pkg_source(dep: &Dependency) -> Result<PkgSource> {
    let source = match dep {
        Dependency::Simple(ref _ver_str) => unimplemented!(),
        Dependency::Detailed(ref det) => match (&det.path, &det.version, &det.git, &det.branch) {
            (Some(path), _, _, _) => PkgSource::Path(PathBuf::from(path)),
            (_, _, Some(repo), branch) => {
                let reference = match branch {
                    None => "master".to_string(),
                    Some(branch) => branch.clone(),
                };
                let repo = Url::parse(repo)?;
                let source = PkgSourceGit { repo, reference };
                PkgSource::Git(source)
            }
            _ => {
                bail!(
                    "unsupported set of arguments for dependency: {:?}",
                    dep
                );
            }
        },
    };
    Ok(source)
}

fn pkg_build_config(
    pkg_path: PathBuf,
    manifest: &Manifest,
    build_conf: &BuildConf,
) -> Result<BuildConfig> {
    // Prepare the build config to pass through to the compiler.
    let main_path = find_main_path(&pkg_path, manifest);
    let file_name = find_file_name(&pkg_path, &main_path)?;
    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
        file_name.to_path_buf(),
        pkg_path.to_path_buf(),
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
fn compile_pkg(
    pkg: &PkgPinnedFetched,
    build_conf: &BuildConf,
    namespace: NamespaceRef,
    source_map: &mut SourceMap,
    silent_mode: bool,
) -> Result<CompiledPkg> {
    let manifest = read_manifest(&pkg.path)?;
    let source = get_main_file(&manifest, &pkg.path)?;
    let build_config = pkg_build_config(pkg.path.clone(), &manifest, build_conf)?;

    // First, compile to an AST. We'll update the namespace and check for JSON ABI output.
    let ast_res = sway_core::compile_to_ast(source, namespace, &build_config);
    match &ast_res {
        CompileAstResult::Failure { warnings, errors } => {
            print_on_failure(silent_mode, warnings, errors);
            Err(anyhow!("Failed to compile {}", pkg.name))
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
                    Ok(CompiledPkg { json_abi, bytecode })
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
                            Ok(CompiledPkg { json_abi, bytecode })
                        }
                        BytecodeCompilationResult::Library { .. } => {
                            unreachable!("compilation of library program types is handled above")
                        }
                        BytecodeCompilationResult::Failure { errors, warnings } => {
                            print_on_failure(silent_mode, &warnings, &errors);
                            Err(anyhow!("Failed to compile {}", pkg.name))
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
