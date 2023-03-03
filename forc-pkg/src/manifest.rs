use crate::pkg::{manifest_file_missing, parsing_failed, wrong_program_type};
use anyhow::{anyhow, bail, Context, Result};
use forc_tracing::println_yellow_err;
use forc_util::{find_manifest_dir, validate_name};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

use sway_core::{fuel_prelude::fuel_tx, language::parsed::TreeType, parse_tree_type, BuildTarget};
pub use sway_types::ConfigTimeConstant;
use sway_utils::constants;

/// The name of a workspace member package.
pub type MemberName = String;
/// A manifest for each workspace member, or just one manifest if working with a single package
pub type MemberManifestFiles = BTreeMap<MemberName, PackageManifestFile>;

pub enum ManifestFile {
    Package(Box<PackageManifestFile>),
    Workspace(WorkspaceManifestFile),
}

impl ManifestFile {
    /// Returns a `PackageManifestFile` if the path is within a package directory, otherwise
    /// returns a `WorkspaceManifestFile` if within a workspace directory.
    pub fn from_dir(manifest_dir: &Path) -> Result<Self> {
        let maybe_pkg_manifest = PackageManifestFile::from_dir(manifest_dir);
        let manifest_file = if let Err(e) = maybe_pkg_manifest {
            if e.to_string().contains("missing field `project`") {
                // This might be a workspace manifest file
                let workspace_manifest_file = WorkspaceManifestFile::from_dir(manifest_dir)?;
                ManifestFile::Workspace(workspace_manifest_file)
            } else {
                bail!("{}", e)
            }
        } else if let Ok(pkg_manifest) = maybe_pkg_manifest {
            ManifestFile::Package(Box::new(pkg_manifest))
        } else {
            bail!("Cannot find a valid `Forc.toml` at {:?}", manifest_dir)
        };
        Ok(manifest_file)
    }

    /// Returns a `PackageManifestFile` if the path is pointing to package manifest, otherwise
    /// returns a `WorkspaceManifestFile` if it is pointing to a workspace manifest.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let maybe_pkg_manifest = PackageManifestFile::from_file(path.clone());
        let manifest_file = if let Err(e) = maybe_pkg_manifest {
            if e.to_string().contains("missing field `project`") {
                // This might be a workspace manifest file
                let workspace_manifest_file = WorkspaceManifestFile::from_file(path)?;
                ManifestFile::Workspace(workspace_manifest_file)
            } else {
                bail!("{}", e)
            }
        } else if let Ok(pkg_manifest) = maybe_pkg_manifest {
            ManifestFile::Package(Box::new(pkg_manifest))
        } else {
            bail!("Cannot find a valid `Forc.toml` at {:?}", path)
        };
        Ok(manifest_file)
    }

    /// The path to the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn path(&self) -> &Path {
        match self {
            ManifestFile::Package(pkg_manifest_file) => pkg_manifest_file.path(),
            ManifestFile::Workspace(workspace_manifest_file) => workspace_manifest_file.path(),
        }
    }

    /// The path to the directory containing the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn dir(&self) -> &Path {
        self.path()
            .parent()
            .expect("failed to retrieve manifest directory")
    }

    /// Returns manifest file map from member name to the corresponding package manifest file
    pub fn member_manifests(&self) -> Result<MemberManifestFiles> {
        let mut member_manifest_files = BTreeMap::new();
        match self {
            ManifestFile::Package(pkg_manifest_file) => {
                // Check if this package is in a workspace, in that case insert all member manifests
                if let Some(workspace_manifest_file) = pkg_manifest_file.workspace()? {
                    for member_manifest in workspace_manifest_file.member_pkg_manifests()? {
                        let member_manifest =
                            member_manifest.with_context(|| "Invalid member manifest")?;
                        member_manifest_files
                            .insert(member_manifest.project.name.clone(), member_manifest);
                    }
                } else {
                    let member_name = &pkg_manifest_file.project.name;
                    member_manifest_files.insert(member_name.clone(), *pkg_manifest_file.clone());
                }
            }
            ManifestFile::Workspace(workspace_manifest_file) => {
                for member_manifest in workspace_manifest_file.member_pkg_manifests()? {
                    let member_manifest =
                        member_manifest.with_context(|| "Invalid member manifest")?;
                    member_manifest_files
                        .insert(member_manifest.project.name.clone(), member_manifest);
                }
            }
        }
        Ok(member_manifest_files)
    }

    /// Returns the path of the lock file for the given ManifestFile
    pub fn lock_path(&self) -> Result<PathBuf> {
        match self {
            ManifestFile::Package(pkg_manifest) => pkg_manifest.lock_path(),
            ManifestFile::Workspace(workspace_manifest) => Ok(workspace_manifest.lock_path()),
        }
    }
}

