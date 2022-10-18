use crate::{pkg, DepKind, Edge};
use anyhow::{anyhow, Result};
use forc_util::{println_green, println_red};
use petgraph::{visit::EdgeRef, Direction};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
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
/// (<dep_name>) <dep-kind> <pkg_name> <source_string>
/// ```
///
/// The `(<dep_name>)` segment is only included in the uncommon case that the dependency name does
/// not match the package name, i.e. if the `package` field was specified for the dependency.
///
/// The source string is included in order to be able to uniquely distinguish between multiple
/// different versions of the same package.
pub type PkgDepLine = String;

impl PkgLock {
    /// Construct a package lock given a package's entry in the package graph.
    pub fn from_node(graph: &pkg::Graph, node: pkg::NodeIx, disambiguate: &HashSet<&str>) -> Self {
        let pinned = &graph[node];
        let name = pinned.name.clone();
        let version = match &pinned.source {
            pkg::SourcePinned::Registry(reg) => Some(reg.source.version.clone()),
            _ => None,
        };
        let source = pinned.source.to_string();
        let mut dependencies: Vec<String> = graph
            .edges_directed(node, Direction::Outgoing)
            .map(|edge| {
                let dep_edge = edge.weight();
                let dep_node = edge.target();
                let dep_pkg = &graph[dep_node];
                let dep_name = if *dep_edge.name != dep_pkg.name {
                    Some(&dep_edge.name[..])
                } else {
                    None
                };
                let dep_kind = &dep_edge.kind;
                let disambiguate = disambiguate.contains(&dep_pkg.name[..]);
                pkg_dep_line(
                    dep_name,
                    &dep_pkg.name,
                    &dep_pkg.source,
                    dep_kind,
                    disambiguate,
                )
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

    /// A string that uniquely identifies a package and its source.
    ///
    /// Formatted as `<name> <source>`.
    pub fn unique_string(&self) -> String {
        pkg_unique_string(&self.name, &self.source)
    }

    /// The string representation used for specifying this package as a dependency.
    ///
    /// If this package's name is not enough to disambiguate it from other packages within the
    /// graph, this returns `<name> <source>`. If it is, it simply returns the name.
    pub fn name_disambiguated(&self, disambiguate: &HashSet<&str>) -> Cow<str> {
        let disambiguate = disambiguate.contains(&self.name[..]);
        pkg_name_disambiguated(&self.name, &self.source, disambiguate)
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
        let names = graph.node_indices().map(|n| &graph[n].name[..]);
        let disambiguate: HashSet<_> = names_requiring_disambiguation(names).collect();
        // Collect the packages.
        let package: BTreeSet<_> = graph
            .node_indices()
            .map(|node| PkgLock::from_node(graph, node, &disambiguate))
            .collect();
        Self { package }
    }

    /// Given a `Lock` loaded from a `Forc.lock` file, produce the graph of pinned dependencies.
    pub fn to_graph(&self) -> Result<pkg::Graph> {
        let mut graph = pkg::Graph::new();

        // Track the names which need to be disambiguated in the dependency list.
        let names = self.package.iter().map(|pkg| &pkg.name[..]);
        let disambiguate: HashSet<_> = names_requiring_disambiguation(names).collect();

        // Add all nodes to the graph.
        // Keep track of "<name> <source>" to node-index mappings for the edge collection pass.
        let mut pkg_to_node: HashMap<String, pkg::NodeIx> = HashMap::new();
        for pkg in &self.package {
            // Note: `key` may be either `<name> <source>` or just `<name>` if disambiguation not
            // required.
            let key = pkg.name_disambiguated(&disambiguate).into_owned();
            let name = pkg.name.clone();
            let source: pkg::SourcePinned = pkg.source.parse().map_err(|e| {
                anyhow!("invalid 'source' entry for package {} lock: {:?}", name, e)
            })?;
            let pkg = pkg::Pinned { name, source };
            let node = graph.add_node(pkg);
            pkg_to_node.insert(key, node);
        }

        // On the second pass, add all edges.
        for pkg in &self.package {
            let key = pkg.name_disambiguated(&disambiguate);
            let node = pkg_to_node[&key[..]];
            for dep_line in &pkg.dependencies {
                let (dep_name, dep_kind, dep_key) = parse_pkg_dep_line(dep_line)
                    .map_err(|e| anyhow!("failed to parse dependency \"{}\": {}", dep_line, e))?;
                let dep_node = pkg_to_node
                    .get(dep_key)
                    .cloned()
                    .ok_or_else(|| anyhow!("found dep {} without node entry in graph", dep_key))?;
                let dep_name = dep_name.unwrap_or(&graph[dep_node].name).to_string();
                let dep_edge = Edge::new(dep_name, dep_kind);
                graph.update_edge(node, dep_node, dep_edge);
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

/// Collect the set of package names that require disambiguation.
fn names_requiring_disambiguation<'a, I>(names: I) -> impl Iterator<Item = &'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut visited = BTreeSet::default();
    names.into_iter().filter(move |&name| !visited.insert(name))
}

fn pkg_name_disambiguated<'a>(name: &'a str, source: &'a str, disambiguate: bool) -> Cow<'a, str> {
    match disambiguate {
        true => Cow::Owned(pkg_unique_string(name, source)),
        false => Cow::Borrowed(name),
    }
}

fn pkg_unique_string(name: &str, source: &str) -> String {
    format!("{} {}", name, source)
}

fn pkg_dep_line(
    dep_name: Option<&str>,
    name: &str,
    source: &pkg::SourcePinned,
    dep_kind: &DepKind,
    disambiguate: bool,
) -> PkgDepLine {
    // Only include the full unique string in the case that this dep requires disambiguation.
    let source_string = source.to_string();
    let pkg_string = pkg_name_disambiguated(name, &source_string, disambiguate);
    // Prefix the dependency name if it differs from the package name.
    match dep_name {
        None => format!("{} {}", dep_kind.to_string(), pkg_string.into_owned()),
        Some(dep_name) => format!("({}) {} {}", dep_name, dep_kind.to_string(), pkg_string),
    }
}

type ParsedPkgLine<'a> = (Option<&'a str>, DepKind, &'a str);
// Parse the given `PkgDepLine` into its dependency name and unique string segments.
//
// I.e. given "(<dep_name>) <dep-kind> <name> <source>", returns ("<dep_name>", DepKind, "<name> <source>").
//
// Note that <source> may not appear in the case it is not required for disambiguation.
fn parse_pkg_dep_line(pkg_dep_line: &str) -> anyhow::Result<ParsedPkgLine> {
    let s = pkg_dep_line.trim();

    // Check for the open bracket.
    if !s.starts_with('(') {
        let dep_kind_str = s
            .split(' ')
            .next()
            .ok_or_else(|| anyhow!("missing dep kind"))?;
        let dep_kind = DepKind::from_str(dep_kind_str)?;
        let unique_pkg_str = &s[dep_kind_str.len()..].trim_start();
        return Ok((None, dep_kind, unique_pkg_str));
    }

    // If we have the open bracket, grab everything until the closing bracket.
    let s = &s["(".len()..];
    let mut iter = s.split(')');
    let dep_name = iter
        .next()
        .ok_or_else(|| anyhow!("missing closing parenthesis"))?;

    let s = &s[dep_name.len() + ")".len()..].trim_start();
    let dep_kind_str = s
        .split(' ')
        .next()
        .ok_or_else(|| anyhow!("missing dep kind"))?;
    let dep_kind = DepKind::from_str(dep_kind_str)?;
    // The rest is the unique package string.
    let unique_pkg_str = &s[dep_kind_str.len()..].trim_start();
    Ok((Some(dep_name), dep_kind, unique_pkg_str))
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
            let name = name_or_git_unique_string(pkg);
            println_red(&format!("  Removing {}", name));
        }
    }
}

pub fn print_added_pkgs<'a, I>(proj_name: &str, removed: I)
where
    I: IntoIterator<Item = &'a PkgLock>,
{
    for pkg in removed {
        if pkg.name != proj_name {
            let name = name_or_git_unique_string(pkg);
            println_green(&format!("    Adding {}", name));
        }
    }
}

// Only includes source after the name for git sources for friendlier printing.
fn name_or_git_unique_string(pkg: &PkgLock) -> Cow<str> {
    match pkg.source.starts_with(pkg::SourceGitPinned::PREFIX) {
        true => Cow::Owned(pkg.unique_string()),
        false => Cow::Borrowed(&pkg.name),
    }
}
