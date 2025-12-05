use crate::{pkg, source, DepKind, Edge};
use anyhow::{anyhow, Result};
use forc_diagnostic::{println_action_green, println_action_red};
use petgraph::{visit::EdgeRef, Direction};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    str::FromStr,
};
use sway_core::fuel_prelude::fuel_tx;

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
#[serde(rename_all = "kebab-case")]
pub struct PkgLock {
    pub(crate) name: String,
    // TODO: Cargo *always* includes version, whereas we don't even parse it when reading a
    // project's `Manifest` yet. If we decide to enforce versions, we'll want to remove the
    // `Option`.
    version: Option<semver::Version>,
    // Short-hand string describing where this package is sourced from.
    source: String,
    dependencies: Option<Vec<PkgDepLine>>,
    contract_dependencies: Option<Vec<PkgDepLine>>,
}

/// `PkgDepLine` is a terse, single-line, git-diff-friendly description of a package's
/// dependency. It is formatted like so:
///
/// ```ignore
/// (<dep_name>) <pkg_name> <source_string> (<salt>)
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
        let version = pinned.source.semver();
        let source = pinned.source.to_string();
        // Collection of all dependencies, so this includes both contract-dependencies and
        // lib-dependencies
        let all_dependencies: Vec<(String, DepKind)> = graph
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
                (
                    pkg_dep_line(
                        dep_name,
                        &dep_pkg.name,
                        &dep_pkg.source,
                        dep_kind,
                        disambiguate,
                    ),
                    dep_kind.clone(),
                )
            })
            .collect();
        let mut dependencies: Vec<String> = all_dependencies
            .iter()
            .filter_map(|(dep_pkg, dep_kind)| {
                (*dep_kind == DepKind::Library).then_some(dep_pkg.clone())
            })
            .collect();
        let mut contract_dependencies: Vec<String> = all_dependencies
            .iter()
            .filter_map(|(dep_pkg, dep_kind)| {
                matches!(*dep_kind, DepKind::Contract { .. }).then_some(dep_pkg.clone())
            })
            .collect();
        dependencies.sort();
        contract_dependencies.sort();

        let dependencies = if !dependencies.is_empty() {
            Some(dependencies)
        } else {
            None
        };

        let contract_dependencies = if !contract_dependencies.is_empty() {
            Some(contract_dependencies)
        } else {
            None
        };

        Self {
            name,
            version,
            source,
            dependencies,
            contract_dependencies,
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

/// Represents a `DepKind` before getting parsed.
///
/// Used to carry on the type of the `DepKind` until parsing. After parsing pkg_dep_line converted into `DepKind`.
enum UnparsedDepKind {
    Library,
    Contract,
}

impl Lock {
    /// Load the `Lock` structure from the TOML `Forc.lock` file at the specified path.
    pub fn from_path(path: &Path) -> Result<Self> {
        let string = fs::read_to_string(path)
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
            let source: source::Pinned = pkg.source.parse().map_err(|e| {
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
            // If `pkg.contract_dependencies` is None, we will be collecting an empty list of
            // contract_deps so that we will omit them during edge adding phase
            let contract_deps = pkg
                .contract_dependencies
                .as_ref()
                .into_iter()
                .flatten()
                .map(|contract_dep| (contract_dep, UnparsedDepKind::Contract));
            // If `pkg.dependencies` is None, we will be collecting an empty list of
            // lib_deps so that we will omit them during edge adding phase
            let lib_deps = pkg
                .dependencies
                .as_ref()
                .into_iter()
                .flatten()
                .map(|lib_dep| (lib_dep, UnparsedDepKind::Library));
            for (dep_line, dep_kind) in lib_deps.chain(contract_deps) {
                let (dep_name, dep_key, dep_salt) = parse_pkg_dep_line(dep_line)
                    .map_err(|e| anyhow!("failed to parse dependency \"{}\": {}", dep_line, e))?;
                let dep_node = pkg_to_node
                    .get(dep_key)
                    .copied()
                    .ok_or_else(|| anyhow!("found dep {} without node entry in graph", dep_key))?;
                let dep_name = dep_name.unwrap_or(&graph[dep_node].name).to_string();
                let dep_kind = match dep_kind {
                    UnparsedDepKind::Library => DepKind::Library,
                    UnparsedDepKind::Contract => {
                        let dep_salt = dep_salt.unwrap_or_default();
                        DepKind::Contract { salt: dep_salt }
                    }
                };
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
    format!("{name} {source}")
}

fn pkg_dep_line(
    dep_name: Option<&str>,
    name: &str,
    source: &source::Pinned,
    dep_kind: &DepKind,
    disambiguate: bool,
) -> PkgDepLine {
    // Only include the full unique string in the case that this dep requires disambiguation.
    let source_string = source.to_string();
    let pkg_string = pkg_name_disambiguated(name, &source_string, disambiguate);
    // Prefix the dependency name if it differs from the package name.
    let pkg_string = match dep_name {
        None => pkg_string.into_owned(),
        Some(dep_name) => format!("({dep_name}) {pkg_string}"),
    };
    // Append the salt if dep_kind is DepKind::Contract.
    match dep_kind {
        DepKind::Library => pkg_string,
        DepKind::Contract { salt } => {
            if *salt == fuel_tx::Salt::zeroed() {
                pkg_string
            } else {
                format!("{pkg_string} ({salt})")
            }
        }
    }
}

type ParsedPkgLine<'a> = (Option<&'a str>, &'a str, Option<fuel_tx::Salt>);
// Parse the given `PkgDepLine` into its dependency name and unique string segments.
//
// I.e. given "(<dep_name>) <name> <source> (<salt>)", returns ("<dep_name>", "<name> <source>", "<salt>").
//
// Note that <source> may not appear in the case it is not required for disambiguation.
fn parse_pkg_dep_line(pkg_dep_line: &str) -> anyhow::Result<ParsedPkgLine> {
    let s = pkg_dep_line.trim();
    let (dep_name, s) = match s.starts_with('(') {
        false => (None, s),
        true => {
            // If we have the open bracket, grab everything until the closing bracket.
            let s = &s["(".len()..];
            let mut iter = s.split(')');
            let dep_name = iter
                .next()
                .ok_or_else(|| anyhow!("missing closing parenthesis"))?;
            // The rest is the unique package string and possibly the salt.
            let s = &s[dep_name.len() + ")".len()..];
            (Some(dep_name), s)
        }
    };

    // Check for salt.
    let mut iter = s.split('(');
    let pkg_str = iter
        .next()
        .ok_or_else(|| anyhow!("missing pkg string"))?
        .trim();
    let salt_str = iter.next().map(|s| s.trim()).map(|s| &s[..s.len() - 1]);
    let salt = match salt_str {
        Some(salt_str) => Some(
            fuel_tx::Salt::from_str(salt_str)
                .map_err(|e| anyhow!("invalid salt in lock file: {e}"))?,
        ),
        None => None,
    };

    Ok((dep_name, pkg_str, salt))
}

pub fn print_diff(member_names: &HashSet<String>, diff: &Diff) {
    print_removed_pkgs(member_names, diff.removed.iter().copied());
    print_added_pkgs(member_names, diff.added.iter().copied());
}

pub fn print_removed_pkgs<'a, I>(member_names: &HashSet<String>, removed: I)
where
    I: IntoIterator<Item = &'a PkgLock>,
{
    for pkg in removed {
        if !member_names.contains(&pkg.name) {
            let src = match pkg.source.starts_with(source::git::Pinned::PREFIX) {
                true => format!(" {}", pkg.source),
                false => String::new(),
            };
            println_action_red(
                "Removing",
                &format!("{}{src}", ansiterm::Style::new().bold().paint(&pkg.name)),
            );
        }
    }
}

pub fn print_added_pkgs<'a, I>(member_names: &HashSet<String>, removed: I)
where
    I: IntoIterator<Item = &'a PkgLock>,
{
    for pkg in removed {
        if !member_names.contains(&pkg.name) {
            let src = match pkg.source.starts_with(source::git::Pinned::PREFIX) {
                true => format!(" {}", pkg.source),
                false => "".to_string(),
            };
            println_action_green(
                "Adding",
                &format!("{}{src}", ansiterm::Style::new().bold().paint(&pkg.name)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use sway_core::fuel_prelude::fuel_tx;

    use super::parse_pkg_dep_line;

    #[test]
    fn test_parse_pkg_line_with_salt_with_dep_name() {
        let pkg_dep_line = "(std2) std path+from-root (0000000000000000000000000000000000000000000000000000000000000000)";
        let (dep_name, pkg_string, salt) = parse_pkg_dep_line(pkg_dep_line).unwrap();
        assert_eq!(salt, Some(fuel_tx::Salt::zeroed()));
        assert_eq!(dep_name, Some("std2"));
        assert_eq!(pkg_string, "std path+from-root");
    }

    #[test]
    fn test_parse_pkg_line_with_salt_without_dep_name() {
        let pkg_dep_line =
            "std path+from-root (0000000000000000000000000000000000000000000000000000000000000000)";
        let (dep_name, pkg_string, salt) = parse_pkg_dep_line(pkg_dep_line).unwrap();
        assert_eq!(salt, Some(fuel_tx::Salt::zeroed()));
        assert_eq!(dep_name, None);
        assert_eq!(pkg_string, "std path+from-root");
    }

    #[test]
    fn test_parse_pkg_line_without_salt_with_dep_name() {
        let pkg_dep_line = "(std2) std path+from-root";
        let (dep_name, pkg_string, salt) = parse_pkg_dep_line(pkg_dep_line).unwrap();
        assert_eq!(salt, None);
        assert_eq!(dep_name, Some("std2"));
        assert_eq!(pkg_string, "std path+from-root");
    }

    #[test]
    fn test_parse_pkg_line_without_salt_without_dep_name() {
        let pkg_dep_line = "std path+from-root";
        let (dep_name, pkg_string, salt) = parse_pkg_dep_line(pkg_dep_line).unwrap();
        assert_eq!(salt, None);
        assert_eq!(dep_name, None);
        assert_eq!(pkg_string, "std path+from-root");
    }

    #[test]
    #[should_panic]
    fn test_parse_pkg_line_invalid_salt() {
        let pkg_dep_line = "std path+from-root (1)";
        parse_pkg_dep_line(pkg_dep_line).unwrap();
    }
}
