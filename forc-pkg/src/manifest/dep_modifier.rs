use crate::manifest::{
    ContractDependency, Dependency, DependencyDetails, GenericManifestFile, HexSalt,
};
use crate::source::IPFSNode;
use crate::{self as pkg, Lock, PackageManifestFile};
use anyhow::{anyhow, bail, Result};
use pkg::manifest::ManifestFile;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use sway_core::fuel_prelude::fuel_tx;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};
use tracing::info;

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
    let manifest_file = if let Some(p) = &opts.manifest_path {
        let path = &PathBuf::from(p);
        ManifestFile::from_file(path)?
    } else {
        let cwd = std::env::current_dir()?;
        ManifestFile::from_dir(cwd)?
    };

    let root_dir = manifest_file.root_dir();
    let member_manifests = manifest_file.member_manifests()?;

    let package_manifest_dir =
        resolve_package_path(&manifest_file, &opts.package, &root_dir, &member_manifests)?;

    let content = std::fs::read_to_string(&package_manifest_dir)?;
    let mut toml_doc = content.parse::<DocumentMut>()?;
    let backup_doc = toml_doc.clone();

    let old_package_manifest = PackageManifestFile::from_file(&package_manifest_dir)?;
    let lock_path = old_package_manifest.lock_path()?;
    let old_lock = Lock::from_path(&lock_path).ok().unwrap_or_default();

    let section = if opts.contract_deps {
        Section::ContractDeps
    } else {
        Section::Deps
    };

    match opts.action {
        Action::Add => {
            for dependency in &opts.dependencies {
                let (dep_name, dependency_data) = resolve_dependency(
                    dependency,
                    &opts,
                    &member_manifests,
                    &old_package_manifest.dir().to_path_buf(),
                )?;

                section.add_deps_manifest_table(
                    &mut toml_doc,
                    dep_name,
                    dependency_data,
                    opts.salt.clone(),
                )?;
            }
        }
        Action::Remove => {
            let dep_refs: Vec<&str> = opts.dependencies.iter().map(String::as_str).collect();

            section.remove_deps_manifest_table(&mut toml_doc, &dep_refs)?;
        }
    }

    // write updates to toml doc
    std::fs::write(&package_manifest_dir, toml_doc.to_string())?;

    let updated_package_manifest = PackageManifestFile::from_file(&package_manifest_dir)?;

    let member_manifests = updated_package_manifest.member_manifests()?;

    let new_plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        false,
        opts.offline,
        &opts.ipfs_node.clone().unwrap_or_default(),
    );

    new_plan.or_else(|e| {
        std::fs::write(&package_manifest_dir, backup_doc.to_string())
            .map_err(|write_err| anyhow!("failed to write toml file: {}", write_err))?;
        Err(e)
    })?;

    if opts.dry_run {
        info!("Dry run enabled. toml file not modified.");
        std::fs::write(&package_manifest_dir, backup_doc.to_string())?;

        let string = toml::ser::to_string_pretty(&old_lock)?;
        std::fs::write(&lock_path, string)?;

        return Ok(());
    }

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
        bail!(
            "package(s) {} not found in workspace {}",
            package_name,
            root_dir.to_string_lossy()
        )
    }
}

fn resolve_dependency(
    raw: &str,
    opts: &ModifyOpts,
    member_manifests: &BTreeMap<String, PackageManifestFile>,
    package_dir: &PathBuf,
) -> Result<(String, Dependency)> {
    let dep_spec: DepSpec = raw.parse()?;
    let dep_name = dep_spec.name;

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
    } else if details.is_source_empty() {
        if let Some(member) = member_manifests.get(&dep_name) {
            if member.dir() == package_dir {
                bail!("cannot add `{}` as a dependency to itself", dep_name);
            }

            let sibling_parent = package_dir.parent().unwrap();
            let rel_path = member
                .dir()
                .strip_prefix(sibling_parent)
                .map(|p| PathBuf::from("..").join(p))
                .unwrap_or_else(|_| member.dir().to_path_buf());

            details.path = Some(rel_path.to_string_lossy().to_string());
            Dependency::Detailed(details)
        } else {
            // Fallback: no explicit source & not a sibling package.
            // TODO: Integrate registry support (e.g., forc.pub) here.
            bail!(
                "dependency `{}` source not specified. Please specify a source (e.g., git, path) or version.",
                dep_name
            );
        }
    } else {
        Dependency::Detailed(details)
    };

    Ok((dep_name, dependency_data))
}

