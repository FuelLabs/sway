use anyhow::anyhow;
use forc_util::{println_yellow_err, validate_name};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    #[deprecated = "use the authors field instead, the author field will be removed soon."]
    pub author: Option<String>,
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
}

impl Manifest {
    pub const DEFAULT_ENTRY_FILE_NAME: &'static str = "main.sw";

    /// Given a path to a `Forc.toml`, read it and construct a `Manifest`.
    ///
    /// This also `validate`s the manifest, returning an `Err` in the case that invalid names,
    /// fields were used.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let manifest = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("failed to read manifest at {:?}: {}", path, e))?;
        warn_unused_key(&manifest)?;
        let manifest: Self =
            toml::from_str(&manifest).map_err(|e| anyhow!("failed to parse manifest: {}.", e))?;
        manifest.validate()?;
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
    /// keywords and patterns.
    pub fn validate(&self) -> anyhow::Result<()> {
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

// Checks for unused manifest keys and displays a warning for each key present.
//
// This strategy is a combination of [cargo/util/toml/mod.rs](https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/util/toml/mod.rs#L100),
// [serde_ignored](https://docs.rs/serde_ignored/latest/serde_ignored/)'s default example and `forc-pkg::pkg::validate`.
fn warn_unused_key(manifest: &str) -> anyhow::Result<()> {
    let manifest_str = manifest;
    let toml = &mut toml::de::Deserializer::new(manifest_str);
    let mut unused_keys = BTreeSet::new();
    let _manifest: Manifest = serde_ignored::deserialize(toml, |path| {
        let mut key = String::new();
        verify_keypath(&mut key, &path);
        unused_keys.insert(key);
    })?;
    for key in unused_keys {
        println_yellow_err(&format!("  WARNING! unused manifest key: {key}")).unwrap();
    }
    Ok(())
}

// Some hackery needed to verify which keys aren't being used and add them to `unused_keys` as strings.
// Following the logic from `serde_ignored`'s documentation, `dst` is a deserialized string
fn verify_keypath(dst: &mut String, path: &serde_ignored::Path<'_>) {
    use serde_ignored::Path;

    match *path {
        Path::Root => {}
        Path::Seq { parent, index } => {
            verify_keypath(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(&index.to_string());
        }
        Path::Map { parent, ref key } => {
            verify_keypath(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(key);
        }
        Path::Some { parent }
        | Path::NewtypeVariant { parent }
        | Path::NewtypeStruct { parent } => verify_keypath(dst, parent),
    }
}

fn default_entry() -> String {
    Manifest::DEFAULT_ENTRY_FILE_NAME.to_string()
}

fn default_url() -> String {
    constants::DEFAULT_NODE_URL.into()
}
