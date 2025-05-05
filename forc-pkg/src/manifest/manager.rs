use crate::manifest::{
    ContractDependency, Dependency, DependencyDetails, GenericManifestFile, HexSalt,
};
use crate::source::IPFSNode;
use crate::{self as pkg, lock, Lock, PackageManifestFile};
use anyhow::{anyhow, bail, Result};
use pkg::manifest::ManifestFile;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use sway_core::fuel_prelude::fuel_tx;
use toml_edit::{value, DocumentMut, InlineTable, Item, Table, Value};
use tracing::info;

use super::PackageManifest;

#[derive(Clone, Debug, Default)]
pub struct AddOpts {
    // === Manifest Options ===
    pub manifest_path: Option<String>,

    // === Package Selection ===
    pub package: Option<String>,

    // === Source ===
    pub source_path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub ipfs: Option<String>,

    // === Section ===
    pub contract_deps: bool,
    pub salt: Option<String>,

    // === IPFS Node ===
    pub ipfs_node: Option<IPFSNode>,

    // === Dependencies & Flags ===
    pub dependencies: Vec<String>,
    pub dry_run: bool,
    pub offline: bool,
}

#[derive(Clone, Debug, Default)]
pub struct RemoveOpts {
    // === Manifest Options ===
    pub manifest_path: Option<String>,

    // === Package Selection ===
    pub package: Option<String>,

    // === Section ===
    pub contract_deps: bool,
    pub salt: Option<String>,

    // === IPFS Node ===
    pub ipfs_node: Option<IPFSNode>,

    // === Dependencies & Flags ===
    pub dependencies: Vec<String>,
    pub dry_run: bool,
    pub offline: bool,
}

pub fn add_dependencies(opts: AddOpts) -> Result<()> {
    let dry_run = opts.dry_run;

    // get manifest path
    let this_dir = match opts.manifest_path.clone() {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };

    let manifest_file = ManifestFile::from_dir(&this_dir)?;
    let root_dir = manifest_file.root_dir();
    let mut member_manifests = manifest_file.member_manifests()?;

    let package_spec_dir = resolve_package_path(
        &manifest_file,
        opts.package.clone(),
        &root_dir,
        &member_manifests,
    )?;

    let mut package_spec = PackageManifestFile::from_file(&package_spec_dir)?;

    let lock_path = package_spec.lock_path()?;
    let old_lock = Lock::from_path(&lock_path).ok().unwrap_or_default();

    let mut deps_regular = package_spec
        .manifest
        .dependencies
        .take()
        .unwrap_or_default();
    let mut deps_contract = package_spec
        .manifest
        .contract_dependencies
        .take()
        .unwrap_or_default();

    let mut section = if opts.contract_deps {
        DepSection::Contract(&mut deps_contract, opts.salt.clone())
    } else {
        DepSection::Regular(&mut deps_regular)
    };

    for dependency in &opts.dependencies {
        let (dep_name, dependency_data) =
            resolve_dependency(dependency, &opts, &member_manifests, package_spec.dir())?;
        section.insert_dep(dep_name, dependency_data, opts.salt.clone());
    }

    section.add_deps_manifest_table(package_spec_dir, &mut package_spec.manifest)?;

    update_lock_file(
        &mut member_manifests,
        &package_spec,
        opts.ipfs_node,
        dry_run,
        old_lock,
        lock_path,
        opts.offline,
    )?;

    Ok(())
}

pub fn remove_dependencies(opts: RemoveOpts) -> Result<()> {
    let dry_run = opts.dry_run;
    let package_name = opts.package.clone();

    // get manifest path
    let this_dir = match opts.manifest_path.clone() {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };

    let manifest_file = ManifestFile::from_dir(&this_dir)?;
    let root_dir = manifest_file.root_dir();
    let mut member_manifests = manifest_file.member_manifests()?;

    let package_spec_dir =
        resolve_package_path(&manifest_file, package_name, &root_dir, &member_manifests)?;

    let mut package_spec = PackageManifestFile::from_file(&package_spec_dir)?;

    let lock_path = package_spec.lock_path()?;
    let old_lock = Lock::from_path(&lock_path).unwrap_or_default();

    let mut deps_regular = package_spec
        .manifest
        .dependencies
        .take()
        .unwrap_or_default();
    let mut deps_contract = package_spec
        .manifest
        .contract_dependencies
        .take()
        .unwrap_or_default();

    let mut section = if opts.contract_deps {
        DepSection::Contract(&mut deps_contract, opts.salt.clone())
    } else {
        DepSection::Regular(&mut deps_regular)
    };

    let dep_refs: Vec<&str> = opts.dependencies.iter().map(String::as_str).collect();

    section.remove_deps_manifest_table(package_spec_dir, &mut package_spec.manifest, &dep_refs)?;

    update_lock_file(
        &mut member_manifests,
        &package_spec,
        opts.ipfs_node,
        dry_run,
        old_lock,
        lock_path,
        opts.offline,
    )?;

    Ok(())
}