type PatchMap = BTreeMap<String, Dependency>;

/// A [PackageManifest] that was deserialized from a file at a particular path.
#[derive(Clone, Debug)]
pub struct PackageManifestFile {
    /// The deserialized `Forc.toml`.
    manifest: PackageManifest,
    /// The path from which the `Forc.toml` file was read.
    path: PathBuf,
}

/// A direct mapping to a `Forc.toml`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PackageManifest {
    pub project: Project,
    pub network: Option<Network>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
    pub patch: Option<BTreeMap<String, PatchMap>>,
    /// A list of [configuration-time constants](https://github.com/FuelLabs/sway/issues/1498).
    pub constants: Option<BTreeMap<String, ConfigTimeConstant>>,
    pub build_target: Option<BTreeMap<String, BuildTarget>>,
    build_profile: Option<BTreeMap<String, BuildProfile>>,
    pub contract_dependencies: Option<BTreeMap<String, ContractDependency>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub authors: Option<Vec<String>>,
    pub name: String,
    pub organization: Option<String>,
    pub license: String,
    #[serde(default = "default_entry")]
    pub entry: String,
    pub implicit_std: Option<bool>,
    pub forc_version: Option<semver::Version>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(default = "default_url")]
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ContractDependency {
    #[serde(flatten)]
    pub dependency: Dependency,
    #[serde(default = "fuel_tx::Salt::default")]
    pub salt: fuel_tx::Salt,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDetails {
    pub(crate) version: Option<String>,
    pub path: Option<String>,
    pub(crate) git: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) package: Option<String>,
    pub(crate) rev: Option<String>,
}

/// Parameters to pass through to the `sway_core::BuildConfig` during compilation.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct BuildProfile {
    pub print_ast: bool,
    pub print_dca_graph: bool,
    pub print_ir: bool,
    pub print_finalized_asm: bool,
    pub print_intermediate_asm: bool,
    pub terse: bool,
    pub time_phases: bool,
    pub include_tests: bool,
    pub json_abi_with_callpaths: bool,
    pub error_on_warnings: bool,
}

impl Dependency {
    /// The string of the `package` field if specified.
    pub fn package(&self) -> Option<&str> {
        match *self {
            Self::Simple(_) => None,
            Self::Detailed(ref det) => det.package.as_deref(),
        }
    }
}

