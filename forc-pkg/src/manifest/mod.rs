pub mod build_profile;
pub mod dep_modifier;

use crate::pkg::{manifest_file_missing, parsing_failed, wrong_program_type};
use anyhow::{anyhow, bail, Context, Result};
use forc_tracing::println_warning;
use forc_util::{validate_name, validate_project_name};
use semver::Version;
use serde::{de, Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};
use sway_core::{fuel_prelude::fuel_tx, language::parsed::TreeType, parse_tree_type, BuildTarget};
use sway_error::handler::Handler;
use sway_types::span::Source;
use sway_utils::{
    constants, find_nested_manifest_dir, find_parent_manifest_dir,
    find_parent_manifest_dir_with_check,
};
use url::Url;

use self::build_profile::BuildProfile;

/// The name of a workspace member package.
pub type MemberName = String;
/// A manifest for each workspace member, or just one manifest if working with a single package
pub type MemberManifestFiles = BTreeMap<MemberName, PackageManifestFile>;

pub trait GenericManifestFile {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
    fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self>
    where
        Self: Sized;

    /// The path to the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    fn path(&self) -> &Path;

    /// The path to the directory containing the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    fn dir(&self) -> &Path {
        self.path()
            .parent()
            .expect("failed to retrieve manifest directory")
    }

    /// Returns the path of the `Forc.lock` file.
    fn lock_path(&self) -> Result<PathBuf>;

    /// Returns a mapping of member member names to package manifest files.
    fn member_manifests(&self) -> Result<MemberManifestFiles>;
}

#[derive(Clone, Debug)]
pub enum ManifestFile {
    Package(Box<PackageManifestFile>),
    Workspace(WorkspaceManifestFile),
}

impl ManifestFile {
    pub fn is_workspace(&self) -> bool {
        matches!(self, ManifestFile::Workspace(_))
    }

    pub fn root_dir(&self) -> PathBuf {
        match self {
            ManifestFile::Package(pkg_manifest_file) => pkg_manifest_file
                .workspace()
                .ok()
                .flatten()
                .map(|ws| ws.dir().to_path_buf())
                .unwrap_or_else(|| pkg_manifest_file.dir().to_path_buf()),
            ManifestFile::Workspace(workspace_manifest_file) => {
                workspace_manifest_file.dir().to_path_buf()
            }
        }
    }
}

impl GenericManifestFile for ManifestFile {
    /// Returns a `PackageManifestFile` if the path is within a package directory, otherwise
    /// returns a `WorkspaceManifestFile` if within a workspace directory.
    fn from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let maybe_pkg_manifest = PackageManifestFile::from_dir(path.as_ref());
        let manifest_file = if let Err(e) = maybe_pkg_manifest {
            if e.to_string().contains("missing field `project`") {
                // This might be a workspace manifest file
                let workspace_manifest_file = WorkspaceManifestFile::from_dir(path.as_ref())?;
                ManifestFile::Workspace(workspace_manifest_file)
            } else {
                bail!("{}", e)
            }
        } else if let Ok(pkg_manifest) = maybe_pkg_manifest {
            ManifestFile::Package(Box::new(pkg_manifest))
        } else {
            bail!(
                "Cannot find a valid `Forc.toml` at {}",
                path.as_ref().to_string_lossy()
            )
        };
        Ok(manifest_file)
    }

    /// Returns a `PackageManifestFile` if the path is pointing to package manifest, otherwise
    /// returns a `WorkspaceManifestFile` if it is pointing to a workspace manifest.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let maybe_pkg_manifest = PackageManifestFile::from_file(path.as_ref());
        let manifest_file = if let Err(e) = maybe_pkg_manifest {
            if e.to_string().contains("missing field `project`") {
                // This might be a workspace manifest file
                let workspace_manifest_file = WorkspaceManifestFile::from_file(path.as_ref())?;
                ManifestFile::Workspace(workspace_manifest_file)
            } else {
                bail!("{}", e)
            }
        } else if let Ok(pkg_manifest) = maybe_pkg_manifest {
            ManifestFile::Package(Box::new(pkg_manifest))
        } else {
            bail!(
                "Cannot find a valid `Forc.toml` at {}",
                path.as_ref().to_string_lossy()
            )
        };
        Ok(manifest_file)
    }

    /// The path to the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    fn path(&self) -> &Path {
        match self {
            ManifestFile::Package(pkg_manifest_file) => pkg_manifest_file.path(),
            ManifestFile::Workspace(workspace_manifest_file) => workspace_manifest_file.path(),
        }
    }

    fn member_manifests(&self) -> Result<MemberManifestFiles> {
        match self {
            ManifestFile::Package(pkg_manifest_file) => pkg_manifest_file.member_manifests(),
            ManifestFile::Workspace(workspace_manifest_file) => {
                workspace_manifest_file.member_manifests()
            }
        }
    }

    /// Returns the path of the lock file for the given ManifestFile
    fn lock_path(&self) -> Result<PathBuf> {
        match self {
            ManifestFile::Package(pkg_manifest) => pkg_manifest.lock_path(),
            ManifestFile::Workspace(workspace_manifest) => workspace_manifest.lock_path(),
        }
    }
}

impl TryInto<PackageManifestFile> for ManifestFile {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PackageManifestFile> {
        match self {
            ManifestFile::Package(pkg_manifest_file) => Ok(*pkg_manifest_file),
            ManifestFile::Workspace(_) => {
                bail!("Cannot convert workspace manifest to package manifest")
            }
        }
    }
}

impl TryInto<WorkspaceManifestFile> for ManifestFile {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<WorkspaceManifestFile> {
        match self {
            ManifestFile::Package(_) => {
                bail!("Cannot convert package manifest to workspace manifest")
            }
            ManifestFile::Workspace(workspace_manifest_file) => Ok(workspace_manifest_file),
        }
    }
}

type PatchMap = BTreeMap<String, Dependency>;

/// A [PackageManifest] that was deserialized from a file at a particular path.
#[derive(Clone, Debug, PartialEq)]
pub struct PackageManifestFile {
    /// The deserialized `Forc.toml`.
    manifest: PackageManifest,
    /// The path from which the `Forc.toml` file was read.
    path: PathBuf,
}