/// Reference to a package to be added as a dependency.
///
/// See `forc add` help for more info.
#[derive(Clone, Debug, Default)]
pub struct DepSpec {
    pub name: String,
    pub version_req: Option<String>,
}

impl FromStr for DepSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s.trim().is_empty() {
            bail!("Dependency spec cannot be empty");
        }

        let mut s = s.trim().split('@');

        let name = s
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing dependency name"))?;

        let version_req = s.next().map(|s| s.to_string());

        if let Some(ref v) = version_req {
            semver::VersionReq::parse(v)
                .map_err(|_| anyhow::anyhow!("invalid version requirement `{v}`"))?;
        }

        Ok(Self {
            name: name.to_string(),
            version_req,
        })
    }
}

impl fmt::Display for DepSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version_req {
            Some(version) => write!(f, "{}@{}", self.name, version),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Clone)]
pub enum Section {
    Deps,
    ContractDeps,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let section = match self {
            Section::Deps => "dependencies",
            Section::ContractDeps => "contract-dependencies",
        };
        write!(f, "{}", section)
    }
}

impl Section {
    pub fn add_deps_manifest_table(
        &self,
        doc: &mut DocumentMut,
        dep_name: String,
        dep_data: Dependency,
        salt: Option<String>,
    ) -> Result<()> {
        let section_name = self.to_string();

        if !doc.as_table().contains_key(&section_name) {
            doc[&section_name] = Item::Table(Table::new());
        }

        let table = doc[section_name.as_str()].as_table_mut().unwrap();

        match self {
            Section::Deps => {
                let item = match dep_data {
                    Dependency::Simple(ver) => ver.to_string().into(),
                    Dependency::Detailed(details) => {
                        Item::Value(toml_edit::Value::InlineTable(generate_table(&details)))
                    }
                };
                table.insert(&dep_name, item);
            }
            Section::ContractDeps => {
                let resolved_salt = match salt.as_ref().or(salt.as_ref()) {
                    Some(s) => {
                        HexSalt::from_str(s).map_err(|e| anyhow!("Invalid salt format: {}", e))?
                    }
                    None => HexSalt(fuel_tx::Salt::default()),
                };
                let contract_dep = ContractDependency {
                    dependency: dep_data,
                    salt: resolved_salt.clone(),
                };

                let dep = &contract_dep.dependency;
                let salt: &HexSalt = &contract_dep.salt;
                let item = match dep {
                    Dependency::Simple(ver) => {
                        let mut inline = InlineTable::default();
                        inline.insert("version", Value::from(ver.to_string()));
                        inline.insert("salt", Value::from(format!("0x{}", salt)));
                        Item::Value(toml_edit::Value::InlineTable(inline))
                    }
                    Dependency::Detailed(details) => {
                        let mut inline = generate_table(details);
                        inline.insert("salt", Value::from(format!("0x{}", salt)));
                        Item::Value(toml_edit::Value::InlineTable(inline))
                    }
                };
                table.insert(&dep_name, item);
            }
        };

        Ok(())
    }

