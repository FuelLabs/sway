use crate::pkg::{manifest_file_missing, parsing_failed, wrong_program_type};
use anyhow::{anyhow, bail, Result};
use forc_util::{find_manifest_dir, println_yellow_err, validate_name};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use sway_core::{parse, TreeType};
pub use sway_types::ConfigTimeConstant;
use sway_utils::constants;

type PatchMap = BTreeMap<String, Dependency>;

/// A [Manifest] that was deserialized from a file at a particular path.
#[derive(Clone, Debug)]
pub struct ManifestFile {
    /// The deserialized `Forc.toml`.
    manifest: Manifest,
    /// The path from which the `Forc.toml` file was read.
    path: PathBuf,
}

/// A direct mapping to a `Forc.toml`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub network: Option<Network>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
    pub patch: Option<BTreeMap<String, PatchMap>>,
    /// A list of [configuration-time constants](https://github.com/FuelLabs/sway/issues/1498).
    pub constants: Option<BTreeMap<String, ConfigTimeConstant>>,
    build_profile: Option<BTreeMap<String, BuildProfile>>,
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
    pub(crate) path: Option<String>,
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
    pub print_ir: bool,
    pub print_finalized_asm: bool,
    pub print_intermediate_asm: bool,
    pub silent: bool,
    pub time_phases: bool,
    pub generate_logged_types: bool,
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

impl ManifestFile {
    /// Given a path to a `Forc.toml`, read it and construct a `Manifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `core` and `std` are unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;
        let manifest = Manifest::from_file(&path)?;
        Ok(Self { manifest, path })
    }

    /// Read the manifest from the `Forc.toml` in the directory specified by the given `path` or
    /// any of its parent directories.
    ///
    /// This is short for `Manifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(manifest_dir: &Path) -> Result<Self> {
        let dir = forc_util::find_manifest_dir(manifest_dir)
            .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path)
    }

    /// Validate the `Manifest`.
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

    /// Given the directory in which the file associated with this `Manifest` resides, produce the
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
        let entry_string = std::fs::read_to_string(&entry_path)?;
        Ok(Arc::from(entry_string))
    }

    /// Parse and return the associated project's program type.
    pub fn program_type(&self) -> Result<TreeType> {
        let entry_string = self.entry_string()?;
        let parse_res = parse(entry_string, None);
        match parse_res.value {
            Some(parse_program) => Ok(parse_program.kind),
            None => bail!(parsing_failed(&self.project.name, parse_res.errors)),
        }
    }

    /// Given the current directory and expected program type, determines whether the correct program type is present.
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
}

impl Manifest {
    pub const DEFAULT_ENTRY_FILE_NAME: &'static str = "main.sw";

    /// Given a path to a `Forc.toml`, read it and construct a `Manifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    ///
    /// If `core` and `std` are unspecified, `std` will be added to the `dependencies` table
    /// implicitly. In this case, the git tag associated with the version of this crate is used to
    /// specify the pinned commit at which we fetch `std`.
    pub fn from_file(path: &Path) -> Result<Self> {
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        let toml_de = &mut toml::de::Deserializer::new(&manifest_str);
        let mut manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {}", path);
            println_yellow_err(&warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        manifest.implicitly_include_std_if_missing();
        manifest.implicitly_include_default_build_profiles_if_missing();
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the `Manifest`.
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
    /// This is short for `Manifest::from_file`, but takes care of constructing the path to the
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
            print_ir: false,
            print_finalized_asm: false,
            print_intermediate_asm: false,
            silent: false,
            time_phases: false,
            generate_logged_types: false,
        }
    }

    pub fn release() -> Self {
        Self {
            print_ast: false,
            print_ir: false,
            print_finalized_asm: false,
            print_intermediate_asm: false,
            silent: false,
            time_phases: false,
            generate_logged_types: false,
        }
    }
}

impl std::ops::Deref for ManifestFile {
    type Target = Manifest;
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
    // This git tag or revision is used during `Manifest` construction to pin the version of the
    // implicit `std` dependency to the `forc-pkg` version.
    //
    // This is important to ensure that the version of `sway-core` that is baked into `forc-pkg` is
    // compatible with the version of the `std` lib.
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const SWAY_GIT_REPO_URL: &str = "https://github.com/fuellabs/sway";

    fn rev_from_build_metadata(build_metadata: &str) -> Option<String> {
        // Nightlies are in the format v<version>+nightly.<date>.<hash>
        build_metadata
            .split('.')
            .last()
            .map(|r| r.to_string())
            .filter(|s| !s.is_empty())
    }

    let sway_git_tag: String = "v".to_string() + VERSION;

    let mut det = DependencyDetails {
        git: Some(SWAY_GIT_REPO_URL.to_string()),
        tag: Some(sway_git_tag),
        ..Default::default()
    };

    if let Some((_tag, build_metadata)) = VERSION.split_once('+') {
        // If some revision is available and parsed from the 'nightly' build metadata,
        // we always prefer the revision over the tag.
        if let Some(rev) = rev_from_build_metadata(build_metadata) {
            det.tag = None;
            det.rev = Some(rev);
        }
    };

    Dependency::Detailed(det)
}

fn default_entry() -> String {
    Manifest::DEFAULT_ENTRY_FILE_NAME.to_string()
}

fn default_url() -> String {
    constants::DEFAULT_NODE_URL.into()
}
