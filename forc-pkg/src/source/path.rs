use crate::{manifest::PackageManifestFile, pkg::PinnedId, source};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

/// A path to a directory with a `Forc.toml` manifest at its root.
pub type Source = PathBuf;

/// A pinned instance of a path source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned {
    /// The ID of the package that is the root of the subgraph of path dependencies that this
    /// package is a part of.
    ///
    /// In other words, when traversing the parents of this package, this is the ID of the first
    /// non-path ancestor package.
    ///
    /// As a result, this will always be either a git package or the root package.
    ///
    /// This allows for disambiguating path dependencies of the same name that have different path
    /// roots.
    pub path_root: PinnedId,
}

/// Error returned upon failed parsing of `SourcePathPinned::from_str`.
#[derive(Clone, Debug)]
pub struct SourcePathPinnedParseError;

impl Pinned {
    pub const PREFIX: &'static str = "path";
}

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // path+from-root-<id>
        write!(f, "{}+from-root-{}", Self::PREFIX, self.path_root)
    }
}

impl FromStr for Pinned {
    type Err = SourcePathPinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // path+from-root-<id>
        let s = s.trim();

        // Check for prefix at the start.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus) != Some(0) {
            return Err(SourcePathPinnedParseError);
        }
        let s = &s[prefix_plus.len()..];

        // Parse the `from-root-*` section.
        let path_root = s
            .split("from-root-")
            .nth(1)
            .ok_or(SourcePathPinnedParseError)?
            .parse()
            .map_err(|_| SourcePathPinnedParseError)?;

        Ok(Self { path_root })
    }
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, ctx: source::PinCtx) -> anyhow::Result<(Self::Pinned, PathBuf)> {
        let path_root = ctx.path_root();
        let pinned = Pinned { path_root };
        Ok((pinned, self.clone()))
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
        Ok(source::DependencyPath::Root(self.path_root))
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Path(p)
    }
}
