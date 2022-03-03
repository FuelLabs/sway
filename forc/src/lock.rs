use crate::pkg;
use anyhow::{anyhow, Result};
use petgraph::{visit::EdgeRef, Direction};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::Path,
    str::FromStr,
};

/// The graph of pinned packages represented as a toml-serialization-friendly structure.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Lock {
    // Named `package` so that each entry serializes to lock file under `[[package]]` like cargo.
    pub(crate) package: BTreeSet<PkgLock>,
}

/// Packages that have been removed and added between two `Lock` instances.
///
/// The result of `new_lock.diff(&old_lock)`.
pub(crate) struct Diff<'a> {
    pub(crate) removed: BTreeSet<&'a PkgLock>,
    pub(crate) added: BTreeSet<&'a PkgLock>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub(crate) struct PkgLock {
    pub(crate) name: String,
    // TODO: Cargo *always* includes version, whereas we don't even parse it when reading a
    // project's `Manifest` yet. If we decide to enforce versions, we'll want to remove the
    // `Option`.
    version: Option<semver::Version>,
    source: Option<String>,
    // Dependency string is "<name> <source_string>". The source string is included in order to be
    // able to uniquely distinguish between multiple different versions of the same package.
    dependencies: Vec<String>,
}

/// Convert the given package source to a string for use in the package lock.
///
/// Returns `None` for sources that refer to a direct `Path`.
pub fn source_to_string(source: &pkg::SourcePinned) -> Option<String> {
    match source {
        pkg::SourcePinned::Path => None,
        pkg::SourcePinned::Git(git) => Some(git.to_string()),
        pkg::SourcePinned::Registry(_reg) => unimplemented!("pkg registries not yet implemented"),
    }
}

/// Convert the given package source string read from a package lock to a `pkg::SourcePinned`.
pub fn source_from_str(s: &str) -> Result<pkg::SourcePinned> {
    if let Ok(src) = pkg::SourceGitPinned::from_str(s) {
        return Ok(pkg::SourcePinned::Git(src));
    }
    // TODO: Try parse registry source.
    Err(anyhow!(
        "Unable to parse valid pinned source from given string {}",
        s
    ))
}

impl PkgLock {
    /// Construct a package lock given a package's entry in the package graph.
    pub fn from_node(graph: &pkg::Graph, node: pkg::NodeIx) -> Self {
        let pinned = &graph[node];
        let name = pinned.name.clone();
        let version = match &pinned.source {
            pkg::SourcePinned::Registry(reg) => Some(reg.source.version.clone()),
            _ => None,
        };
        let source = source_to_string(&pinned.source);
        let mut dependencies: Vec<String> = graph
            .edges_directed(node, Direction::Outgoing)
            .map(|edge| {
                let dep_node = edge.target();
                let dep = &graph[dep_node];
                let source_string = source_to_string(&dep.source);
                pkg_unique_string(&dep.name, source_string.as_deref())
            })
            .collect();
        dependencies.sort();
        Self {
            name,
            version,
            source,
            dependencies,
        }
    }

    /// The string representation used for specifying this package as a dependency.
    pub fn unique_string(&self) -> String {
        pkg_unique_string(&self.name, self.source.as_deref())
    }
}

impl Lock {
    /// Load the `Lock` structure from the TOML `Forc.lock` file at the specified path.
    pub fn from_path(path: &Path) -> Result<Self> {
        let string = fs::read_to_string(&path)
            .map_err(|e| anyhow!("failed to read {}: {}", path.display(), e))?;
        toml::de::from_str(&string).map_err(|e| anyhow!("failed to parse lock file: {}", e))
    }

    /// Given a graph of pinned packages, create a `Lock` representing the `Forc.lock` file
    /// structure.
    pub fn from_graph(graph: &pkg::Graph) -> Self {
        let package: BTreeSet<_> = graph
            .node_indices()
            .map(|node| PkgLock::from_node(graph, node))
            .collect();
        Self { package }
    }

    /// Given a `Lock` loaded from a `Forc.lock` file, produce the graph of pinned dependencies.
    pub fn to_graph(&self) -> Result<pkg::Graph> {
        let mut graph = pkg::Graph::new();

        // On the first pass, add all nodes to the graph.
        // Keep track of name+source to node-index mappings for the edge collection pass.
        let mut pkg_to_node: HashMap<String, pkg::NodeIx> = HashMap::new();
        for pkg in &self.package {
            let key = pkg.unique_string();
            let name = pkg.name.clone();
            // TODO: We shouldn't use `pkg::SourcePinned` as we don't actually know the `Path`
            // until we follow the dependency graph. Use something like a `ParsedSource` type here
            // instead.
            let pkg_source_string = pkg.source.clone();
            let source = match &pkg_source_string {
                None => pkg::SourcePinned::Path,
                Some(s) => source_from_str(s).map_err(|e| {
                    anyhow!("invalid 'source' entry for package {} lock: {}", name, e)
                })?,
            };
            let pkg = pkg::Pinned { name, source };
            let node = graph.add_node(pkg);
            pkg_to_node.insert(key, node);
        }

        // On the second pass, add all edges.
        for pkg in &self.package {
            let key = pkg.unique_string();
            let node = pkg_to_node[&key];
            for dep_key in &pkg.dependencies {
                let dep_node = pkg_to_node
                    .get(&dep_key[..])
                    .cloned()
                    .ok_or_else(|| anyhow!("found dep {} without node entry in graph", dep_key))?;
                graph.add_edge(node, dep_node, ());
            }
        }

        Ok(graph)
    }

    /// Create a diff between `self` and the `old` `Lock`.
    ///
    /// Useful for showing the user which dependencies are out of date, or which have been updated.
    pub fn diff<'a>(&'a self, old: &'a Self) -> Diff<'a> {
        let added = self.package.difference(&old.package).collect();
        let removed = old.package.difference(&self.package).collect();
        Diff { added, removed }
    }
}

fn pkg_unique_string(name: &str, source: Option<&str>) -> String {
    match source {
        None => name.to_string(),
        Some(s) => format!("{} {}", name, s),
    }
}