/// A direct mapping to a `Forc.toml`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PackageManifest {
    pub project: Project,
    pub network: Option<Network>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
    pub patch: Option<BTreeMap<String, PatchMap>>,
    /// A list of [configuration-time constants](https://github.com/FuelLabs/sway/issues/1498).
    pub build_target: Option<BTreeMap<String, BuildTarget>>,
    build_profile: Option<BTreeMap<String, BuildProfile>>,
    pub contract_dependencies: Option<BTreeMap<String, ContractDependency>>,
    pub proxy: Option<Proxy>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub authors: Option<Vec<String>>,
    #[serde(deserialize_with = "validate_package_name")]
    pub name: String,
    pub version: Option<Version>,
    pub description: Option<String>,
    pub organization: Option<String>,
    pub license: String,
    pub homepage: Option<Url>,
    pub repository: Option<Url>,
    pub documentation: Option<Url>,
    pub categories: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    #[serde(default = "default_entry")]
    pub entry: String,
    pub implicit_std: Option<bool>,
    pub forc_version: Option<semver::Version>,
    #[serde(default)]
    pub experimental: HashMap<String, bool>,
    pub metadata: Option<toml::Value>,
    pub force_dbg_in_release: Option<bool>,
}

// Validation function for the `name` field
fn validate_package_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let name: String = Deserialize::deserialize(deserializer)?;
    match validate_project_name(&name) {
        Ok(_) => Ok(name),
        Err(e) => Err(de::Error::custom(e.to_string())),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(default = "default_url")]
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HexSalt(pub fuel_tx::Salt);

impl FromStr for HexSalt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // cut 0x from start.
        let normalized = s
            .strip_prefix("0x")
            .ok_or_else(|| anyhow::anyhow!("hex salt declaration needs to start with 0x"))?;
        let salt: fuel_tx::Salt =
            fuel_tx::Salt::from_str(normalized).map_err(|e| anyhow::anyhow!("{e}"))?;
        let hex_salt = Self(salt);
        Ok(hex_salt)
    }
}

impl Display for HexSalt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let salt = self.0;
        write!(f, "{salt}")
    }
}

