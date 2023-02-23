use crate::{manifest::PackageManifestFile, source};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A package from the official registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Source {
    /// The base version specified for the package.
    pub version: semver::Version,
}

/// A pinned instance of the registry source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned {
    /// The registry package with base version.
    pub source: Source,
    /// The pinned version.
    pub version: semver::Version,
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, _ctx: source::PinCtx) -> anyhow::Result<(Self::Pinned, PathBuf)> {
        bail!("registry dependencies are not yet supported");
    }
}

impl source::Fetch for Pinned {
    fn fetch(&self, _ctx: source::PinCtx, _local: &Path) -> anyhow::Result<PackageManifestFile> {
        bail!("registry dependencies are not yet supported");
    }
}

impl source::DepPath for Pinned {
    fn dep_path(&self, _name: &str) -> anyhow::Result<source::DependencyPath> {
        bail!("registry dependencies are not yet supported");
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Registry(p)
    }
}
