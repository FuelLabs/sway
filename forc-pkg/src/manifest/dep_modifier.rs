use super::PackageManifest;
use crate::manifest::{
    ContractDependency, Dependency, DependencyDetails, GenericManifestFile, HexSalt,
};
use crate::source::IPFSNode;
use crate::{self as pkg, Lock, PackageManifestFile};
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

const DEPS: &str = "dependencies";
const CONTRACT_DEPS: &str = "contract-dependencies";

#[derive(Clone, Debug, Default)]
pub enum Action {
    #[default]
    Add,
    Remove,
}

#[derive(Clone, Debug, Default)]
pub struct ModifyOpts {
    // === Manifest Options ===
    pub manifest_path: Option<String>,
    // === Package Selection ===
    pub package: Option<String>,
    // === Source (Add only) ===
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
    pub action: Action,
}

pub fn modify_dependencies(opts: ModifyOpts) -> Result<()> {
    let cwd = if let Some(p) = &opts.manifest_path {
        PathBuf::from(p)
    } else {
        std::env::current_dir()?
    };

    let manifest_file = ManifestFile::from_dir(&cwd)?;
    let root_dir = manifest_file.root_dir();
    let mut member_manifests = manifest_file.member_manifests()?;

    let package_spec_dir =
        resolve_package_path(&manifest_file, &opts.package, &root_dir, &member_manifests)?;

    let content = std::fs::read_to_string(&package_spec_dir)?;
    let mut toml_doc = content.parse::<DocumentMut>()?;
    let backup_doc = toml_doc.clone();

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

    match opts.action {
        Action::Add => {
            for dependency in &opts.dependencies {
                let (dep_name, dependency_data) =
                    resolve_dependency(dependency, &opts, &member_manifests, package_spec.dir())?;
                section.insert_dep(dep_name, dependency_data, opts.salt.clone())?;
            }

            section.add_deps_manifest_table(
                &mut toml_doc,
                package_spec_dir.clone(),
                &mut package_spec.manifest,
            )?;
        }
        Action::Remove => {
            let dep_refs: Vec<&str> = opts.dependencies.iter().map(String::as_str).collect();

            section.remove_deps_manifest_table(
                &mut toml_doc,
                package_spec_dir.clone(),
                &mut package_spec.manifest,
                &dep_refs,
            )?;
        }
    }

    member_manifests.insert(package_spec.project_name().to_string(), package_spec);

    let new_plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        false,
        opts.offline,
        &opts.ipfs_node.clone().unwrap_or_default(),
    );

    let new_plan = new_plan.or_else(|e| {
        std::fs::write(&package_spec_dir, backup_doc.to_string())
            .map_err(|write_err| anyhow!("failed to write toml file: {}", write_err))?;
        Err(e)
    })?;

    let new_lock = Lock::from_graph(new_plan.graph());

    new_lock.diff(&old_lock);

    if opts.dry_run {
        info!("Dry run enabled. toml file not modified.");
        std::fs::write(cwd, backup_doc.to_string())?;
        return Ok(());
    }

    let string = toml::ser::to_string_pretty(&new_lock)
        .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;

    if let Err(e) = fs::write(&lock_path, string) {
        std::fs::write(&package_spec_dir, backup_doc.to_string())
            .map_err(|e| anyhow!("failed to write toml file: {}", e))?;
        bail!("failed to write lock file: {}", e);
    };

    Ok(())
}

fn resolve_package_path(
    manifest_file: &ManifestFile,
    package: &Option<String>,
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

        resolve_workspace_path_inner(member_manifests, package_name, root_dir)
    } else if let Some(package_name) = package {
        resolve_workspace_path_inner(member_manifests, package_name, root_dir)
    } else {
        Ok(manifest_file.path().to_path_buf())
    }
}