fn default_hex_salt() -> HexSalt {
    HexSalt(fuel_tx::Salt::default())
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ContractDependency {
    #[serde(flatten)]
    pub dependency: Dependency,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_hex_salt")]
    pub salt: HexSalt,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Dependency {
    /// In the simple format, only a version is specified, eg.
    /// `package = "<version>"`
    Simple(String),
    /// The simple format is equivalent to a detailed dependency
    /// specifying only a version, eg.
    /// `package = { version = "<version>" }`
    Detailed(DependencyDetails),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetails {
    pub(crate) version: Option<String>,
    pub(crate) namespace: Option<String>,
    pub path: Option<String>,
    pub(crate) git: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) package: Option<String>,
    pub(crate) rev: Option<String>,
    pub(crate) ipfs: Option<String>,
}

/// Describes the details around proxy contract.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Proxy {
    pub enabled: bool,
    /// Points to the proxy contract to be updated with the new contract id.
    /// If there is a value for this field, forc will try to update the proxy contract's storage
    /// field such that it points to current contract's deployed instance.
    pub address: Option<String>,
}

impl DependencyDetails {
    /// Checks if dependency details reserved for a specific dependency type used without the main
    /// detail for that type.
    ///
    /// Following dependency details sets are considered to be invalid:
    /// 1. A set of dependency details which declares `branch`, `tag` or `rev` without `git`.
    pub fn validate(&self) -> anyhow::Result<()> {
        let DependencyDetails {
            git,
            branch,
            tag,
            rev,
            version,
            ipfs,
            namespace,
            path,
            ..
        } = self;

        if git.is_none() && (branch.is_some() || tag.is_some() || rev.is_some()) {
            bail!("Details reserved for git sources used without a git field");
        }

        if git.is_some() && branch.is_some() && tag.is_some() && rev.is_some() {
            bail!("Cannot specify `branch`, `tag`, and `rev` together for dependency with a Git source");
        }

        if git.is_some() && branch.is_some() && tag.is_some() {
            bail!("Cannot specify both `branch` and `tag` for dependency with a Git source");
        }

        if git.is_some() && rev.is_some() && tag.is_some() {
            bail!("Cannot specify both `rev` and `tag` for dependency with a Git source");
        }

        if git.is_some() && branch.is_some() && rev.is_some() {
            bail!("Cannot specify both `branch` and `rev` for dependency with a Git source");
        }

        if version.is_some() && git.is_some() {
            bail!("Both version and git details provided for same dependency");
        }

        if version.is_some() && ipfs.is_some() {
            bail!("Both version and ipfs details provided for same dependency");
        }

        if version.is_none() && namespace.is_some() {
            bail!("Namespace can only be specified for sources with version");
        }

        if version.is_some() && path.is_some() {
            bail!("Both version and path details provided for same dependency");
        }

        Ok(())
    }

    pub fn is_source_empty(&self) -> bool {
        self.git.is_none() && self.path.is_none() && self.ipfs.is_none()
    }
}

impl Dependency {
    /// The string of the `package` field if specified.
    pub fn package(&self) -> Option<&str> {
        match *self {
            Self::Simple(_) => None,
            Self::Detailed(ref det) => det.package.as_deref(),
        }
    }

    /// The string of the `version` field if specified.
    pub fn version(&self) -> Option<&str> {
        match *self {
            Self::Simple(ref version) => Some(version),
            Self::Detailed(ref det) => det.version.as_deref(),
        }
    }
}

impl PackageManifestFile {
    /// Returns an iterator over patches defined in underlying `PackageManifest` if this is a
    /// standalone package.
    ///
    /// If this package is a member of a workspace, patches are fetched from
    /// the workspace manifest file, ignoring any patch defined in the package
    /// manifest file, even if a patch section is not defined in the namespace.
    fn resolve_patches(&self) -> Result<impl Iterator<Item = (String, PatchMap)>> {
        if let Some(workspace) = self.workspace().ok().flatten() {
            // If workspace is defined, passing a local patch is a warning, but the global patch is used
            if self.patch.is_some() {
                println_warning("Patch for the non root package will be ignored.");
                println_warning(&format!(
                    "Specify patch at the workspace root: {}",
                    workspace.path().to_str().unwrap_or_default()
                ));
            }
            Ok(workspace
                .patch
                .as_ref()
                .cloned()
                .unwrap_or_default()
                .into_iter())
        } else {
            Ok(self.patch.as_ref().cloned().unwrap_or_default().into_iter())
        }
    }

    /// Retrieve the listed patches for the given name from underlying `PackageManifest` if this is
    /// a standalone package.
    ///
    /// If this package is a member of a workspace, patch is fetched from
    /// the workspace manifest file.
    pub fn resolve_patch(&self, patch_name: &str) -> Result<Option<PatchMap>> {
        Ok(self
            .resolve_patches()?
            .find(|(p_name, _)| patch_name == p_name.as_str())
            .map(|(_, patch)| patch))
    }

    /// Given the directory in which the file associated with this `PackageManifest` resides, produce the
    /// path to the entry file as specified in the manifest.
    ///
    /// This will always be a canonical path.
    pub fn entry_path(&self) -> PathBuf {
        self.dir()
            .join(constants::SRC_DIR)
            .join(&self.project.entry)
    }

    /// Produces the string of the entry point file.
    pub fn entry_string(&self) -> Result<Source> {
        let entry_path = self.entry_path();
        let entry_string = std::fs::read_to_string(entry_path)?;
        Ok(entry_string.as_str().into())
    }

    /// Parse and return the associated project's program type.
    pub fn program_type(&self) -> Result<TreeType> {
        let entry_string = self.entry_string()?;
        let handler = Handler::default();
        let parse_res = parse_tree_type(&handler, entry_string);

        parse_res.map_err(|_| {
            let (errors, _warnings, _infos) = handler.consume();
            parsing_failed(&self.project.name, &errors)
        })
    }

    /// Given the current directory and expected program type,
    /// determines whether the correct program type is present.
    pub fn check_program_type(&self, expected_types: &[TreeType]) -> Result<()> {
        let parsed_type = self.program_type()?;
        if !expected_types.contains(&parsed_type) {
            bail!(wrong_program_type(
                &self.project.name,
                expected_types,
                parsed_type
            ));
        } else {
            Ok(())
        }
    }

    /// Access the build profile associated with the given profile name.
    pub fn build_profile(&self, profile_name: &str) -> Option<&BuildProfile> {
        self.build_profile
            .as_ref()
            .and_then(|profiles| profiles.get(profile_name))
    }

    /// Given the name of a `path` dependency, returns the full canonical `Path` to the dependency.
    pub fn dep_path(&self, dep_name: &str) -> Option<PathBuf> {
        let dir = self.dir();
        let details = self.dep_detailed(dep_name)?;
        details.path.as_ref().and_then(|path_str| {
            let path = Path::new(path_str);
            match path.is_absolute() {
                true => Some(path.to_owned()),
                false => dir.join(path).canonicalize().ok(),
            }
        })
    }

    /// Returns the workspace manifest file if this `PackageManifestFile` is one of the members.
    pub fn workspace(&self) -> Result<Option<WorkspaceManifestFile>> {
        let parent_dir = match self.dir().parent() {
            None => return Ok(None),
            Some(dir) => dir,
        };
        let ws_manifest = match WorkspaceManifestFile::from_dir(parent_dir) {
            Ok(manifest) => manifest,
            Err(e) => {
                // Check if the error is missing workspace manifest file. Do not return that error if that
                // is the case as we do not want to return error if this is a single project
                // without a workspace.
                if e.to_string().contains("could not find") {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };
        if ws_manifest.is_member_path(self.dir())? {
            Ok(Some(ws_manifest))
        } else {
            Ok(None)
        }
    }

    /// Returns an immutable reference to the project name that this manifest file describes.
    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    /// Validate the `PackageManifestFile`.
    ///
    /// This checks:
    /// 1. Validity of the underlying `PackageManifest`.
    /// 2. Existence of the entry file.
    pub fn validate(&self) -> Result<()> {
        self.manifest.validate()?;
        let mut entry_path = self.path.clone();
        entry_path.pop();
        let entry_path = entry_path
            .join(constants::SRC_DIR)
            .join(&self.project.entry);
        if !entry_path.exists() {
            bail!(
                "failed to validate path from entry field {:?} in Forc manifest file.",
                self.project.entry
            )
        }

        // Check for nested packages.
        //
        // `path` is the path to manifest file. To start nested package search we need to start
        // from manifest's directory. So, last part of the path (the filename, "/forc.toml") needs
        // to be removed.
        let mut pkg_dir = self.path.to_path_buf();
        pkg_dir.pop();
        if let Some(nested_package) = find_nested_manifest_dir(&pkg_dir) {
            // remove file name from nested_package_manifest
            bail!("Nested packages are not supported, please consider separating the nested package at {} from the package at {}, or if it makes sense consider creating a workspace.", nested_package.display(), pkg_dir.display())
        }
        Ok(())
    }
}

impl GenericManifestFile for PackageManifestFile {
    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `std` is unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let manifest = PackageManifest::from_file(&path)?;
        let manifest_file = Self { manifest, path };
        manifest_file.validate()?;
        Ok(manifest_file)
    }

    /// Read the manifest from the `Forc.toml` in the directory specified by the given `path` or
    /// any of its parent directories.
    ///
    /// This is short for `PackageManifest::from_file`, but takes care of constructing the path to the
    /// file.
    fn from_dir<P: AsRef<Path>>(manifest_dir: P) -> Result<Self> {
        let manifest_dir = manifest_dir.as_ref();
        let dir = find_parent_manifest_dir(manifest_dir)
            .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the location of the lock file for `PackageManifestFile`.
    /// Checks if this PackageManifestFile corresponds to a workspace member and if that is the case
    /// returns the workspace level lock file's location.
    ///
    /// This will always be a canonical path.
    fn lock_path(&self) -> Result<PathBuf> {
        // Check if this package is in a workspace
        let workspace_manifest = self.workspace()?;
        if let Some(workspace_manifest) = workspace_manifest {
            workspace_manifest.lock_path()
        } else {
            Ok(self.dir().to_path_buf().join(constants::LOCK_FILE_NAME))
        }
    }

    fn member_manifests(&self) -> Result<MemberManifestFiles> {
        let mut member_manifest_files = BTreeMap::new();
        // Check if this package is in a workspace, in that case insert all member manifests
        if let Some(workspace_manifest_file) = self.workspace()? {
            for member_manifest in workspace_manifest_file.member_pkg_manifests()? {
                let member_manifest = member_manifest.with_context(|| "Invalid member manifest")?;
                member_manifest_files.insert(member_manifest.project.name.clone(), member_manifest);
            }
        } else {
            let member_name = &self.project.name;
            member_manifest_files.insert(member_name.clone(), self.clone());
        }

        Ok(member_manifest_files)
    }
}

impl PackageManifest {
    pub const DEFAULT_ENTRY_FILE_NAME: &'static str = "main.sw";

    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `std` is unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        // While creating a `ManifestFile` we need to check if the given path corresponds to a
        // package or a workspace. While doing so, we should be printing the warnings if the given
        // file parses so that we only see warnings for the correct type of manifest.
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        Self::from_string(contents)
    }

    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `std` is unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_string(contents: String) -> Result<Self> {
        // While creating a `ManifestFile` we need to check if the given path corresponds to a
        // package or a workspace. While doing so, we should be printing the warnings if the given
        // file parses so that we only see warnings for the correct type of manifest.
        let mut warnings = vec![];
        let toml_de = toml::de::Deserializer::new(&contents);
        let mut manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("unused manifest key: {path}");
            warnings.push(warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        for warning in warnings {
            println_warning(&warning);
        }
        manifest.implicitly_include_std_if_missing();
        manifest.implicitly_include_default_build_profiles_if_missing();
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the `PackageManifest`.
    ///
    /// This checks:
    /// 1. The project and organization names against a set of reserved/restricted keywords and patterns.
    /// 2. The validity of the details provided. Makes sure that there are no mismatching detail
    ///    declarations (to prevent mixing details specific to certain types).
    /// 3. The dependencies listed does not have an alias ("package" field) that is the same as package name.
    pub fn validate(&self) -> Result<()> {
        validate_project_name(&self.project.name)?;
        if let Some(ref org) = self.project.organization {
            validate_name(org, "organization name")?;
        }
        for (dep_name, dependency_details) in self.deps_detailed() {
            dependency_details.validate()?;
            if dependency_details
                .package
                .as_ref()
                .is_some_and(|package_alias| package_alias == &self.project.name)
            {
                bail!(format!("Dependency \"{dep_name}\" declares an alias (\"package\" field) that is the same as project name"))
            }
            if dep_name == &self.project.name {
                bail!(format!(
                    "Dependency \"{dep_name}\" collides with project name."
                ))
            }
        }
        Ok(())
    }

    /// Given a directory to a forc project containing a `Forc.toml`, read the manifest.
    ///
    /// This is short for `PackageManifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let manifest_dir =
            find_parent_manifest_dir(dir).ok_or_else(|| manifest_file_missing(dir))?;
        let file_path = manifest_dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(file_path)
    }

    /// Produce an iterator yielding all listed dependencies.
    pub fn deps(&self) -> impl Iterator<Item = (&String, &Dependency)> {
        self.dependencies
            .as_ref()
            .into_iter()
            .flat_map(|deps| deps.iter())
    }

    /// Produce an iterator yielding all listed build profiles.
    pub fn build_profiles(&self) -> impl Iterator<Item = (&String, &BuildProfile)> {
        self.build_profile
            .as_ref()
            .into_iter()
            .flat_map(|deps| deps.iter())
    }

    /// Produce an iterator yielding all listed contract dependencies
    pub fn contract_deps(&self) -> impl Iterator<Item = (&String, &ContractDependency)> {
        self.contract_dependencies
            .as_ref()
            .into_iter()
            .flat_map(|deps| deps.iter())
    }

    /// Produce an iterator yielding all `Detailed` dependencies.
    pub fn deps_detailed(&self) -> impl Iterator<Item = (&String, &DependencyDetails)> {
        self.deps().filter_map(|(name, dep)| match dep {
            Dependency::Detailed(ref det) => Some((name, det)),
            Dependency::Simple(_) => None,
        })
    }

    /// Produce an iterator yielding all listed patches.
    pub fn patches(&self) -> impl Iterator<Item = (&String, &PatchMap)> {
        self.patch
            .as_ref()
            .into_iter()
            .flat_map(|patches| patches.iter())
    }

    /// Retrieve the listed patches for the given name.
    pub fn patch(&self, patch_name: &str) -> Option<&PatchMap> {
        self.patch
            .as_ref()
            .and_then(|patches| patches.get(patch_name))
    }

    /// Retrieve the proxy table for the package.
    pub fn proxy(&self) -> Option<&Proxy> {
        self.proxy.as_ref()
    }

    /// Check for the `std` package under `[dependencies]`. If it is missing, add
    /// `std` implicitly.
    ///
    /// This makes the common case of depending on `std` a lot smoother for most users, while still
    /// allowing for the uncommon case of custom `std` deps.
    fn implicitly_include_std_if_missing(&mut self) {
        use sway_types::constants::STD;
        // Don't include `std` if:
        // - this *is* `std`.
        // - `std` package is already specified.
        // - a dependency already exists with the name "std".
        if self.project.name == STD
            || self.pkg_dep(STD).is_some()
            || self.dep(STD).is_some()
            || !self.project.implicit_std.unwrap_or(true)
        {
            return;
        }
        // Add a `[dependencies]` table if there isn't one.
        let deps = self.dependencies.get_or_insert_with(Default::default);
        // Add the missing dependency.
        let std_dep = implicit_std_dep();
        deps.insert(STD.to_string(), std_dep);
    }

    /// Check for the `debug` and `release` packages under `[build-profile]`. If they are missing add them.
    /// If they are provided, use the provided `debug` or `release` so that they override the default `debug`
    /// and `release`.
    fn implicitly_include_default_build_profiles_if_missing(&mut self) {
        let build_profiles = self.build_profile.get_or_insert_with(Default::default);

        if build_profiles.get(BuildProfile::DEBUG).is_none() {
            build_profiles.insert(BuildProfile::DEBUG.into(), BuildProfile::debug());
        }
        if build_profiles.get(BuildProfile::RELEASE).is_none() {
            build_profiles.insert(BuildProfile::RELEASE.into(), BuildProfile::release());
        }
    }

    /// Retrieve a reference to the dependency with the given name.
    pub fn dep(&self, dep_name: &str) -> Option<&Dependency> {
        self.dependencies
            .as_ref()
            .and_then(|deps| deps.get(dep_name))
    }

    /// Retrieve a reference to the dependency with the given name.
    pub fn dep_detailed(&self, dep_name: &str) -> Option<&DependencyDetails> {
        self.dep(dep_name).and_then(|dep| match dep {
            Dependency::Simple(_) => None,
            Dependency::Detailed(detailed) => Some(detailed),
        })
    }

    /// Retrieve a reference to the contract dependency with the given name.
    pub fn contract_dep(&self, contract_dep_name: &str) -> Option<&ContractDependency> {
        self.contract_dependencies
            .as_ref()
            .and_then(|contract_dependencies| contract_dependencies.get(contract_dep_name))
    }

    /// Retrieve a reference to the contract dependency with the given name.
    pub fn contract_dependency_detailed(
        &self,
        contract_dep_name: &str,
    ) -> Option<&DependencyDetails> {
        self.contract_dep(contract_dep_name)
            .and_then(|contract_dep| match &contract_dep.dependency {
                Dependency::Simple(_) => None,
                Dependency::Detailed(detailed) => Some(detailed),
            })
    }

    /// Finds and returns the name of the dependency associated with a package of the specified
    /// name if there is one.
    ///
    /// Returns `None` in the case that no dependencies associate with a package of the given name.
    fn pkg_dep<'a>(&'a self, pkg_name: &str) -> Option<&'a str> {
        for (dep_name, dep) in self.deps() {
            if dep.package().unwrap_or(dep_name) == pkg_name {
                return Some(dep_name);
            }
        }
        None
    }
}

impl std::ops::Deref for PackageManifestFile {
    type Target = PackageManifest;
    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

/// The definition for the implicit `std` dependency.
///
/// This can be configured using environment variables:
/// - use `FORC_IMPLICIT_STD_PATH` for the path for the std-lib;
/// - use `FORC_IMPLICIT_STD_GIT`, `FORC_IMPLICIT_STD_GIT_TAG` and/or `FORC_IMPLICIT_STD_GIT_BRANCH` to configure
///   the git repo of the std-lib.
fn implicit_std_dep() -> Dependency {
    if let Ok(path) = std::env::var("FORC_IMPLICIT_STD_PATH") {
        return Dependency::Detailed(DependencyDetails {
            path: Some(path),
            ..Default::default()
        });
    }

    let tag = std::env::var("FORC_IMPLICIT_STD_GIT_TAG").ok();
    let branch = std::env::var("FORC_IMPLICIT_STD_GIT_BRANCH").ok();
    let git_target = std::env::var("FORC_IMPLICIT_STD_GIT").ok();

    // If any of the git based std variables is set, we select the git version
    // for std.
    let det = if tag.is_some() || branch.is_some() || git_target.is_some() {
        const SWAY_GIT_REPO_URL: &str = "https://github.com/fuellabs/sway";
        // Here, we use the `forc-pkg` crate version formatted with the `v` prefix (e.g. "v1.2.3"),
        // or the revision commit hash (e.g. "abcdefg").
        //
        // This git tag or revision is used during `PackageManifest` construction to pin the version of the
        // implicit `std` dependency to the `forc-pkg` version.
        //
        // This is important to ensure that the version of `sway-core` that is baked into `forc-pkg` is
        // compatible with the version of the `std` lib.
        let tag = tag.unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));

        // only use tag/rev if the branch is None
        let tag = branch.as_ref().map_or_else(|| Some(tag), |_| None);
        let mut det = DependencyDetails {
            git: git_target.or_else(|| Some(SWAY_GIT_REPO_URL.to_string())),
            tag,
            branch,
            ..Default::default()
        };

        if let Some((_, build_metadata)) = det.tag.as_ref().and_then(|tag| tag.split_once('+')) {
            // Nightlies are in the format v<version>+nightly.<date>.<hash>
            let rev = build_metadata.split('.').next_back().map(|r| r.to_string());

            // If some revision is available and parsed from the 'nightly' build metadata,
            // we always prefer the revision over the tag.
            det.tag = None;
            det.rev = rev;
        };
        det
    } else {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        DependencyDetails {
            version: Some(current_version),
            ..Default::default()
        }
    };

    Dependency::Detailed(det)
}

