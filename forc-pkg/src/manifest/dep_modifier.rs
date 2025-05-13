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
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};
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

            section.add_deps_manifest_table(&mut toml_doc, &mut package_spec.manifest);
        }
        Action::Remove => {
            let dep_refs: Vec<&str> = opts.dependencies.iter().map(String::as_str).collect();

            section.remove_deps_manifest_table(
                &mut toml_doc,
                &mut package_spec.manifest,
                &dep_refs,
            )?;
        }
    }

    // write updates to toml doc
    std::fs::write(&package_spec_dir, toml_doc.to_string())?;

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
                    salt: resolved_salt.clone(),
                };
                map.insert(name, contract_dep);
            }
        }
        Ok(())
    }

    pub fn remove_deps_manifest_table(
        &mut self,
        doc: &mut DocumentMut,
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
                remove_deps_from_table(doc, DEPS, deps)?;
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
                remove_deps_from_table(doc, CONTRACT_DEPS, deps)?;
            }
        };
        Ok(())
    }

    pub fn add_deps_manifest_table(self, doc: &mut DocumentMut, manifest: &mut PackageManifest) {
        let (section_name, table) = match self {
            DepSection::Regular(deps) => {
                manifest.dependencies = Some(deps.clone());
                let mut table = Table::new();
                for (name, dep) in deps.iter() {
                    let item = match dep {
                        Dependency::Simple(ver) => ver.to_string().into(),
                        Dependency::Detailed(details) => {
                            Item::Value(toml_edit::Value::InlineTable(generate_table(details)))
                        }
                    };
                    table.insert(name, item);
                }
                (DEPS, table)
            }
            DepSection::Contract(deps, _) => {
                manifest.contract_dependencies = Some(deps.clone());
                let mut table = Table::new();
                for (name, contract_dep) in deps {
                    let dep = &contract_dep.dependency;
                    let salt = &contract_dep.salt;
                    let item = match dep {
                        Dependency::Simple(ver) => {
                            let mut inline = InlineTable::default();
                            inline.insert("version", ver.to_string().into());
                            inline.insert("salt", salt.to_string().into());
                            Item::Value(toml_edit::Value::InlineTable(inline))
                        }
                        Dependency::Detailed(details) => {
                            let mut inline = generate_table(details);
                            inline.insert("salt", salt.to_string().into());
                            Item::Value(toml_edit::Value::InlineTable(inline))
                        }
                    };
                    table.insert(name, item);
                }
                (CONTRACT_DEPS, table)
            }
        };

        doc[section_name] = Item::Table(table);
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

fn remove_deps_from_table(doc: &mut DocumentMut, section: &str, deps: &[&str]) -> Result<()> {
    let section_table = doc[section]
        .as_table_mut()
        .ok_or_else(|| anyhow!("section [{}] not found in manifest", section))?;

    for dep in deps {
        section_table.remove(dep);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn get_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(relative_path)
            .canonicalize()
            .expect("failed to resolve path")
    }

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

    #[test]
    fn test_resolve_package_path_single_package_mode() {
        let dir = PathBuf::from("./tests/test_package");
        let expected_path = get_path("./tests/test_package/Forc.toml");

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();
        let result = resolve_package_path(&manifest_file, &None, &root_dir, &members).unwrap();

        assert_eq!(result, expected_path);
    }

    #[test]
    fn test_resolve_package_path_workspace_with_package_found() {
        let dir = PathBuf::from("./tests/test_workspace");
        let expected_path = get_path("./tests/test_workspace/package-1/Forc.toml");

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();

        let package = "package-1".to_string();
        let result =
            resolve_package_path(&manifest_file, &Some(package), &root_dir, &members).unwrap();

        assert_eq!(result, expected_path);
    }

    #[test]
    fn test_resolve_package_path_workspace_package_not_found() {
        let dir = PathBuf::from("./tests/test_workspace");

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();

        let err = resolve_package_path(
            &manifest_file,
            &Some("missing_pkg".into()),
            &root_dir,
            &members,
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("package(s) missing_pkg not found"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_resolve_package_path_workspace_package_not_set() {
        let dir = PathBuf::from("./tests/test_workspace");

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();

        let err = resolve_package_path(&manifest_file, &None, &root_dir, &members).unwrap_err();

        let resp = "`forc add` could not determine which package to modify. Use --package.\nAvailable: package-1, package-2".to_string();
        assert!(err.to_string().contains(&resp), "unexpected error: {err}");
    }

    #[test]
    fn test_resolve_dependency_simple_version() {
        let opts = ModifyOpts {
            dependencies: vec!["dep@1.0.0".to_string()],
            ..Default::default()
        };

        let dir = PathBuf::from("./tests/test_package");

        let package_spec_dir = get_path("./tests/test_package");

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let (name, data) =
            resolve_dependency("dep@1.0.0", &opts, &members, package_spec_dir).unwrap();

        assert_eq!(name, "dep");
        match data {
            Dependency::Simple(v) => assert_eq!(v, "1.0.0"),
            _ => panic!("Expected simple dependency"),
        }
    }

    #[test]
    fn test_resolve_dependency_detailed_variants() {
        let base_opts = ModifyOpts {
            ..Default::default()
        };

        let package_spec_dir = get_path("./tests/test_package");
        let manifest_file = ManifestFile::from_dir(&package_spec_dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let dep = "dummy_dep";
        let git = "https://github.com/example/repo.git";

        // Git + branch
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.branch = Some("main".to_string());

            let (name, data) = resolve_dependency(dep, &opts, &members, &package_spec_dir).unwrap();
            assert_eq!(name, dep);
            match data {
                Dependency::Detailed(details) => {
                    assert_eq!(details.git.as_deref(), Some(git));
                    assert_eq!(details.branch.as_deref(), Some("main"));
                }
                _ => panic!("Expected detailed dependency with git+branch"),
            }
        }

        // Git + rev
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.rev = Some("deadbeef".to_string());

            let (name, data) = resolve_dependency(dep, &opts, &members, &package_spec_dir).unwrap();
            assert_eq!(name, dep);
            match data {
                Dependency::Detailed(details) => {
                    assert_eq!(details.git.as_deref(), Some(git));
                    assert_eq!(details.rev.as_deref(), Some("deadbeef"));
                }
                _ => panic!("Expected detailed dependency with git+rev"),
            }
        }

        // Git + tag
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.tag = Some("v1.2.3".to_string());

            let (name, data) = resolve_dependency(dep, &opts, &members, &package_spec_dir).unwrap();
            assert_eq!(name, dep);
            match data {
                Dependency::Detailed(details) => {
                    assert_eq!(details.git.as_deref(), Some(git));
                    assert_eq!(details.tag.as_deref(), Some("v1.2.3"));
                }
                _ => panic!("Expected detailed dependency with git+tag"),
            }
        }

        // dep + ipfs
        {
            let mut opts = base_opts.clone();
            opts.ipfs = Some("QmYwAPJzv5CZsnA".to_string());

            let (name, data) = resolve_dependency(dep, &opts, &members, &package_spec_dir).unwrap();
            assert_eq!(name, dep);
            match data {
                Dependency::Detailed(details) => {
                    assert_eq!(details.ipfs.as_deref(), Some("QmYwAPJzv5CZsnA"));
                }
                _ => panic!("Expected detailed dependency with git+tag"),
            }
        }
    }

    #[test]
    fn test_resolve_dependency_from_workspace_sibling() {
        let dir = PathBuf::from("./tests/test_workspace");
        let package_dir = get_path("./tests/test_workspace/package-2");
        let dep = "package-1";

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let opts = ModifyOpts {
            source_path: None,
            dependencies: vec![dep.to_string()],
            package: Some("package-2".to_string()),
            ..Default::default()
        };

        let (name, data) =
            resolve_dependency(dep, &opts, &members, &package_dir).expect("should resolve");

        assert_eq!(name, dep);
        match data {
            Dependency::Detailed(details) => {
                assert!(details.path.is_some());
                let actual_path = details.path.as_ref().unwrap();
                assert_eq!(actual_path, "../package-1");
            }
            _ => panic!("Expected detailed dependency with fallback path"),
        }
    }

    #[test]
    fn test_resolve_dependency_self_dependency_error() {
        let dir = PathBuf::from("./tests/test_workspace");
        let package_dir = get_path("./tests/test_workspace/package-1");
        let dep = "package-1";
        let resp = format!("cannot add `{}` as a dependency to itself", dep);

        let manifest_file = ManifestFile::from_dir(&dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let opts = ModifyOpts {
            dependencies: vec![dep.to_string()],
            package: Some("package-1".to_string()),
            ..Default::default()
        };

        let error = resolve_dependency(dep, &opts, &members, package_dir).unwrap_err();
        assert!(error.to_string().contains(&resp));
    }

    #[test]
    fn test_resolve_dependency_invalid_string() {
        let opts = ModifyOpts {
            dependencies: vec!["".to_string()],
            ..Default::default()
        };

        let result = resolve_dependency("", &opts, &BTreeMap::new(), PathBuf::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dependency spec cannot be empty"));
    }

    #[test]
    fn test_dep_section_insert_regular_dependency() {
        let mut deps = BTreeMap::new();
        let mut section = DepSection::Regular(&mut deps);
        let dep_name = "custom_dep";

        let dep = Dependency::Simple("1.0.0".to_string());
        section
            .insert_dep(dep_name.to_string(), dep.clone(), None)
            .unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps.get(dep_name).unwrap(), &dep);
    }

    #[test]
    fn test_dep_section_insert_regular_detailed_dependency() {
        let mut deps = BTreeMap::new();
        let mut section = DepSection::Regular(&mut deps);
        let dep_name = "detailed_dep";

        let details = DependencyDetails {
            version: Some("0.2.0".to_string()),
            git: Some("https://github.com/example/repo.git".to_string()),
            ..Default::default()
        };

        let dep = Dependency::Detailed(details.clone());
        section
            .insert_dep(dep_name.to_string(), dep.clone(), None)
            .unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps.get(dep_name).unwrap(), &dep);

        if let Dependency::Detailed(inserted_details) = deps.get(dep_name).unwrap() {
            assert_eq!(inserted_details.version, details.version);
            assert_eq!(inserted_details.git, details.git);
        } else {
            panic!("Expected a detailed dependency");
        }
    }

    #[test]
    fn test_dep_section_insert_contract_dependency_with_salt() {
        let mut deps = BTreeMap::new();
        let salt_str =
            "0x2222222222222222222222222222222222222222222222222222222222222222".to_string();
        let mut section = DepSection::Contract(&mut deps, None);
        let dep_name = "custom_dep";

        let dep = Dependency::Simple("1.0.0".to_string());
        section
            .insert_dep(dep_name.to_string(), dep.clone(), Some(salt_str.clone()))
            .unwrap();

        assert_eq!(deps.len(), 1);
        let stored = deps.get(dep_name).unwrap();
        assert_eq!(stored.dependency, dep);
        assert_eq!(stored.salt, HexSalt::from_str(&salt_str).unwrap());
    }

    #[test]
    fn test_dep_section_insert_contract_dependency_with_default_salt() {
        let mut deps = BTreeMap::new();
        let mut section = DepSection::Contract(&mut deps, None);
        let dep_name = "custom_dep";

        let dep = Dependency::Simple("1.0.0".to_string());
        section
            .insert_dep(dep_name.to_string(), dep.clone(), None)
            .unwrap();

        assert_eq!(deps.len(), 1);
        let stored = deps.get(dep_name).unwrap();
        assert_eq!(stored.dependency, dep);
        assert_eq!(stored.salt, HexSalt(fuel_tx::Salt::default()));
    }

    #[test]
    fn test_dep_section_insert_contract_dependency_with_invalid_salt() {
        let mut deps = BTreeMap::new();
        let mut section = DepSection::Contract(&mut deps, None);
        let dep_name = "custom_dep";

        let dep = Dependency::Simple("1.0.0".to_string());
        let result = section.insert_dep(dep_name.to_string(), dep, Some("not_hex".to_string()));

        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("Invalid salt format"));
    }

    #[test]
    fn test_dep_section_add_to_toml_regular_dependency_success() {
        let toml_str = r#"
            [project]
            name = "package"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]
        "#;
        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut deps = BTreeMap::new();
        deps.insert("dep1".into(), Dependency::Simple("1.0.0".into()));

        let section = DepSection::Regular(&mut deps);
        section.add_deps_manifest_table(&mut doc, &mut manifest);

        assert_eq!(doc["dependencies"]["dep1"].as_str(), Some("1.0.0"));
    }

    #[test]
    fn test_dep_section_add_to_toml_regular_detailed_dependency_success() {
        let toml_str = r#"
        [project]
        name = "package"
        entry = "main.sw"
        license = "Apache-2.0"
        authors = ["Fuel Labs"]
    "#;
        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut deps = BTreeMap::new();
        deps.insert(
            "dep2".into(),
            Dependency::Detailed(DependencyDetails {
                git: Some("https://github.com/example/repo".to_string()),
                tag: Some("v1.2.3".to_string()),
                ..Default::default()
            }),
        );

        let section = DepSection::Regular(&mut deps);
        section.add_deps_manifest_table(&mut doc, &mut manifest);

        let table = doc["dependencies"]["dep2"].as_inline_table().unwrap();
        assert_eq!(
            table.get("git").unwrap().as_str(),
            Some("https://github.com/example/repo")
        );
        assert_eq!(table.get("tag").unwrap().as_str(), Some("v1.2.3"));
    }

    #[test]
    fn test_dep_section_add_contract_dependency_with_salt() {
        let toml_str = r#"
            [project]
            name = "contract_pkg"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut deps = BTreeMap::new();
        let mut section = DepSection::Contract(&mut deps, None);
        let dep_name = "custom_dep";
        let dep = Dependency::Simple("1.0.0".to_string());
        section
            .insert_dep(dep_name.to_string(), dep.clone(), None)
            .unwrap();

        section.add_deps_manifest_table(&mut doc, &mut manifest);

        let contract_table = doc["contract-dependencies"][dep_name]
            .as_inline_table()
            .expect("inline table not found");

        assert_eq!(
            contract_table.get("version").unwrap().as_str(),
            Some("1.0.0")
        );
        assert_eq!(
            contract_table.get("salt").unwrap().as_str(),
            Some(fuel_tx::Salt::default().to_string().as_str())
        );
    }

    #[test]
    fn test_dep_section_remove_regular_dependency_success() {
        let toml_str = r#"
            [project]
            name = "package"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]

            [dependencies]
            foo = "1.0.0"
            bar = "2.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut deps = BTreeMap::new();
        deps.insert("foo".to_string(), Dependency::Simple("1.0.0".to_string()));
        deps.insert("bar".to_string(), Dependency::Simple("2.0.0".to_string()));

        let mut section = DepSection::Regular(&mut deps);
        section
            .remove_deps_manifest_table(&mut doc, &mut manifest, &["foo"])
            .unwrap();

        assert!(doc["dependencies"].as_table().unwrap().get("foo").is_none());
        assert!(doc["dependencies"].as_table().unwrap().get("bar").is_some());
    }

    #[test]
    fn test_remove_regular_dependency_not_found() {
        let toml_str = r#"
            [project]
            name = "package"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]

            [dependencies]
            bar = "2.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut deps = BTreeMap::new();
        deps.insert("bar".to_string(), Dependency::Simple("2.0.0".to_string()));

        let mut section = DepSection::Regular(&mut deps);
        let err = section
            .remove_deps_manifest_table(&mut doc, &mut manifest, &["notfound"])
            .unwrap_err()
            .to_string();

        assert!(err.contains("the dependency `notfound` could not be found in `dependencies`"));
    }

    #[test]
    fn test_remove_contract_dependency_success() {
        let toml_str = r#"
            [project]
            name = "package"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]

            [contract-dependencies]
            baz = { path = "../baz", salt = "0x1111111111111111111111111111111111111111111111111111111111111111" }
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut map = BTreeMap::new();
        map.insert(
            "baz".to_string(),
            ContractDependency {
                dependency: Dependency::Detailed(DependencyDetails {
                    path: Some("../baz".to_string()),
                    ..Default::default()
                }),
                salt: HexSalt::from_str(
                    "0x1111111111111111111111111111111111111111111111111111111111111111",
                )
                .unwrap(),
            },
        );

        let mut section = DepSection::Contract(&mut map, None);
        section
            .remove_deps_manifest_table(&mut doc, &mut manifest, &["baz"])
            .unwrap();

        assert!(doc["contract-dependencies"]
            .as_table()
            .unwrap()
            .get("baz")
            .is_none());
    }

    #[test]
    fn test_remove_contract_dependency_not_found() {
        let toml_str = r#"
            [project]
            name = "package"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]

            [contract-dependencies]
            baz = { path = "../baz", salt = "0x1111111111111111111111111111111111111111111111111111111111111111" }
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();
        let mut manifest = PackageManifest::from_string(toml_str.to_string()).unwrap();

        let mut map = BTreeMap::new();
        map.insert(
            "baz".to_string(),
            ContractDependency {
                dependency: Dependency::Detailed(DependencyDetails {
                    path: Some("../baz".to_string()),
                    ..Default::default()
                }),
                salt: HexSalt::from_str(
                    "0x1111111111111111111111111111111111111111111111111111111111111111",
                )
                .unwrap(),
            },
        );

        let mut section = DepSection::Contract(&mut map, None);
        let err = section
            .remove_deps_manifest_table(&mut doc, &mut manifest, &["ghost"])
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("the dependency `ghost` could not be found in `contract-dependencies`")
        );
    }

    #[test]
    fn test_remove_single_dep() -> Result<()> {
        let toml_str = r#"
            [project]
            authors = ["Fuel Labs <contact@fuel.sh>"]
            entry = "main.sw"
            license = "Apache-2.0"
            name = "package-1"

            [dependencies]
            foo = "1.0.0"
            bar = "2.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().expect("failed to parse TOML");

        remove_deps_from_table(&mut doc, "dependencies", &["foo"])?;

        let table = doc["dependencies"].as_table().unwrap();
        assert!(!table.contains_key("foo"));
        assert!(table.contains_key("bar"));

        Ok(())
    }

    #[test]
    fn test_remove_multiple_deps() -> Result<()> {
        let toml_str = r#"
            [project]
            authors = ["Fuel Labs <contact@fuel.sh>"]
            entry = "main.sw"
            license = "Apache-2.0"
            name = "package-1"

            [dependencies]
            foo = "1.0.0"
            bar = "2.0.0"
            baz = "3.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().expect("failed to parse TOML");
        remove_deps_from_table(&mut doc, "dependencies", &["foo", "bar"])?;

        let table = doc["dependencies"].as_table().unwrap();
        assert!(!table.contains_key("foo"));
        assert!(!table.contains_key("bar"));
        assert!(table.contains_key("baz"));

        Ok(())
    }

    #[test]
    fn test_remove_from_missing_section() {
        let toml_str = r#"
            [project]
            authors = ["Fuel Labs <contact@fuel.sh>"]
            entry = "main.sw"
            license = "Apache-2.0"
            name = "package-1"

            [dependencies]
            foo = "1.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().expect("failed to parse TOML");

        let result = remove_deps_from_table(&mut doc, "contract-dependencies", &["foo"]);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err())
            .contains("section [contract-dependencies] not found"));
    }

    #[test]
    fn test_generate_table_basic_fields() {
        let details = DependencyDetails {
            version: Some("1.2.3".to_string()),
            git: Some("https://github.com/example/repo".to_string()),
            branch: Some("main".to_string()),
            tag: Some("v1.0.0".to_string()),
            rev: Some("deadbeef".to_string()),
            path: Some("./lib".to_string()),
            ipfs: Some("QmYw...".to_string()),
            namespace: None,
            package: None,
        };

        let table = generate_table(&details);

        assert_eq!(table.get("version").unwrap().as_str(), Some("1.2.3"));
        assert_eq!(
            table.get("git").unwrap().as_str(),
            Some("https://github.com/example/repo")
        );
        assert_eq!(table.get("branch").unwrap().as_str(), Some("main"));
        assert_eq!(table.get("tag").unwrap().as_str(), Some("v1.0.0"));
        assert_eq!(table.get("rev").unwrap().as_str(), Some("deadbeef"));
        assert_eq!(table.get("path").unwrap().as_str(), Some("./lib"));
        assert_eq!(table.get("cid").unwrap().as_str(), Some("QmYw..."));
    }
}