fn resolve_package_path(
    manifest_file: &ManifestFile,
    package: Option<String>,
    root_dir: &Path,
    member_manifests: &BTreeMap<String, PackageManifestFile>,
) -> Result<PathBuf> {
    if manifest_file.is_workspace() {
        let Some(package_name) = package else {
            let packages = member_manifests
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            bail!("`forc add` could not determine which package to modify. Use --package.\nAvailable: {}", packages);
        };

        resolve_workspace_path_inner(member_manifests, &package_name, root_dir)
    } else if let Some(package_name) = package {
        resolve_workspace_path_inner(member_manifests, &package_name, root_dir)
    } else {
        Ok(manifest_file.path().to_path_buf())
    }
}

fn resolve_workspace_path_inner(
    member_manifests: &BTreeMap<String, PackageManifestFile>,
    package_name: &str,
    root_dir: &Path,
) -> Result<PathBuf> {
    if member_manifests.contains_key(package_name) {
        let dir = member_manifests.get(package_name).unwrap();
        Ok(dir.path().to_path_buf())
    } else {
        anyhow::bail!(
            "package(s) {} not found in workspace {}",
            package_name,
            root_dir.to_string_lossy()
        )
    }
}

fn resolve_dependency<P: AsRef<Path>>(
    raw: &str,
    opts: &AddOpts,
    member_manifests: &BTreeMap<String, PackageManifestFile>,
    package_dir: P,
) -> Result<(String, Dependency)> {
    let dep_spec: DepSpec = raw.parse()?;
    let dep_name = dep_spec
        .name
        .clone()
        .ok_or_else(|| anyhow!("Missing name in `{}`", raw))?;

    let mut details = DependencyDetails {
        version: dep_spec.version_req.clone(),
        namespace: None,
        path: opts.source_path.clone(),
        git: opts.git.clone(),
        branch: opts.branch.clone(),
        tag: opts.tag.clone(),
        package: None,
        rev: opts.rev.clone(),
        ipfs: opts.ipfs.clone(),
    };

    details.validate()?;

    let dependency_data = if let Some(version) = dep_spec.version_req {
        Dependency::Simple(version)
    } else {
        if details.is_source_empty() {
            if let Some(member) = member_manifests.get(&dep_name) {
                let rel_path = pathdiff::diff_paths(member.dir(), package_dir)
                    .unwrap_or_else(|| member.path().to_path_buf());
                details.path = Some(rel_path.to_string_lossy().to_string());
            }
        }
        info!("{:?}", details);
        Dependency::Detailed(details)
    };

    Ok((dep_name, dependency_data))
}

fn update_lock_file(
    member_manifests: &mut BTreeMap<String, PackageManifestFile>,
    package_spec: &PackageManifestFile,
    ipfs_node: Option<IPFSNode>,
    dry_run: bool,
    old_lock: Lock,
    lock_path: PathBuf,
    offline: bool,
) -> Result<()> {
    member_manifests.insert(
        package_spec.project_name().to_string(),
        package_spec.clone(),
    );

    if dry_run {
        info!("Dry run enabled. Lock file not modified.");
        return Ok(());
    }

    let new_plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        member_manifests,
        false,
        offline,
        &ipfs_node.clone().unwrap_or_default(),
    )?;

    let new_lock = Lock::from_graph(new_plan.graph());
    let diff = new_lock.diff(&old_lock);
    let member_names = member_manifests
        .values()
        .map(|m| m.project.name.clone())
        .collect();

    lock::print_diff(&member_names, &diff);

    Ok(())
}

/// Reference to a package to be added as a dependency.
///
/// See `forc add` help for more info.
#[derive(Clone, Debug, Default)]
pub struct DepSpec {
    pub name: Option<String>,
    pub version_req: Option<String>,
}

impl FromStr for DepSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s.is_empty() {
            bail!("dependency spec cannot be empty");
        }

        let mut dep = DepSpec::default();

        let mut s = s.split('@');
        let Some(name) = s.next() else {
            bail!("dependency name is missing");
        };

        dep.name = Some(name.parse()?);

        let Some(version_req) = s.next() else {
            return Ok(dep);
        };

        dep.version_req = Some(version_req.parse()?);

        Ok(dep)
    }
}

impl fmt::Display for DepSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name}")?;
        }

        if let Some(version_req) = &self.version_req {
            write!(f, "@{version_req}")?;
        }

        Ok(())
    }
}

