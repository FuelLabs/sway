use crate::{manifest::PackageManifestFile, source};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// Member source representation as a canonical path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Source(pub(super) PathBuf);

/// A pinned instance of a member source requires no information as it's a part
/// of the workspace.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned;

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "member")
    }
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, _ctx: source::PinCtx) -> anyhow::Result<(Self::Pinned, PathBuf)> {
        Ok((Pinned, self.0.clone()))
    }
}

impl source::Fetch for Pinned {
    fn fetch(&self, _ctx: source::PinCtx, local: &Path) -> anyhow::Result<PackageManifestFile> {
        let manifest = PackageManifestFile::from_dir(local)?;
        Ok(manifest)
    }
}

impl source::DepPath for Pinned {
    fn dep_path(&self, _name: &str) -> anyhow::Result<source::DependencyPath> {
        Ok(source::DependencyPath::Member)
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Member(p)
    }
}
