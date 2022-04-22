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
use sway_utils::constants;

/// A [Manifest] that was deserialized from a file at a particular path.
#[derive(Debug)]
pub struct ManifestFile {
    /// The deserialized `Forc.toml`.
    manifest: Manifest,
    /// The path from which the `Forc.toml` file was read.
    path: PathBuf,
}

/// A direct mapping to a `Forc.toml`.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub network: Option<Network>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub authors: Option<Vec<String>>,
    pub name: String,
    pub organization: Option<String>,
    pub license: String,
    #[serde(default = "default_entry")]
    pub entry: String,
    pub implicit_std: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(default = "default_url")]
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
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
    /// implicitly. In this case, the `sway_git_tag` is used to specify the pinned commit at which
    /// we fetch `std`.
    pub fn from_file(path: PathBuf, sway_git_tag: &str) -> Result<Self> {
        let manifest = Manifest::from_file(&path, sway_git_tag)?;
        Ok(Self { manifest, path })
    }

    /// Read the manifest from the `Forc.toml` in the directory specified by the given `path` or
    /// any of its parent directories.
    ///
    /// This is short for `Manifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(manifest_dir: &Path, sway_git_tag: &str) -> Result<Self> {
        let dir = forc_util::find_manifest_dir(manifest_dir)
            .ok_or_else(|| manifest_file_missing(manifest_dir))?;
        let path = dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(path, sway_git_tag)
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
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path to the directory containing the `Forc.toml` from which this manfiest was loaded.
    pub fn dir(&self) -> &Path {
        self.path()
            .parent()
            .expect("failed to retrieve manifest directory")
    }

    /// Given the directory in which the file associated with this `Manifest` resides, produce the
    /// path to the entry file as specified in the manifest.
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
        let program_type = parse(entry_string, None);
        match program_type.value {
            Some(parse_tree) => Ok(parse_tree.tree_type),
            None => bail!(parsing_failed(&self.project.name, program_type.errors)),
        }
    }

    /// Given the current directory and expected program type, determines whether the correct program type is present.
    pub fn check_program_type(&self, expected_type: TreeType) -> Result<()> {
        let parsed_type = self.program_type()?;
        if parsed_type != expected_type {
            bail!(wrong_program_type(
                &self.project.name,
                expected_type,
                parsed_type
            ));
        } else {
            Ok(())
        }
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
    /// implicitly. In this case, the `sway_git_tag` is used to specify the pinned commit at which
    /// we fetch `std`.
    pub fn from_file(path: &Path, sway_git_tag: &str) -> Result<Self> {
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        let toml_de = &mut toml::de::Deserializer::new(&manifest_str);
        let mut manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {}", path);
            println_yellow_err(&warning);
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        manifest.implicitly_include_std_if_missing(sway_git_tag);
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
    pub fn from_dir(dir: &Path, sway_git_tag: &str) -> Result<Self> {
        let manifest_dir = find_manifest_dir(dir).ok_or_else(|| manifest_file_missing(dir))?;
        let file_path = manifest_dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(&file_path, sway_git_tag)
    }

    /// Produce an iterator yielding all listed dependencies.
    pub fn deps(&self) -> impl Iterator<Item = (&String, &Dependency)> {
        self.dependencies
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

    /// Check for the `core` and `std` packages under `[dependencies]`. If both are missing, add
    /// `std` implicitly.
    ///
    /// This makes the common case of depending on `std` a lot smoother for most users, while still
    /// allowing for the uncommon case of custom `core`/`std` deps.
    ///
    /// Note: If only `core` is specified, we are unable to implicitly add `std` as we cannot
    /// guarantee that the user's `core` is compatible with the implicit `std`.
    fn implicitly_include_std_if_missing(&mut self, sway_git_tag: &str) {
        const CORE: &str = "core";
        const STD: &str = "std";
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
        let std_dep = implicit_std_dep(sway_git_tag.to_string());
        deps.insert(STD.to_string(), std_dep);
    }

    /// Retrieve a reference to the dependency with the given name.
    pub fn dep(&self, dep_name: &str) -> Option<&Dependency> {
        self.dependencies
            .as_ref()
            .and_then(|deps| deps.get(dep_name))
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

impl std::ops::Deref for ManifestFile {
    type Target = Manifest;
    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

/// The definition for the implicit `std` dependency.
fn implicit_std_dep(sway_git_tag: String) -> Dependency {
    const SWAY_GIT_REPO_URL: &str = "https://github.com/fuellabs/sway";
    let det = DependencyDetails {
        git: Some(SWAY_GIT_REPO_URL.to_string()),
        tag: Some(sway_git_tag),
        ..Default::default()
    };
    Dependency::Detailed(det)
}

fn default_entry() -> String {
    Manifest::DEFAULT_ENTRY_FILE_NAME.to_string()
}

fn default_url() -> String {
    constants::DEFAULT_NODE_URL.into()
}