pub enum DepSection<'a> {
    Regular(&'a mut BTreeMap<String, Dependency>),
    Contract(&'a mut BTreeMap<String, ContractDependency>, Option<String>),
}

impl DepSection<'_> {
    pub fn insert_dep(&mut self, name: String, data: Dependency, salt: Option<String>) {
        match self {
            DepSection::Regular(map) => {
                map.insert(name, data);
            }
            DepSection::Contract(map, salt_opt) => {
                let resolved_salt = salt
                    .or_else(|| salt_opt.clone())
                    .map(|s| HexSalt::from_str(&s).unwrap())
                    .unwrap_or_else(|| HexSalt(fuel_tx::Salt::default()));

                let contract_dep = ContractDependency {
                    dependency: data,
                    salt: resolved_salt,
                };

                map.insert(name, contract_dep);
            }
        }
    }

    pub fn remove_deps_manifest_table<P: AsRef<Path>>(
        &mut self,
        manifest_path: P,
        manifest: &mut PackageManifest,
        deps: &[&str],
    ) -> Result<()> {
        match self {
            DepSection::Regular(ref mut map) => {
                for dep in deps {
                    map.remove(*dep);
                }
                manifest.dependencies = Some(map.clone());
                remove_deps_from_table(manifest_path, "dependencies", deps)?;
            }
            DepSection::Contract(ref mut map, _) => {
                for dep in deps {
                    map.remove(*dep);
                }
                manifest.contract_dependencies = Some(map.clone());
                remove_deps_from_table(manifest_path, "contract-dependencies", deps)?;
            }
        };
        Ok(())
    }

    pub fn add_deps_manifest_table<P: AsRef<Path>>(
        self,
        manifest_path: P,
        manifest: &mut PackageManifest,
    ) -> Result<()> {
        let path = manifest_path.as_ref();

        let content =
            fs::read_to_string(path).map_err(|e| anyhow!("failed to read manifest: {e}"))?;

        let mut doc = content
            .parse::<DocumentMut>()
            .map_err(|e| anyhow!("failed to parse TOML: {e}"))?;

        let (section_name, new_table): (&str, Table) = match self {
            DepSection::Regular(deps) => {
                manifest.dependencies = Some(deps.clone());
                let mut table = Table::new();
                for (name, dep) in deps {
                    let item = match dep {
                        Dependency::Simple(ver) => ver.to_string().into(),
                        Dependency::Detailed(details) => {
                            let inline = generate_table(details);
                            Item::Value(toml_edit::Value::InlineTable(inline))
                        }
                    };
                    table.insert(name, item);
                }
                ("dependencies", table)
            }
            DepSection::Contract(deps, salt_hex) => {
                let mut table = Table::new();
                manifest.contract_dependencies = Some(deps.clone());
                for (name, contract_dep) in deps {
                    let dep = &contract_dep.dependency;

                    let item = match dep {
                        Dependency::Simple(ver) => value(ver),
                        Dependency::Detailed(details) => {
                            let mut inline = generate_table(details);
                            if let Some(salt) = &salt_hex {
                                inline.insert("salt", salt.to_string().into());
                            }
                            Item::Value(toml_edit::Value::InlineTable(inline))
                        }
                    };
                    table.insert(name, item);
                }
                ("contract-dependencies", table)
            }
        };
        doc[section_name] = Item::Table(new_table);
        fs::write(path, doc.to_string())?;
        Ok(())
    }
}

fn generate_table(details: &DependencyDetails) -> InlineTable {
    let mut inline = InlineTable::default();

    if let Some(version) = &details.version {
        inline.insert("version", Value::from(version.to_string()));
    }
    if let Some(git) = &details.git {
        inline.insert("git", Value::from(git.to_string()));
    }
    if let Some(branch) = &details.branch {
        inline.insert("branch", Value::from(branch.to_string()));
    }
    if let Some(tag) = &details.tag {
        inline.insert("tag", Value::from(tag.to_string()));
    }
    if let Some(rev) = &details.rev {
        inline.insert("rev", Value::from(rev.to_string()));
    }
    if let Some(path) = &details.path {
        inline.insert("path", Value::from(path.to_string()));
    }
    if let Some(ipfs) = &details.ipfs {
        inline.insert("cid", Value::from(ipfs.to_string()));
    }

    inline
}

fn remove_deps_from_table<P: AsRef<Path>>(
    manifest_path: P,
    section: &str,
    deps: &[&str],
) -> Result<()> {
    let content = std::fs::read_to_string(&manifest_path)?;
    let mut doc = content.parse::<DocumentMut>()?;

    let section_table = doc[section].as_table_mut().ok_or_else(|| {
        anyhow!(
            "section [{}] not found in manifest: {}",
            section,
            manifest_path.as_ref().display()
        )
    })?;

    for dep in deps {
        section_table.remove(dep);
    }

    std::fs::write(manifest_path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::DepSpec;
    use test_case::test_case;
    #[test_case("abc", Some("abc"), None)]
    #[test_case("abc@1", Some("abc"), Some("1"))]
    fn dep_is_from_str_valid(s: &str, expected_name: Option<&str>, expected_version: Option<&str>) {
        let dep: DepSpec = s.parse().expect("parsing dep spec failed");
        assert_eq!(
            (
                dep.name.map(|p| p.to_string()),
                dep.version_req.map(|p| p.to_string())
            ),
            (
                expected_name.map(|n| n.to_string()),
                expected_version.map(|v| v.to_string())
            ),
        );
    }

    #[test]
    fn dep_is_from_str_invalid() {
        assert!(DepSpec::from_str("").is_err());
    }
}