impl PackageManifestFile {
    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `core` and `std` are unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;
        let manifest = PackageManifest::from_file(&path)?;
        Ok(Self { manifest, path })
    }

    /// Read the manifest from the `Forc.toml` in the directory specified by the given `path` or
    /// any of its parent directories.
    ///
    /// This is short for `PackageManifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(manifest_dir: &Path) -> Result<Self> {
        let dir = forc_util::find_manifest_dir(manifest_dir)
            .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path)
    }

    /// Validate the `PackageManifest`.
    ///
    /// This checks the project and organization names against a set of reserved/restricted
    /// keywords and patterns, and if a given entry point exists.
    pub fn validate(&self, path: &Path) -> Result<()> {
        self.manifest.validate()?;
        let mut entry_path = path.to_path_buf();
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
        Ok(())
    }

    /// The path to the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path to the directory containing the `Forc.toml` from which this manfiest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn dir(&self) -> &Path {
        self.path()
            .parent()
            .expect("failed to retrieve manifest directory")
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
    pub fn entry_string(&self) -> Result<Arc<str>> {
        let entry_path = self.entry_path();
        let entry_string = std::fs::read_to_string(entry_path)?;
        Ok(Arc::from(entry_string))
    }

    /// Parse and return the associated project's program type.
    pub fn program_type(&self) -> Result<TreeType> {
        let entry_string = self.entry_string()?;
        let parse_res = parse_tree_type(entry_string);
        parse_res
            .value
            .ok_or_else(|| parsing_failed(&self.project.name, parse_res.errors))
    }

    /// Given the current directory and expected program type,
    /// determines whether the correct program type is present.
    pub fn check_program_type(&self, expected_types: Vec<TreeType>) -> Result<()> {
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
    /// Getter for the config time constants on the manifest.
    pub fn config_time_constants(&self) -> BTreeMap<String, ConfigTimeConstant> {
        self.constants.as_ref().cloned().unwrap_or_default()
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

    /// Returns the location of the lock file for `PackageManifestFile`.
    /// Checks if this PackageManifestFile corresponds to a workspace member and if that is the case
    /// returns the workspace level lock file's location.
    ///
    /// This will always be a canonical path.
    pub fn lock_path(&self) -> Result<PathBuf> {
        // Check if this package is in a workspace
        let workspace_manifest = self.workspace()?;
        if let Some(workspace_manifest) = workspace_manifest {
            Ok(workspace_manifest.lock_path())
        } else {
            Ok(self.dir().to_path_buf().join(constants::LOCK_FILE_NAME))
        }
    }
}

impl PackageManifest {
    pub const DEFAULT_ENTRY_FILE_NAME: &'static str = "main.sw";

    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `core` and `std` are unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_file(path: &Path) -> Result<Self> {
        // While creating a `ManifestFile` we need to check if the given path corresponds to a
        // package or a workspace. While doing so, we should be printing the warnings if the given
        // file parses so that we only see warnings for the correct type of manifest.
        let mut warnings = vec![];
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        let toml_de = &mut toml::de::Deserializer::new(&manifest_str);
        let mut manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {path}");
            warnings.push(warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        for warning in warnings {
            println_yellow_err(&warning);
        }
        manifest.implicitly_include_std_if_missing();
        manifest.implicitly_include_default_build_profiles_if_missing();
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the `PackageManifest`.
    ///
    /// This checks the project and organization names against a set of reserved/restricted
    /// keywords and patterns.
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.project.name, "package name")?;
        if let Some(ref org) = self.project.organization {
            validate_name(org, "organization name")?;
        }
        Ok(())
    }

    /// Given a directory to a forc project containing a `Forc.toml`, read the manifest.
    ///
    /// This is short for `PackageManifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let manifest_dir = find_manifest_dir(dir).ok_or_else(|| manifest_file_missing(dir))?;
        let file_path = manifest_dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(&file_path)
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

    /// Check for the `core` and `std` packages under `[dependencies]`. If both are missing, add
    /// `std` implicitly.
    ///
    /// This makes the common case of depending on `std` a lot smoother for most users, while still
    /// allowing for the uncommon case of custom `core`/`std` deps.
    ///
    /// Note: If only `core` is specified, we are unable to implicitly add `std` as we cannot
    /// guarantee that the user's `core` is compatible with the implicit `std`.
    fn implicitly_include_std_if_missing(&mut self) {
        use crate::{CORE, STD};
        // Don't include `std` if:
        // - this *is* `core` or `std`.
        // - either `core` or `std` packages are already specified.
        // - a dependency already exists with the name "std".
        if self.project.name == CORE
            || self.project.name == STD
            || self.pkg_dep(CORE).is_some()
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

    /// Retrieve the listed patches for the given name.
    pub fn patch(&self, patch_name: &str) -> Option<&PatchMap> {
        self.patch
            .as_ref()
            .and_then(|patches| patches.get(patch_name))
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

impl BuildProfile {
    pub const DEBUG: &'static str = "debug";
    pub const RELEASE: &'static str = "release";
    pub const DEFAULT: &'static str = Self::DEBUG;

    pub fn debug() -> Self {
        Self {
            print_ast: false,
            print_dca_graph: false,
            print_ir: false,
            print_finalized_asm: false,
            print_intermediate_asm: false,
            terse: false,
            time_phases: false,
            include_tests: false,
            json_abi_with_callpaths: false,
            error_on_warnings: false,
        }
    }

    pub fn release() -> Self {
        Self {
            print_ast: false,
            print_dca_graph: false,
            print_ir: false,
            print_finalized_asm: false,
            print_intermediate_asm: false,
            terse: false,
            time_phases: false,
            include_tests: false,
            json_abi_with_callpaths: false,
            error_on_warnings: false,
        }
    }
}

impl std::ops::Deref for PackageManifestFile {
    type Target = PackageManifest;
    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

impl Default for BuildProfile {
    fn default() -> Self {
        Self::debug()
    }
}

/// The definition for the implicit `std` dependency.
fn implicit_std_dep() -> Dependency {
    // Here, we use the `forc-pkg` crate version formatted with the `v` prefix (e.g. "v1.2.3"),
    // or the revision commit hash (e.g. "abcdefg").
    //
    // This git tag or revision is used during `PackageManifest` construction to pin the version of the
    // implicit `std` dependency to the `forc-pkg` version.
    //
    // This is important to ensure that the version of `sway-core` that is baked into `forc-pkg` is
    // compatible with the version of the `std` lib.
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const SWAY_GIT_REPO_URL: &str = "https://github.com/fuellabs/sway";

    fn rev_from_build_metadata(build_metadata: &str) -> Option<String> {
        // Nightlies are in the format v<version>+nightly.<date>.<hash>
        build_metadata.split('.').last().map(|r| r.to_string())
    }

    let sway_git_tag: String = "v".to_string() + VERSION;

    let mut det = DependencyDetails {
        git: Some(SWAY_GIT_REPO_URL.to_string()),
        tag: Some(sway_git_tag),
        ..Default::default()
    };

    if let Some((_tag, build_metadata)) = VERSION.split_once('+') {
        let rev = rev_from_build_metadata(build_metadata);

        // If some revision is available and parsed from the 'nightly' build metadata,
        // we always prefer the revision over the tag.
        det.tag = None;
        det.rev = rev;
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Workspace {
    pub members: Vec<PathBuf>,
}

impl WorkspaceManifestFile {
    /// Given a path to a `Forc.toml`, read it and construct a `PackageManifest`
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that given members are
    /// not present in the manifest dir.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;
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
    pub fn from_dir(manifest_dir: &Path) -> Result<Self> {
        let dir = forc_util::find_manifest_dir_with_check(manifest_dir, |possible_manifest_dir| {
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
            // errors so that the erros will be displayed to the user.
            Self::from_file(possible_path)
                .err()
                .map(|e| !e.to_string().contains("missing field `workspace`"))
                .unwrap_or_else(|| true)
        })
        .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path)
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
        let member_pkg_manifests =
            member_paths.map(|member_path| PackageManifestFile::from_dir(&member_path));
        Ok(member_pkg_manifests)
    }

    /// The path to the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path to the directory containing the `Forc.toml` from which this manifest was loaded.
    ///
    /// This will always be a canonical path.
    pub fn dir(&self) -> &Path {
        self.path()
            .parent()
            .expect("failed to retrieve manifest directory")
    }

    /// Check if given path corresponds to any workspace member's path
    pub fn is_member_path(&self, path: &Path) -> Result<bool> {
        Ok(self.member_paths()?.any(|member_path| member_path == path))
    }

    /// Returns the location of the lock file for `WorkspaceManifestFile`.
    ///
    /// This will always be a canonical path.
    pub fn lock_path(&self) -> PathBuf {
        self.dir().to_path_buf().join(constants::LOCK_FILE_NAME)
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
        let toml_de = &mut toml::de::Deserializer::new(&manifest_str);
        let manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {path}");
            warnings.push(warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        for warning in warnings {
            println_yellow_err(&warning);
        }
        Ok(manifest)
    }

    /// Validate the `WorkspaceManifest`
    ///
    /// This checks if the listed members in the `WorkspaceManifest` are indeed in the given `Forc.toml`'s directory.
    pub fn validate(&self, path: &Path) -> Result<()> {
        let mut pkg_name_to_paths: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for member in self.workspace.members.iter() {
            let member_path = path.join(member).join("Forc.toml");
            if !member_path.exists() {
                bail!(
                    "{:?} is listed as a member of the workspace but {:?} does not exists",
                    &member,
                    member_path
                );
            }
            let member_manifest_file = PackageManifestFile::from_file(member_path.clone())?;
            let pkg_name = member_manifest_file.manifest.project.name;
            pkg_name_to_paths
                .entry(pkg_name)
                .or_default()
                .push(member_path);
        }

        // Check for duplicate pkg name entries in member manifests of this workspace.
        let duplciate_pkg_lines = pkg_name_to_paths
            .iter()
            .filter(|(_, paths)| paths.len() > 1)
            .map(|(pkg_name, _)| {
                let duplicate_paths = pkg_name_to_paths
                    .get(pkg_name)
                    .expect("missing duplicate paths");
                format!("{pkg_name}: {duplicate_paths:#?}")
            })
            .collect::<Vec<_>>();

        if !duplciate_pkg_lines.is_empty() {
            let error_message = duplciate_pkg_lines.join("\n");
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
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().ends_with(constants::MANIFEST_FILE_NAME))
        .find_map(|entry| {
            let path = entry.path();
            let manifest = PackageManifest::from_file(path).ok()?;
            if manifest.project.name == pkg_name {
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