fn default_entry() -> String {
    PackageManifest::DEFAULT_ENTRY_FILE_NAME.to_string()
}

fn default_url() -> String {
    constants::DEFAULT_NODE_URL.into()
}

/// A [WorkspaceManifest] that was deserialized from a file at a particular path.
#[derive(Clone, Debug)]
pub struct WorkspaceManifestFile {
    /// The derserialized `Forc.toml`
    manifest: WorkspaceManifest,
    /// The path from which the `Forc.toml` file was read.
    path: PathBuf,
}

/// A direct mapping to `Forc.toml` if it is a WorkspaceManifest
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspaceManifest {
    workspace: Workspace,
    patch: Option<BTreeMap<String, PatchMap>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Workspace {
    pub members: Vec<PathBuf>,
    pub metadata: Option<toml::Value>,
}

impl WorkspaceManifestFile {
    /// Produce an iterator yielding all listed patches.
    pub fn patches(&self) -> impl Iterator<Item = (&String, &PatchMap)> {
        self.patch
            .as_ref()
            .into_iter()
            .flat_map(|patches| patches.iter())
    }

    /// Returns an iterator over relative paths of workspace members.
    pub fn members(&self) -> impl Iterator<Item = &PathBuf> + '_ {
        self.workspace.members.iter()
    }

    /// Returns an iterator over workspace member root directories.
    ///
    /// This will always return canonical paths.
    pub fn member_paths(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        Ok(self
            .workspace
            .members
            .iter()
            .map(|member| self.dir().join(member)))
    }

    /// Returns an iterator over workspace member package manifests.
    pub fn member_pkg_manifests(
        &self,
    ) -> Result<impl Iterator<Item = Result<PackageManifestFile>> + '_> {
        let member_paths = self.member_paths()?;
        let member_pkg_manifests = member_paths.map(PackageManifestFile::from_dir);
        Ok(member_pkg_manifests)
    }

    /// Check if given path corresponds to any workspace member's path
    pub fn is_member_path(&self, path: &Path) -> Result<bool> {
        Ok(self.member_paths()?.any(|member_path| member_path == path))
    }
}