    pub fn remove_deps_manifest_table(self, doc: &mut DocumentMut, deps: &[&str]) -> Result<()> {
        let section_name = self.to_string();

        let section_table = doc[section_name.as_str()].as_table_mut().ok_or_else(|| {
            anyhow!(
                "the dependency `{}` could not be found in `{}`",
                deps.join(", "),
                section_name,
            )
        })?;

        match self {
            Section::Deps => {
                for dep in deps {
                    if !section_table.contains_key(dep) {
                        bail!(
                            "the dependency `{}` could not be found in `{}`",
                            dep,
                            section_name
                        );
                    }
                    section_table.remove(dep);
                }
            }
            Section::ContractDeps => {
                for dep in deps {
                    if !section_table.contains_key(dep) {
                        bail!(
                            "the dependency `{}` could not be found in `{}`",
                            dep,
                            section_name
                        );
                    }
                    section_table.remove(dep);
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkspaceManifestFile;
    use std::fs;
    use std::str::FromStr;
    use tempfile::{tempdir, TempDir};

    fn create_test_package(
        name: &str,
        source_files: Vec<(&str, &str)>,
    ) -> Result<(TempDir, PackageManifestFile)> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path();

        // Create package structure
        fs::create_dir_all(base_path.join("src"))?;

        // Create Forc.toml
        let forc_toml = format!(
            r#"
            [project]
            authors = ["Test"]
            entry = "main.sw"
            license = "MIT"
            name = "{}"
            
            [dependencies]
        "#,
            name
        );
        fs::write(base_path.join("Forc.toml"), forc_toml)?;

        // Create source files
        for (file_name, content) in source_files {
            // Handle nested directories in the file path
            let file_path = base_path.join("src").join(file_name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, content)?;
        }

        // Create the manifest file
        let manifest_file = PackageManifestFile::from_file(base_path.join("Forc.toml"))?;

        Ok((temp_dir, manifest_file))
    }

    fn create_test_workspace(
        members: Vec<(&str, Vec<(&str, &str)>)>,
    ) -> Result<(TempDir, WorkspaceManifestFile)> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path();

        // Create workspace Forc.toml
        let mut workspace_toml = "[workspace]\nmembers = [".to_string();

        for (i, (name, _)) in members.iter().enumerate() {
            if i > 0 {
                workspace_toml.push_str(", ");
            }
            workspace_toml.push_str(&format!("\"{name}\""));
        }
        workspace_toml.push_str("]\n");

        fs::write(base_path.join("Forc.toml"), workspace_toml)?;

        // Create each member
        for (name, source_files) in members {
            let member_path = base_path.join(name);
            fs::create_dir_all(member_path.join("src"))?;

            // Create member Forc.toml
            let forc_toml = format!(
                r#"
                [project]
                authors = ["Test"]
                entry = "main.sw"
                license = "MIT"
                name = "{}"
                
                [dependencies]
            "#,
                name
            );
            fs::write(member_path.join("Forc.toml"), forc_toml)?;

            // Create source files
            for (file_name, content) in source_files {
                // Handle nested directories in the file path
                let file_path = member_path.join("src").join(file_name);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(file_path, content)?;
            }
        }

        // Create the workspace manifest file
        let manifest_file = WorkspaceManifestFile::from_file(base_path.join("Forc.toml"))?;

        Ok((temp_dir, manifest_file))
    }

    #[test]
    fn test_dep_from_str_name_only() {
        let dep: DepSpec = "abc".parse().expect("parsing dep spec failed");
        assert_eq!(dep.name, "abc".to_string());
        assert_eq!(dep.version_req, None);
    }

    #[test]
    fn test_dep_from_str_name_and_version() {
        let dep: DepSpec = "abc@1".parse().expect("parsing dep spec failed");
        assert_eq!(dep.name, "abc".to_string());
        assert_eq!(dep.version_req, Some("1".to_string()));
    }

    #[test]
    fn test_dep_spec_invalid_version_req() {
        let input = "foo@not-a-version";
        let result = DepSpec::from_str(input);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid version requirement"),
            "Expected version requirement parse failure"
        );
    }

    #[test]
    fn test_dep_from_str_invalid() {
        assert!(DepSpec::from_str("").is_err());
    }

    #[test]
    fn test_resolve_package_path_single_package_mode() {
        let (temp_dir, pkg_manifest) =
            create_test_package("test_pkg", vec![("main.sw", "fn main() -> u64 { 42 }")]).unwrap();

        let package_spec_dir = temp_dir.path().to_path_buf();
        let expected_path = pkg_manifest.path;

        let manifest_file = ManifestFile::from_dir(&package_spec_dir).unwrap();

        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();
        let result = resolve_package_path(&manifest_file, &None, &root_dir, &members).unwrap();

        assert_eq!(result, expected_path);
    }

    #[test]
    fn test_resolve_package_path_workspace_with_package_found() {
        let (temp_dir, _) = create_test_workspace(vec![
            ("pkg1", vec![("main.sw", "fn main() -> u64 { 1 }")]),
            ("pkg2", vec![("main.sw", "fn main() -> u64 { 2 }")]),
        ])
        .unwrap();

        let base_path = temp_dir.path();

        let expected_path = base_path.join("pkg1/Forc.toml");

        let manifest_file = ManifestFile::from_dir(base_path).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();

        let package = "pkg1".to_string();
        let result =
            resolve_package_path(&manifest_file, &Some(package), &root_dir, &members).unwrap();

        assert_eq!(result, expected_path);
    }

    #[test]
    fn test_resolve_package_path_workspace_package_not_found() {
        let (temp_dir, _) = create_test_workspace(vec![
            ("pkg1", vec![("main.sw", "fn main() -> u64 { 1 }")]),
            ("pkg2", vec![("main.sw", "fn main() -> u64 { 2 }")]),
        ])
        .unwrap();

        let base_path = temp_dir.path();

        let manifest_file = ManifestFile::from_dir(base_path).unwrap();
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
        let (temp_dir, _) = create_test_workspace(vec![
            ("pkg1", vec![("main.sw", "fn main() -> u64 { 1 }")]),
            ("pkg2", vec![("main.sw", "fn main() -> u64 { 2 }")]),
        ])
        .unwrap();

        let base_path = temp_dir.path();

        let manifest_file = ManifestFile::from_dir(base_path).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let root_dir = manifest_file.root_dir();

        let err = resolve_package_path(&manifest_file, &None, &root_dir, &members).unwrap_err();

        let resp = "`forc add` could not determine which package to modify. Use --package.\nAvailable: pkg1, pkg2".to_string();
        assert!(err.to_string().contains(&resp), "unexpected error: {err}");
    }

    #[test]
    fn test_resolve_dependency_simple_version() {
        let opts = ModifyOpts {
            dependencies: vec!["dep@1.0.0".to_string()],
            ..Default::default()
        };

        let (temp_dir, _) =
            create_test_package("test_pkg", vec![("main.sw", "fn main() -> u64 { 42 }")]).unwrap();

        let package_spec_dir = temp_dir.path().to_path_buf();

        let manifest_file = ManifestFile::from_dir(&package_spec_dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let (name, data) =
            resolve_dependency("dep@1.0.0", &opts, &members, &package_spec_dir).unwrap();

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

        let (temp_dir, _) =
            create_test_package("test_pkg", vec![("main.sw", "fn main() -> u64 { 42 }")]).unwrap();

        let package_spec_dir = temp_dir.path().to_path_buf();

        let manifest_file = ManifestFile::from_dir(&package_spec_dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let dep = "dummy_dep";
        let git = "https://github.com/example/repo.git";

        // Git alone
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());

            let (name, data) = resolve_dependency(dep, &opts, &members, &package_spec_dir).unwrap();
            assert_eq!(name, dep);
            match data {
                Dependency::Detailed(details) => {
                    assert_eq!(details.git.as_deref(), Some(git));
                }
                _ => panic!("Expected detailed dependency with git"),
            }
        }

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
    fn test_resolve_dependency_detailed_variant_failure() {
        let base_opts = ModifyOpts {
            ..Default::default()
        };

        let (temp_dir, _) =
            create_test_package("test_pkg", vec![("main.sw", "fn main() -> u64 { 42 }")]).unwrap();

        let package_spec_dir = temp_dir.path().to_path_buf();
        let manifest_file = ManifestFile::from_dir(&package_spec_dir).unwrap();
        let members = manifest_file.member_manifests().unwrap();
        let dep = "dummy_dep";
        let git = "https://github.com/example/repo.git";

        // no Git + branch
        {
            let mut opts = base_opts.clone();
            opts.branch = Some("main".to_string());
            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Details reserved for git sources used without a git field"));
        }

        // no Git + rev
        {
            let mut opts = base_opts.clone();
            opts.rev = Some("deadbeef".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Details reserved for git sources used without a git field"));
        }

        // no Git + tag
        {
            let mut opts = base_opts.clone();
            opts.tag = Some("v1.2.3".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Details reserved for git sources used without a git field"));
        }

        // git + tag + rev + branch
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.tag = Some("v1.2.3".to_string());
            opts.rev = Some("deadbeef".to_string());
            opts.branch = Some("main".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Cannot specify `branch`, `tag`, and `rev` together for dependency with a Git source"));
        }

        // git + branch + tag
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.tag = Some("v1.2.3".to_string());
            opts.branch = Some("main".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains(
                "Cannot specify both `branch` and `tag` for dependency with a Git source"
            ));
        }

        // git + tag + rev
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.tag = Some("v1.2.3".to_string());
            opts.rev = Some("deadbeef".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Cannot specify both `rev` and `tag` for dependency with a Git source"));
        }

        // git + branch + rev
        {
            let mut opts = base_opts.clone();
            opts.git = Some(git.to_string());
            opts.rev = Some("deadbeef".to_string());
            opts.branch = Some("main".to_string());

            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains(
                "Cannot specify both `branch` and `rev` for dependency with a Git source"
            ));
        }

        // no source provided
        {
            let opts = base_opts.clone();
            let result = resolve_dependency(dep, &opts, &members, &package_spec_dir);

            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains(
                "dependency `dummy_dep` source not specified. Please specify a source (e.g., git, path) or version"
            ));
        }
    }

    #[test]
    fn test_resolve_dependency_from_workspace_sibling() {
        let (temp_dir, _) = create_test_workspace(vec![
            ("pkg1", vec![("main.sw", "fn main() -> u64 { 1 }")]),
            ("pkg2", vec![("main.sw", "fn main() -> u64 { 2 }")]),
        ])
        .unwrap();

        let base_path = temp_dir.path();
        let package_dir = base_path.join("pkg2");

        let dep = "pkg1";

        let manifest_file = ManifestFile::from_dir(base_path).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let opts = ModifyOpts {
            source_path: None,
            dependencies: vec![dep.to_string()],
            package: Some("pkg2".to_string()),
            ..Default::default()
        };

        let (name, data) =
            resolve_dependency(dep, &opts, &members, &package_dir).expect("should resolve");

        assert_eq!(name, dep);
        match data {
            Dependency::Detailed(details) => {
                assert!(details.path.is_some());
                let actual_path = details.path.as_ref().unwrap();
                assert_eq!(actual_path, "../pkg1");
            }
            _ => panic!("Expected detailed dependency with fallback path"),
        }
    }

    #[test]
    fn test_resolve_dependency_self_dependency_error() {
        let (temp_dir, _) = create_test_workspace(vec![
            ("pkg1", vec![("main.sw", "fn main() -> u64 { 1 }")]),
            ("pkg2", vec![("main.sw", "fn main() -> u64 { 2 }")]),
        ])
        .unwrap();

        let base_path = temp_dir.path();
        let package_dir = base_path.join("pkg1");
        let dep = "pkg1";
        let resp = format!("cannot add `{}` as a dependency to itself", dep);

        let manifest_file = ManifestFile::from_dir(base_path).unwrap();
        let members = manifest_file.member_manifests().unwrap();

        let opts = ModifyOpts {
            dependencies: vec![dep.to_string()],
            package: Some("package-1".to_string()),
            ..Default::default()
        };

        let error = resolve_dependency(dep, &opts, &members, &package_dir).unwrap_err();
        assert!(error.to_string().contains(&resp));
    }

    #[test]
    fn test_resolve_dependency_invalid_string() {
        let opts = ModifyOpts {
            dependencies: vec!["".to_string()],
            ..Default::default()
        };

        let result = resolve_dependency("", &opts, &BTreeMap::new(), &PathBuf::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Dependency spec cannot be empty"));
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

        let dep_data = Dependency::Simple("1.0.0".into());

        let section = Section::Deps;

        section
            .add_deps_manifest_table(&mut doc, "dep1".into(), dep_data, None)
            .unwrap();

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

        let dep_data = Dependency::Detailed(DependencyDetails {
            git: Some("https://github.com/example/repo".to_string()),
            tag: Some("v1.2.3".to_string()),
            ..Default::default()
        });

        let section = Section::Deps;

        section
            .add_deps_manifest_table(&mut doc, "dep2".into(), dep_data, None)
            .unwrap();

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

        let section = Section::ContractDeps;
        let dep_name = "custom_dep";
        let dep_data = Dependency::Simple("1.0.0".to_string());
        let salt_str = "0x2222222222222222222222222222222222222222222222222222222222222222";
        let hex_salt = HexSalt::from_str(salt_str).unwrap();

        section
            .add_deps_manifest_table(
                &mut doc,
                dep_name.to_string(),
                dep_data,
                Some(salt_str.to_string()),
            )
            .unwrap();

        let contract_table = doc["contract-dependencies"][dep_name]
            .as_inline_table()
            .expect("inline table not found");

        assert_eq!(
            contract_table.get("version").unwrap().as_str(),
            Some("1.0.0")
        );
        assert_eq!(
            contract_table.get("salt").unwrap().as_str(),
            Some(format!("0x{}", hex_salt).as_str())
        );
    }

    #[test]
    fn test_dep_section_add_contract_dependency_with_default_salt() {
        let toml_str = r#"
            [project]
            name = "contract_pkg"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();

        let section = Section::ContractDeps;
        let dep_name = "custom_dep";
        let dep_data = Dependency::Simple("1.0.0".to_string());

        section
            .add_deps_manifest_table(&mut doc, dep_name.to_string(), dep_data, None)
            .unwrap();

        let contract_table = doc["contract-dependencies"][dep_name]
            .as_inline_table()
            .expect("inline table not found");

        assert_eq!(
            contract_table.get("version").unwrap().as_str(),
            Some("1.0.0")
        );
        assert_eq!(
            contract_table.get("salt").unwrap().as_str(),
            Some(format!("0x{}", fuel_tx::Salt::default()).as_str())
        );
    }

    #[test]
    fn test_dep_section_add_contract_dependency_with_invalid_salt() {
        let toml_str = r#"
            [project]
            name = "contract_pkg"
            entry = "main.sw"
            license = "Apache-2.0"
            authors = ["Fuel Labs"]
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();

        let section = Section::ContractDeps;
        let dep_name = "custom_dep";
        let dep_data = Dependency::Simple("1.0.0".to_string());

        let result = section.add_deps_manifest_table(
            &mut doc,
            dep_name.to_string(),
            dep_data,
            Some("not_hex".to_string()),
        );

        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("Invalid salt format"));
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

        let section = Section::Deps;
        section
            .remove_deps_manifest_table(&mut doc, &["foo"])
            .unwrap();

        assert!(doc["dependencies"].as_table().unwrap().get("foo").is_none());
        assert!(doc["dependencies"].as_table().unwrap().get("bar").is_some());
    }

    #[test]
    fn test_dep_section_remove_regular_dependency_not_found() {
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

        let section = Section::Deps;

        let err = section
            .remove_deps_manifest_table(&mut doc, &["notfound"])
            .unwrap_err()
            .to_string();

        assert!(err.contains("the dependency `notfound` could not be found in `dependencies`"));
    }

    #[test]
    fn test_dep_section_remove_contract_dependency_success() {
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

        let section = Section::ContractDeps;
        section
            .remove_deps_manifest_table(&mut doc, &["baz"])
            .unwrap();

        assert!(doc["contract-dependencies"]
            .as_table()
            .unwrap()
            .get("baz")
            .is_none());
    }

    #[test]
    fn test_dep_section_remove_contract_dependency_not_found() {
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

        let section = Section::ContractDeps;

        let result = section.remove_deps_manifest_table(&mut doc, &["ghost"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("the dependency `ghost` could not be found in `contract-dependencies`"));
    }

    #[test]
    fn test_dep_section_remove_from_missing_section() {
        let toml_str = r#"
            [project]
            authors = ["Fuel Labs <contact@fuel.sh>"]
            entry = "main.sw"
            license = "Apache-2.0"
            name = "package-1"

            [dependencies]
            foo = "1.0.0"
        "#;

        let mut doc: DocumentMut = toml_str.parse().unwrap();

        let section = Section::ContractDeps;

        let result = section.remove_deps_manifest_table(&mut doc, &["ghost"]);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("the dependency `ghost` could not be found in `contract-dependencies`"));
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
