use crate::pkg;
use anyhow::{anyhow, Result};
use forc_util::{println_green, println_red};
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
pub struct Lock {
    // Named `package` so that each entry serializes to lock file under `[[package]]` like cargo.
    pub(crate) package: BTreeSet<PkgLock>,
}

/// Packages that have been removed and added between two `Lock` instances.
///
/// The result of `new_lock.diff(&old_lock)`.
pub struct Diff<'a> {
    pub removed: BTreeSet<&'a PkgLock>,
    pub added: BTreeSet<&'a PkgLock>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct PkgLock {
    pub(crate) name: String,
    // TODO: Cargo *always* includes version, whereas we don't even parse it when reading a
    // project's `Manifest` yet. If we decide to enforce versions, we'll want to remove the
    // `Option`.
    version: Option<semver::Version>,
    // Short-hand string describing where this package is sourced from.
    source: String,
    dependencies: Vec<PkgDepLine>,
}

/// `PkgDepLine` is a terse, single-line, git-diff-friendly description of a package's
/// dependency. It is formatted like so:
///
/// ```ignore
/// (<dep_name>) <pkg_name> <source_string>
/// ```
///
/// The `(<dep_name>)` segment is only included in the uncommon case that the dependency name does
/// not match the package name, i.e. if the `package` field was specified for the dependency.
///
/// The source string is included in order to be able to uniquely distinguish between multiple
/// different versions of the same package.
pub type PkgDepLine = String;

/// Convert the given package source to a string for use in the package lock.
///
/// Returns `None` for sources that refer to a direct `Path`.
pub fn source_to_string(source: &pkg::SourcePinned) -> String {
    match source {
        pkg::SourcePinned::Root => "root".to_string(),
        pkg::SourcePinned::Path(src) => src.to_string(),
        pkg::SourcePinned::Git(src) => src.to_string(),
        pkg::SourcePinned::Registry(_reg) => unimplemented!("pkg registries not yet implemented"),
    }
}

/// Convert the given package source string read from a package lock to a `pkg::SourcePinned`.
pub fn source_from_str(s: &str) -> Result<pkg::SourcePinned> {
    let source = if s == "root" {
        pkg::SourcePinned::Root
    } else if let Ok(src) = pkg::SourcePathPinned::from_str(s) {
        pkg::SourcePinned::Path(src)
    } else if let Ok(src) = pkg::SourceGitPinned::from_str(s) {
        pkg::SourcePinned::Git(src)
    } else {
        // TODO: Try parse registry source.
        return Err(anyhow!(
            "Unable to parse valid pinned source from given string {}",
            s
        ));
    };
    Ok(source)
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
                let dep_name = edge.weight();
                let dep_node = edge.target();
                let dep_pkg = &graph[dep_node];
                let dep_name = if *dep_name != dep_pkg.name {
                    Some(&dep_name[..])
                } else {
                    None
                };
                let source_string = source_to_string(&dep_pkg.source);
                pkg_dep_line(dep_name, &dep_pkg.name, &source_string)
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
        pkg_unique_string(&self.name, &self.source)
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
            let source = source_from_str(&pkg.source)
                .map_err(|e| anyhow!("invalid 'source' entry for package {} lock: {}", name, e))?;
            let pkg = pkg::Pinned { name, source };
            let node = graph.add_node(pkg);
            pkg_to_node.insert(key, node);
        }

        // On the second pass, add all edges.
        for pkg in &self.package {
            let key = pkg.unique_string();
            let node = pkg_to_node[&key];
            for dep_line in &pkg.dependencies {
                let (dep_name, dep_key) = parse_pkg_dep_line(dep_line)
                    .map_err(|e| anyhow!("failed to parse dependency \"{}\": {}", dep_line, e))?;
                let dep_node = pkg_to_node
                    .get(dep_key)
                    .cloned()
                    .ok_or_else(|| anyhow!("found dep {} without node entry in graph", dep_key))?;
                let dep_name = dep_name.unwrap_or(&graph[dep_node].name).to_string();
                graph.add_edge(node, dep_node, dep_name);
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

fn pkg_unique_string(name: &str, source: &str) -> String {
    format!("{} {}", name, source)
}

fn pkg_dep_line(dep_name: Option<&str>, name: &str, source: &str) -> PkgDepLine {
    let pkg_string = pkg_unique_string(name, source);
    match dep_name {
        None => pkg_string,
        Some(dep_name) => format!("({}) {}", dep_name, pkg_string),
    }
}

// Parse the given `PkgDepLine` into its dependency name and unique string segments.
//
// I.e. given "(<dep_name>) <name> <source>", returns ("<dep_name>", "<name> <source>").
fn parse_pkg_dep_line(pkg_dep_line: &str) -> anyhow::Result<(Option<&str>, &str)> {
    let s = pkg_dep_line.trim();

    // Check for the open bracket.
    if !s.starts_with('(') {
        return Ok((None, s));
    }

    // If we have the open bracket, grab everything until the closing bracket.
    let s = &s["(".len()..];
    let mut iter = s.split(')');
    let dep_name = iter
        .next()
        .ok_or_else(|| anyhow!("missing closing parenthesis"))?;

    // The rest is the unique package string.
    let s = &s[dep_name.len() + ")".len()..];
    let pkg_str = s.trim_start();
    Ok((Some(dep_name), pkg_str))
}

pub fn print_diff(proj_name: &str, diff: &Diff) {
    print_removed_pkgs(proj_name, diff.removed.iter().cloned());
    print_added_pkgs(proj_name, diff.added.iter().cloned());
}

pub fn print_removed_pkgs<'a, I>(proj_name: &str, removed: I)
where
    I: IntoIterator<Item = &'a PkgLock>,
{
    for pkg in removed {
        if pkg.name != proj_name {
            let _ = println_red(&format!("  Removing {}", pkg.unique_string()));
        }
    }
}

pub fn print_added_pkgs<'a, I>(proj_name: &str, removed: I)
where
    I: IntoIterator<Item = &'a PkgLock>,
{
    for pkg in removed {
        if pkg.name != proj_name {
            let _ = println_green(&format!("    Adding {}", pkg.unique_string()));
        }
    }
}