impl GenericManifestFile for WorkspaceManifestFile {
    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that given members are
    /// not present in the manifest dir.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("Cannot get parent dir of {:?}", path))?;
        let manifest = WorkspaceManifest::from_file(&path)?;
        manifest.validate(parent)?;
        Ok(Self { manifest, path })
    }

    /// Read the manifest from the `Forc.toml` in the directory specified by the given `path` or
    /// any of its parent directories.
    ///
    /// This is short for `PackageManifest::from_file`, but takes care of constructing the path to the
    /// file.
    fn from_dir<P: AsRef<Path>>(manifest_dir: P) -> Result<Self> {
        let manifest_dir = manifest_dir.as_ref();
        let dir = find_parent_manifest_dir_with_check(manifest_dir, |possible_manifest_dir| {
            // Check if the found manifest file is a workspace manifest file or a standalone
            // package manifest file.
            let possible_path = possible_manifest_dir.join(constants::MANIFEST_FILE_NAME);
            // We should not continue to search if the given manifest is a workspace manifest with
            // some issues.
            //
            // If the error is missing field `workspace` (which happens when trying to read a
            // package manifest as a workspace manifest), look into the parent directories for a
            // legitimate workspace manifest. If the error returned is something else this is a
            // workspace manifest with errors, classify this as a workspace manifest but with
            // errors so that the errors will be displayed to the user.
            Self::from_file(possible_path)
                .err()
                .map(|e| !e.to_string().contains("missing field `workspace`"))
                .unwrap_or_else(|| true)
        })
        .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the location of the lock file for `WorkspaceManifestFile`.
    ///
    /// This will always be a canonical path.
    fn lock_path(&self) -> Result<PathBuf> {
        Ok(self.dir().to_path_buf().join(constants::LOCK_FILE_NAME))
    }

    fn member_manifests(&self) -> Result<MemberManifestFiles> {
        let mut member_manifest_files = BTreeMap::new();
        for member_manifest in self.member_pkg_manifests()? {
            let member_manifest = member_manifest.with_context(|| "Invalid member manifest")?;
            member_manifest_files.insert(member_manifest.project.name.clone(), member_manifest);
        }

        Ok(member_manifest_files)
    }
}