fn resolve_workspace_path_inner(
    member_manifests: &BTreeMap<String, PackageManifestFile>,
    package_name: &str,
    root_dir: &Path,
) -> Result<PathBuf> {
    if let Some(dir) = member_manifests.get(package_name) {
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
    opts: &ModifyOpts,
    member_manifests: &BTreeMap<String, PackageManifestFile>,
    package_dir: P,
) -> Result<(String, Dependency)> {
    let dep_spec: DepSpec = raw.parse()?;
    let dep_name = dep_spec
        .name
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
                if member.dir() == package_dir.as_ref() {
                    bail!("cannot add `{}` as a dependency to itself", dep_name);
                }
                let sibling_parent = package_dir.as_ref().parent().unwrap();
                let rel_path = member
                    .dir()
                    .strip_prefix(sibling_parent)
                    .map(|p| PathBuf::from("..").join(p))
                    .unwrap_or_else(|_| member.dir().to_path_buf());
                details.path = Some(rel_path.to_string_lossy().to_string());
            }
        }
        Dependency::Detailed(details)
    };

    Ok((dep_name, dependency_data))
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

        let mut s = s.split('@');
        let Some(name) = s.next() else {
            bail!("dependency name is missing");
        };

        let dep = &mut DepSpec::default();
        dep.name = Some(name.parse()?);

        let Some(version_req) = s.next() else {
            return Ok(dep.clone());
        };

        dep.version_req = Some(version_req.parse()?);
        Ok(dep.clone())
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
    pub fn insert_dep(
        &mut self,
        name: String,
        data: Dependency,
        salt: Option<String>,
    ) -> Result<()> {
        match self {
            DepSection::Regular(map) => {
                map.insert(name, data);
            }
            DepSection::Contract(map, salt_opt) => {
                let resolved_salt = match salt.as_ref().or(salt_opt.as_ref()) {
                    Some(s) => {
                        HexSalt::from_str(s).map_err(|e| anyhow!("Invalid salt format: {}", e))?
                    }
                    None => HexSalt(fuel_tx::Salt::default()),
                };
                let contract_dep = ContractDependency {
                    dependency: data,
                    salt: resolved_salt,
                };
                map.insert(name, contract_dep);
            }
        }
        Ok(())
    }

    pub fn remove_deps_manifest_table<P: AsRef<Path>>(
        &mut self,
        doc: &mut DocumentMut,
        manifest_path: P,
        manifest: &mut PackageManifest,
        deps: &[&str],
    ) -> Result<()> {
        match self {
            DepSection::Regular(ref mut map) => {
                for dep in deps {
                    if !map.contains_key(*dep) {
                        bail!("the dependency `{}` could not be found in `{}`", dep, DEPS);
                    }
                    map.remove(*dep);
                }
                manifest.dependencies = Some(map.clone());
                remove_deps_from_table(doc, manifest_path, DEPS, deps)?;
            }
            DepSection::Contract(ref mut map, _) => {
                for dep in deps {
                    if !map.contains_key(*dep) {
                        bail!(
                            "the dependency `{}` could not be found in `{}`",
                            dep,
                            CONTRACT_DEPS
                        );
                    }
                    map.remove(*dep);
                }
                manifest.contract_dependencies = Some(map.clone());
                remove_deps_from_table(doc, manifest_path, CONTRACT_DEPS, deps)?;
            }
        };
        Ok(())
    }

    pub fn add_deps_manifest_table<P: AsRef<Path>>(
        self,
        doc: &mut DocumentMut,
        manifest_path: P,
        manifest: &mut PackageManifest,
    ) -> Result<()> {
        let path = manifest_path.as_ref();
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
                (DEPS, table)
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
                (CONTRACT_DEPS, table)
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
    doc: &mut DocumentMut,
    manifest_path: P,
    section: &str,
    deps: &[&str],
) -> Result<()> {
    let path = manifest_path.as_ref();
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

    std::fs::write(path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::DepSpec;
    use std::str::FromStr;
    #[test]
    fn dep_from_str_name_only() {
        let dep: DepSpec = "abc".parse().expect("parsing dep spec failed");
        assert_eq!(dep.name, Some("abc".to_string()));
        assert_eq!(dep.version_req, None);
    }

    #[test]
    fn dep_from_str_name_and_version() {
        let dep: DepSpec = "abc@1".parse().expect("parsing dep spec failed");
        assert_eq!(dep.name, Some("abc".to_string()));
        assert_eq!(dep.version_req, Some("1".to_string()));
    }

    #[test]
    fn dep_from_str_invalid() {
        assert!(DepSpec::from_str("").is_err());
    }
}
