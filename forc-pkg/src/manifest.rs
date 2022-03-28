use anyhow::{anyhow, bail};
use forc_util::{println_yellow_err, validate_name};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_utils::constants;

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

#[derive(Serialize, Deserialize, Debug)]
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

impl Manifest {
    pub const DEFAULT_ENTRY_FILE_NAME: &'static str = "main.sw";

    /// Given a path to a `Forc.toml`, read it and construct a `Manifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let manifest_str = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        let toml_de = &mut toml::de::Deserializer::new(&manifest_str);
        let manifest: Self = serde_ignored::deserialize(toml_de, |path| {
            let warning = format!("  WARNING! unused manifest key: {}", path);
            println_yellow_err(&warning).unwrap();
        })
        .map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        manifest.validate(path)?;
        Ok(manifest)
    }

    /// Given a directory to a forc project containing a `Forc.toml`, read the manifest.
    ///
    /// This is short for `Manifest::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(manifest_dir: &Path) -> anyhow::Result<Self> {
        let file_path = manifest_dir.join(constants::MANIFEST_FILE_NAME);
        Self::from_file(&file_path)
    }

    /// Validate the `Manifest`.
    ///
    /// This checks the project and organization names against a set of reserved/restricted
    /// keywords and patterns, and if a given entry point exists.
    pub fn validate(&self, path: &Path) -> anyhow::Result<()> {
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
        validate_name(&self.project.name, "package name")?;
        if let Some(ref org) = self.project.organization {
            validate_name(org, "organization name")?;
        }
        Ok(())
    }

    /// Given the directory in which the file associated with this `Manifest` resides, produce the
    /// path to the entry file as specified in the manifest.
    pub fn entry_path(&self, manifest_dir: &Path) -> PathBuf {
        manifest_dir
            .join(constants::SRC_DIR)
            .join(&self.project.entry)
    }

    /// Produces the string of the entry point file.
    pub fn entry_string(&self, manifest_dir: &Path) -> anyhow::Result<Arc<str>> {
        let entry_path = self.entry_path(manifest_dir);
        let entry_string = std::fs::read_to_string(&entry_path)?;
        Ok(Arc::from(entry_string))
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
}

fn default_entry() -> String {
    Manifest::DEFAULT_ENTRY_FILE_NAME.to_string()
}

fn default_url() -> String {
    constants::DEFAULT_NODE_URL.into()
}