impl WorkspaceManifest {
    /// Given a path to a `Forc.toml`, read it and construct a `WorkspaceManifest`.
    pub fn from_file(path: &Path) -> Result<Self> {
        // While creating a `ManifestFile` we need to check if the given path corresponds to a
        // package or a workspace. While doing so, we should be printing the warnings if the given
        // file parses so that we only see warnings for the correct type of manifest.
        let mut warnings = vec![];
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        let toml_de = toml::de::Deserializer::new(&manifest_str);
        let manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("unused manifest key: {path}");
            warnings.push(warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        for warning in warnings {
            println_warning(&warning);
        }
        Ok(manifest)
    }

    /// Validate the `WorkspaceManifest`
    ///
    /// This checks if the listed members in the `WorkspaceManifest` are indeed in the given `Forc.toml`'s directory.
    pub fn validate(&self, path: &Path) -> Result<()> {
        let mut pkg_name_to_paths: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for member in &self.workspace.members {
            let member_path = path.join(member).join("Forc.toml");
            if !member_path.exists() {
                bail!(
                    "{:?} is listed as a member of the workspace but {:?} does not exists",
                    &member,
                    member_path
                );
            }
            if Self::from_file(&member_path).is_ok() {
                bail!("Unexpected nested workspace '{}'. Workspaces are currently only allowed in the project root.", member.display());
            };

            let member_manifest_file = PackageManifestFile::from_file(member_path.clone())?;
            let pkg_name = member_manifest_file.manifest.project.name;
            pkg_name_to_paths
                .entry(pkg_name)
                .or_default()
                .push(member_path);
        }

        // Check for duplicate pkg name entries in member manifests of this workspace.
        let duplicate_pkg_lines = pkg_name_to_paths
            .iter()
            .filter_map(|(pkg_name, paths)| {
                if paths.len() > 1 {
                    let duplicate_paths = pkg_name_to_paths
                        .get(pkg_name)
                        .expect("missing duplicate paths");
                    Some(format!("{pkg_name}: {duplicate_paths:#?}"))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !duplicate_pkg_lines.is_empty() {
            let error_message = duplicate_pkg_lines.join("\n");
            bail!(
                "Duplicate package names detected in the workspace:\n\n{}",
                error_message
            );
        }
        Ok(())
    }
}

impl std::ops::Deref for WorkspaceManifestFile {
    type Target = WorkspaceManifest;
    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

/// Attempt to find a `Forc.toml` with the given project name within the given directory.
///
/// Returns the path to the package on success, or `None` in the case it could not be found.
pub fn find_within(dir: &Path, pkg_name: &str) -> Option<PathBuf> {
    use crate::source::reg::REG_DIR_NAME;
    use sway_types::constants::STD;
    const SWAY_STD_FOLDER: &str = "sway-lib-std";
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            entry
                .ok()
                .filter(|entry| entry.path().ends_with(constants::MANIFEST_FILE_NAME))
        })
        .find_map(|entry| {
            let path = entry.path();
            let manifest = PackageManifest::from_file(path).ok()?;
            // If the package is STD, make sure it is coming from correct folder.
            // That is either sway-lib-std, by fetching the sway repo (for std added as git dependency)
            // or from registry folder (for std added as a registry dependency).
            if (manifest.project.name == pkg_name && pkg_name != STD)
                || (manifest.project.name == STD
                    && path.components().any(|comp| {
                        comp.as_os_str() == SWAY_STD_FOLDER || comp.as_os_str() == REG_DIR_NAME
                    }))
            {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
}

/// The same as [find_within], but returns the package's project directory.
pub fn find_dir_within(dir: &Path, pkg_name: &str) -> Option<PathBuf> {
    find_within(dir, pkg_name).and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn deserialize_contract_dependency() {
        let contract_dep_str = r#"{"path": "../", "salt": "0x1111111111111111111111111111111111111111111111111111111111111111" }"#;

        let contract_dep_expected: ContractDependency =
            serde_json::from_str(contract_dep_str).unwrap();

        let dependency_det = DependencyDetails {
            path: Some("../".to_owned()),
            ..Default::default()
        };
        let dependency = Dependency::Detailed(dependency_det);
        let contract_dep = ContractDependency {
            dependency,
            salt: HexSalt::from_str(
                "0x1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        };
        assert_eq!(contract_dep, contract_dep_expected)
    }
    #[test]
    fn test_invalid_dependency_details_mixed_together() {
        let dependency_details_path_branch = DependencyDetails {
            version: None,
            path: Some("example_path/".to_string()),
            git: None,
            branch: Some("test_branch".to_string()),
            tag: None,
            package: None,
            rev: None,
            ipfs: None,
            namespace: None,
        };

        let dependency_details_branch = DependencyDetails {
            path: None,
            ..dependency_details_path_branch.clone()
        };

        let dependency_details_ipfs_branch = DependencyDetails {
            path: None,
            ipfs: Some("QmVxgEbiDDdHpG9AesCpZAqNvHYp1P3tWLFdrpUBWPMBcc".to_string()),
            ..dependency_details_path_branch.clone()
        };

        let dependency_details_path_tag = DependencyDetails {
            version: None,
            path: Some("example_path/".to_string()),
            git: None,
            branch: None,
            tag: Some("v0.1.0".to_string()),
            package: None,
            rev: None,
            ipfs: None,
            namespace: None,
        };

        let dependency_details_tag = DependencyDetails {
            path: None,
            ..dependency_details_path_tag.clone()
        };

        let dependency_details_ipfs_tag = DependencyDetails {
            path: None,
            ipfs: Some("QmVxgEbiDDdHpG9AesCpZAqNvHYp1P3tWLFdrpUBWPMBcc".to_string()),
            ..dependency_details_path_branch.clone()
        };

        let dependency_details_path_rev = DependencyDetails {
            version: None,
            path: Some("example_path/".to_string()),
            git: None,
            branch: None,
            tag: None,
            package: None,
            ipfs: None,
            rev: Some("9f35b8e".to_string()),
            namespace: None,
        };

        let dependency_details_rev = DependencyDetails {
            path: None,
            ..dependency_details_path_rev.clone()
        };

        let dependency_details_ipfs_rev = DependencyDetails {
            path: None,
            ipfs: Some("QmVxgEbiDDdHpG9AesCpZAqNvHYp1P3tWLFdrpUBWPMBcc".to_string()),
            ..dependency_details_path_branch.clone()
        };

        let expected_mismatch_error = "Details reserved for git sources used without a git field";
        assert_eq!(
            dependency_details_path_branch
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_ipfs_branch
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_path_tag
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_ipfs_tag
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_path_rev
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_ipfs_rev
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_branch
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_tag
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
        assert_eq!(
            dependency_details_rev
                .validate()
                .err()
                .map(|e| e.to_string()),
            Some(expected_mismatch_error.to_string())
        );
    }
    #[test]
    #[should_panic(expected = "Namespace can only be specified for sources with version")]
    fn test_error_namespace_without_version() {
        PackageManifest::from_dir("./tests/invalid/namespace_without_version").unwrap();
    }

    #[test]
    #[should_panic(expected = "Both version and git details provided for same dependency")]
    fn test_error_version_with_git_for_same_dep() {
        PackageManifest::from_dir("./tests/invalid/version_and_git_same_dep").unwrap();
    }

    #[test]
    #[should_panic(expected = "Both version and ipfs details provided for same dependency")]
    fn test_error_version_with_ipfs_for_same_dep() {
        PackageManifest::from_dir("./tests/invalid/version_and_ipfs_same_dep").unwrap();
    }

    #[test]
    #[should_panic(expected = "duplicate key `foo` in table `dependencies`")]
    fn test_error_duplicate_deps_definition() {
        PackageManifest::from_dir("./tests/invalid/duplicate_keys").unwrap();
    }

    #[test]
    fn test_error_duplicate_deps_definition_in_workspace() {
        // Load each project inside a workspace and load their patches
        // definition. There should be zero, because the file workspace file has
        // no patches
        //
        // The code also prints a warning to the stdout
        let workspace =
            WorkspaceManifestFile::from_dir("./tests/invalid/patch_workspace_and_package").unwrap();
        let projects: Vec<_> = workspace
            .member_pkg_manifests()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(projects.len(), 1);
        let patches: Vec<_> = projects[0].resolve_patches().unwrap().collect();
        assert_eq!(patches.len(), 0);

        // Load the same Forc.toml file but outside of a workspace. There should
        // be a single entry in the patch
        let patches: Vec<_> = PackageManifestFile::from_dir("./tests/test_package")
            .unwrap()
            .resolve_patches()
            .unwrap()
            .collect();
        assert_eq!(patches.len(), 1);
    }

    #[test]
    fn test_valid_dependency_details() {
        let dependency_details_path = DependencyDetails {
            version: None,
            path: Some("example_path/".to_string()),
            git: None,
            branch: None,
            tag: None,
            package: None,
            rev: None,
            ipfs: None,
            namespace: None,
        };

        let git_source_string = "https://github.com/FuelLabs/sway".to_string();
        let dependency_details_git_tag = DependencyDetails {
            version: None,
            path: None,
            git: Some(git_source_string.clone()),
            branch: None,
            tag: Some("v0.1.0".to_string()),
            package: None,
            rev: None,
            ipfs: None,
            namespace: None,
        };
        let dependency_details_git_branch = DependencyDetails {
            version: None,
            path: None,
            git: Some(git_source_string.clone()),
            branch: Some("test_branch".to_string()),
            tag: None,
            package: None,
            rev: None,
            ipfs: None,
            namespace: None,
        };
        let dependency_details_git_rev = DependencyDetails {
            version: None,
            path: None,
            git: Some(git_source_string),
            branch: None,
            tag: None,
            package: None,
            rev: Some("9f35b8e".to_string()),
            ipfs: None,
            namespace: None,
        };

        let dependency_details_ipfs = DependencyDetails {
            version: None,
            path: None,
            git: None,
            branch: None,
            tag: None,
            package: None,
            rev: None,
            ipfs: Some("QmVxgEbiDDdHpG9AesCpZAqNvHYp1P3tWLFdrpUBWPMBcc".to_string()),
            namespace: None,
        };

        assert!(dependency_details_path.validate().is_ok());
        assert!(dependency_details_git_tag.validate().is_ok());
        assert!(dependency_details_git_branch.validate().is_ok());
        assert!(dependency_details_git_rev.validate().is_ok());
        assert!(dependency_details_ipfs.validate().is_ok());
    }

    #[test]
    fn test_project_with_null_metadata() {
        let project = Project {
            authors: Some(vec!["Test Author".to_string()]),
            name: "test-project".to_string(),
            version: Some(Version::parse("0.1.0").unwrap()),
            description: Some("test description".to_string()),
            homepage: None,
            documentation: None,
            categories: None,
            keywords: None,
            repository: None,
            organization: None,
            license: "Apache-2.0".to_string(),
            entry: "main.sw".to_string(),
            implicit_std: None,
            forc_version: None,
            experimental: HashMap::new(),
            metadata: Some(toml::Value::from(toml::value::Table::new())),
            force_dbg_in_release: None,
        };

        let serialized = toml::to_string(&project).unwrap();
        let deserialized: Project = toml::from_str(&serialized).unwrap();

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.metadata, deserialized.metadata);
    }

    #[test]
    fn test_project_without_metadata() {
        let project = Project {
            authors: Some(vec!["Test Author".to_string()]),
            name: "test-project".to_string(),
            version: Some(Version::parse("0.1.0").unwrap()),
            description: Some("test description".to_string()),
            homepage: Some(Url::parse("https://example.com").unwrap()),
            documentation: Some(Url::parse("https://docs.example.com").unwrap()),
            categories: Some(vec!["test-category".to_string()]),
            keywords: Some(vec!["test-keyword".to_string()]),
            repository: Some(Url::parse("https://example.com").unwrap()),
            organization: None,
            license: "Apache-2.0".to_string(),
            entry: "main.sw".to_string(),
            implicit_std: None,
            forc_version: None,
            experimental: HashMap::new(),
            metadata: None,
            force_dbg_in_release: None,
        };

        let serialized = toml::to_string(&project).unwrap();
        let deserialized: Project = toml::from_str(&serialized).unwrap();

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.version, deserialized.version);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(project.homepage, deserialized.homepage);
        assert_eq!(project.documentation, deserialized.documentation);
        assert_eq!(project.repository, deserialized.repository);
        assert_eq!(project.metadata, deserialized.metadata);
        assert_eq!(project.metadata, None);
        assert_eq!(project.categories, deserialized.categories);
        assert_eq!(project.keywords, deserialized.keywords);
    }

    #[test]
    fn test_project_metadata_from_toml() {
        let toml_str = r#"
            name = "test-project"
            license = "Apache-2.0"
            entry = "main.sw"
            authors = ["Test Author"]
            description = "A test project"
            version = "1.0.0"
            keywords = ["test", "project"]
            categories = ["test"]

            [metadata]
            mykey = "https://example.com"
        "#;

        let project: Project = toml::from_str(toml_str).unwrap();
        assert!(project.metadata.is_some());

        let metadata = project.metadata.unwrap();
        let table = metadata.as_table().unwrap();

        assert_eq!(
            table.get("mykey").unwrap().as_str().unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_project_with_invalid_metadata() {
        // Test with invalid TOML syntax - unclosed table
        let invalid_toml = r#"
            name = "test-project"
            license = "Apache-2.0"
            entry = "main.sw"

            [metadata
            description = "Invalid TOML"
        "#;

        let result: Result<Project, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());

        // Test with invalid TOML syntax - invalid key
        let invalid_toml = r#"
            name = "test-project"
            license = "Apache-2.0"
            entry = "main.sw"

            [metadata]
            ] = "Invalid key"
        "#;

        let result: Result<Project, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());

        // Test with duplicate keys
        let invalid_toml = r#"
            name = "test-project"
            license = "Apache-2.0"
            entry = "main.sw"

            [metadata]
            nested = { key = "value1" }

            [metadata.nested]
            key = "value2"
        "#;

        let result: Result<Project, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("duplicate key `nested` in table `metadata`"));
    }

    #[test]
    fn test_metadata_roundtrip() {
        let original_toml = r#"
            name = "test-project"
            license = "Apache-2.0"
            entry = "main.sw"

            [metadata]
            boolean = true
            integer = 42
            float = 3.12
            string = "value"
            array = [1, 2, 3]
            mixed_array = [1, "two", true]

            [metadata.nested]
            key = "value2"
        "#;

        let project: Project = toml::from_str(original_toml).unwrap();
        let serialized = toml::to_string(&project).unwrap();
        let deserialized: Project = toml::from_str(&serialized).unwrap();

        // Verify that the metadata is preserved
        assert_eq!(project.metadata, deserialized.metadata);

        // Verify all types were preserved
        let table_val = project.metadata.unwrap();
        let table = table_val.as_table().unwrap();
        assert!(table.get("boolean").unwrap().as_bool().unwrap());
        assert_eq!(table.get("integer").unwrap().as_integer().unwrap(), 42);
        assert_eq!(table.get("float").unwrap().as_float().unwrap(), 3.12);
        assert_eq!(table.get("string").unwrap().as_str().unwrap(), "value");
        assert_eq!(table.get("array").unwrap().as_array().unwrap().len(), 3);
        assert!(table.get("nested").unwrap().as_table().is_some());
    }

    #[test]
    fn test_workspace_with_metadata() {
        let toml_str = r#"
            [workspace]
            members = ["package1", "package2"]

            [workspace.metadata]
            description = "A test workspace"
            version = "1.0.0"
            authors = ["Test Author"]
            homepage = "https://example.com"

            [workspace.metadata.ci]
            workflow = "main"
            timeout = 3600
        "#;

        let manifest: WorkspaceManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.workspace.metadata.is_some());

        let metadata = manifest.workspace.metadata.unwrap();
        let table = metadata.as_table().unwrap();

        assert_eq!(
            table.get("description").unwrap().as_str().unwrap(),
            "A test workspace"
        );
        assert_eq!(table.get("version").unwrap().as_str().unwrap(), "1.0.0");

        let ci = table.get("ci").unwrap().as_table().unwrap();
        assert_eq!(ci.get("workflow").unwrap().as_str().unwrap(), "main");
        assert_eq!(ci.get("timeout").unwrap().as_integer().unwrap(), 3600);
    }

    #[test]
    fn test_workspace_without_metadata() {
        let toml_str = r#"
            [workspace]
            members = ["package1", "package2"]
        "#;

        let manifest: WorkspaceManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.workspace.metadata.is_none());
    }

    #[test]
    fn test_workspace_empty_metadata() {
        let toml_str = r#"
            [workspace]
            members = ["package1", "package2"]

            [workspace.metadata]
        "#;

        let manifest: WorkspaceManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.workspace.metadata.is_some());
        let metadata = manifest.workspace.metadata.unwrap();
        assert!(metadata.as_table().unwrap().is_empty());
    }

    #[test]
    fn test_workspace_complex_metadata() {
        let toml_str = r#"
            [workspace]
            members = ["package1", "package2"]

            [workspace.metadata]
            numbers = [1, 2, 3]
            strings = ["a", "b", "c"]
            mixed = [1, "two", true]

            [workspace.metadata.nested]
            key = "value"

            [workspace.metadata.nested.deep]
            another = "value"
        "#;

        let manifest: WorkspaceManifest = toml::from_str(toml_str).unwrap();
        let metadata = manifest.workspace.metadata.unwrap();
        let table = metadata.as_table().unwrap();

        assert!(table.get("numbers").unwrap().as_array().is_some());
        assert!(table.get("strings").unwrap().as_array().is_some());
        assert!(table.get("mixed").unwrap().as_array().is_some());

        let nested = table.get("nested").unwrap().as_table().unwrap();
        assert_eq!(nested.get("key").unwrap().as_str().unwrap(), "value");

        let deep = nested.get("deep").unwrap().as_table().unwrap();
        assert_eq!(deep.get("another").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn test_workspace_metadata_roundtrip() {
        let original = WorkspaceManifest {
            workspace: Workspace {
                members: vec![PathBuf::from("package1"), PathBuf::from("package2")],
                metadata: Some(toml::Value::Table({
                    let mut table = toml::value::Table::new();
                    table.insert("key".to_string(), toml::Value::String("value".to_string()));
                    table
                })),
            },
            patch: None,
        };

        let serialized = toml::to_string(&original).unwrap();
        let deserialized: WorkspaceManifest = toml::from_str(&serialized).unwrap();

        assert_eq!(original.workspace.members, deserialized.workspace.members);
        assert_eq!(original.workspace.metadata, deserialized.workspace.metadata);
    }

    #[test]
    fn test_dependency_alias_project_name_collision() {
        let original_toml = r#"
        [project]
        authors = ["Fuel Labs <contact@fuel.sh>"]
        entry = "main.sw"
        license = "Apache-2.0"
        name = "lib_contract_abi"

        [dependencies]
        lib_contract = { path = "../lib_contract_abi/", package = "lib_contract_abi" }
        "#;

        let project = PackageManifest::from_string(original_toml.to_string());
        let err = project.unwrap_err();
        assert_eq!(err.to_string(), format!("Dependency \"lib_contract\" declares an alias (\"package\" field) that is the same as project name"))
    }

    #[test]
    fn test_dependency_name_project_name_collision() {
        let original_toml = r#"
        [project]
        authors = ["Fuel Labs <contact@fuel.sh>"]
        entry = "main.sw"
        license = "Apache-2.0"
        name = "lib_contract"

        [dependencies]
        lib_contract = { path = "../lib_contract_abi/", package = "lib_contract_abi" }
        "#;

        let project = PackageManifest::from_string(original_toml.to_string());
        let err = project.unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("Dependency \"lib_contract\" collides with project name.")
        )
    }
}
