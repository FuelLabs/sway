//! Related to pinning, fetching, validating and caching the source for packages.
//!
//! To add a new source kind:
//!
//! 1. Add a new module.
//! 2. Create types providing implementations for each of the traits in this module.
//! 3. Add a variant to the `Source` and `Pinned` types in this module.
//! 4. Add variant support to the `from_manifest_dep` and `FromStr` implementations.

pub mod git;
pub mod path;
mod reg;

use crate::{
    manifest::{self, MemberManifestFiles, PackageManifestFile},
    pkg::{ManifestMap, PinnedId},
};
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map,
    fmt,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

/// Pin this source at a specific "version", return the local directory to fetch into.
trait Pin {
    type Pinned: Fetch + Hash;
    fn pin(&self, ctx: PinCtx) -> Result<(Self::Pinned, PathBuf)>;
}

/// Fetch (and optionally cache) a pinned instance of this source to the given path.
trait Fetch {
    fn fetch(&self, ctx: PinCtx, local: &Path) -> Result<PackageManifestFile>;
}

/// Given a parent manifest, return the canonical, local path for this source as a dependency.
trait DepPath {
    fn dep_path(&self, name: &str) -> Result<DependencyPath>;
}

type FetchId = u64;

/// Specifies a base source for a package.
///
/// - For registry packages, this includes a base version.
/// - For git packages, this includes a base git reference like a branch or tag.
///
/// Note that a `Source` does not specify a specific, pinned version. Rather, it specifies a source
/// at which the current latest version may be located.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Source {
    /// Used to refer to a workspace member project.
    Member(PathBuf),
    /// A git repo with a `Forc.toml` manifest at its root.
    Git(git::Source),
    /// A path to a directory with a `Forc.toml` manifest at its root.
    Path(path::Source),
    /// A forc project hosted on the official registry.
    Registry(reg::Source),
}

/// A pinned instance of the package source.
///
/// Specifies an exact version to use, or an exact commit in the case of git dependencies. The
/// pinned version or commit is updated upon creation of the lock file and on `forc update`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Pinned {
    Member,
    Git(git::Pinned),
    Path(path::Pinned),
    Registry(reg::Pinned),
}

#[derive(Clone)]
pub(crate) struct PinCtx<'a> {
    /// A unique hash associated with the process' current fetch pass.
    /// NOTE: Only to be used for creating temporary directories. Should not
    /// interact with anything that appears in the pinned output.
    pub(crate) fetch_id: FetchId,
    /// Within the context of a package graph fetch traversal, represents the current path root.
    pub(crate) path_root: PinnedId,
    /// Whether or not the fetch is occurring offline.
    pub(crate) offline: bool,
    /// The name of the package associated with this source.
    pub(crate) name: &'a str,
}

pub(crate) enum DependencyPath {
    /// The dependency is another member of the workspace.
    Member,
    /// The dependency is located at this specific path.
    ManifestPath(PathBuf),
    /// Path is pinned via manifest, relative to the given root node.
    Root(PinnedId),
}

/// A wrapper type for providing `Display` implementations for compiling msgs.
pub struct DisplayCompiling<'a, T> {
    source: &'a T,
    manifest_dir: &'a Path,
}

/// Error returned upon failed parsing of `SourcePinned::from_str`.
#[derive(Clone, Debug)]
pub struct PinnedParseError;

impl Source {
    /// Convert the given manifest `Dependency` declaration to a `Source`.
    pub fn from_manifest_dep(
        manifest_dir: &Path,
        dep: &manifest::Dependency,
        member_manifests: &MemberManifestFiles,
    ) -> Result<Self> {
        let source = match dep {
            manifest::Dependency::Simple(ref ver_str) => {
                bail!(
                    "Unsupported dependency declaration in \"{}\": `{}` - \
                    currently only `git` and `path` dependencies are supported",
                    manifest_dir.display(),
                    ver_str
                )
            }
            manifest::Dependency::Detailed(ref det) => match (&det.path, &det.version, &det.git) {
                (Some(relative_path), _, _) => {
                    let path = manifest_dir.join(relative_path);
                    let canonical_path = path.canonicalize().map_err(|e| {
                        anyhow!("Failed to canonicalize dependency path {:?}: {}", path, e)
                    })?;
                    // Check if path is a member of a workspace.
                    if member_manifests
                        .values()
                        .any(|pkg_manifest| pkg_manifest.dir() == canonical_path)
                    {
                        Source::Member(canonical_path)
                    } else {
                        Source::Path(canonical_path)
                    }
                }
                (_, _, Some(repo)) => {
                    let reference = match (&det.branch, &det.tag, &det.rev) {
                        (Some(branch), None, None) => git::Reference::Branch(branch.clone()),
                        (None, Some(tag), None) => git::Reference::Tag(tag.clone()),
                        (None, None, Some(rev)) => git::Reference::Rev(rev.clone()),
                        (None, None, None) => git::Reference::DefaultBranch,
                        _ => bail!(
                            "git dependencies support at most one reference: \
                                either `branch`, `tag` or `rev`"
                        ),
                    };
                    let repo = Url::parse(repo)?;
                    let source = git::Source { repo, reference };
                    Source::Git(source)
                }
                _ => {
                    bail!("unsupported set of fields for dependency: {:?}", dep);
                }
            },
        };
        Ok(source)
    }

    /// Convert the given manifest `Dependency` declaration to a source,
    /// applying any relevant patches from within the given `manifest` as
    /// necessary.
    pub fn from_manifest_dep_patched(
        manifest: &PackageManifestFile,
        dep_name: &str,
        dep: &manifest::Dependency,
        members: &MemberManifestFiles,
    ) -> Result<Self> {
        let unpatched = Self::from_manifest_dep(manifest.dir(), dep, members)?;
        unpatched.apply_patch(dep_name, manifest, members)
    }

    /// If a patch exists for this dependency source within the given project
    /// manifest, this returns the patch.
    fn dep_patch<'manifest>(
        &self,
        dep_name: &str,
        manifest: &'manifest PackageManifestFile,
    ) -> Option<&'manifest manifest::Dependency> {
        if let Source::Git(git) = self {
            if let Some(patches) = manifest.patch(git.repo.as_str()) {
                if let Some(patch) = patches.get(dep_name) {
                    return Some(patch);
                }
            }
        }
        None
    }

    /// If a patch exists for the dependency associated with this source within
    /// the given manifest, this returns a new `Source` with the patch applied.
    ///
    /// If no patch exists, this returns the original `Source`.
    pub fn apply_patch(
        &self,
        dep_name: &str,
        manifest: &PackageManifestFile,
        members: &MemberManifestFiles,
    ) -> Result<Self> {
        match self.dep_patch(dep_name, manifest) {
            Some(patch) => Self::from_manifest_dep(manifest.dir(), patch, members),
            None => Ok(self.clone()),
        }
    }

    /// Attempt to determine the pinned version or commit for the source.
    ///
    /// Also updates the `path_map` with a path to the local copy of the source.
    ///
    /// The `path_root` is required for `Path` dependencies and must specify the package that is the
    /// root of the current subgraph of path dependencies.
    pub(crate) fn pin(&self, ctx: PinCtx, manifests: &mut ManifestMap) -> Result<Pinned> {
        fn f<T>(source: &T, ctx: PinCtx, manifests: &mut ManifestMap) -> Result<T::Pinned>
        where
            T: Pin,
            T::Pinned: Clone,
            Pinned: From<T::Pinned>,
        {
            let (pinned, fetch_path) = source.pin(ctx.clone())?;
            // TODO: Could potentially omit `name`? Breaks all existing locks though...
            let id = PinnedId::new(ctx.name(), &Pinned::from(pinned.clone()));
            if let hash_map::Entry::Vacant(entry) = manifests.entry(id) {
                entry.insert(pinned.fetch(ctx, &fetch_path)?);
            }
            Ok(pinned)
        }
        match self {
            Source::Member(_path) => Ok(Pinned::Member),
            Source::Path(source) => Ok(Pinned::Path(f(source, ctx, manifests)?)),
            Source::Git(source) => Ok(Pinned::Git(f(source, ctx, manifests)?)),
            Source::Registry(source) => Ok(Pinned::Registry(f(source, ctx, manifests)?)),
        }
    }
}

impl Pinned {
    /// Return how the pinned source for a dependency can be found on the local file system.
    pub(crate) fn dep_path(&self, name: &str) -> Result<DependencyPath> {
        match self {
            Self::Member => Ok(DependencyPath::Member),
            Self::Path(pinned) => pinned.dep_path(name),
            Self::Git(pinned) => pinned.dep_path(name),
            Self::Registry(pinned) => pinned.dep_path(name),
        }
    }

    /// If the source is associated with a specific semver version, emit it.
    ///
    /// Used soley for the package lock file.
    pub fn semver(&self) -> Option<semver::Version> {
        match self {
            Self::Registry(reg) => Some(reg.source.version.clone()),
            _ => None,
        }
    }

    /// Wrap `self` in some type able to be formatted for the compiling output.
    ///
    /// This refers to `<source>` in the following:
    /// ```ignore
    /// Compiling <kind> <name> (<source>)
    /// ```
    pub fn display_compiling<'a>(&'a self, manifest_dir: &'a Path) -> DisplayCompiling<'a, Self> {
        DisplayCompiling {
            source: self,
            manifest_dir,
        }
    }
}

impl<'a> PinCtx<'a> {
    fn fetch_id(&self) -> FetchId {
        self.fetch_id
    }
    fn path_root(&self) -> PinnedId {
        self.path_root
    }
    fn offline(&self) -> bool {
        self.offline
    }
    fn name(&self) -> &str {
        self.name
    }
}

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Member => write!(f, "member"),
            Self::Path(src) => src.fmt(f),
            Self::Git(src) => src.fmt(f),
            Self::Registry(_reg) => todo!("pkg registries not yet implemented"),
        }
    }
}

impl<'a> fmt::Display for DisplayCompiling<'a, Pinned> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source {
            Pinned::Member => self.manifest_dir.display().fmt(f),
            Pinned::Path(_src) => self.manifest_dir.display().fmt(f),
            Pinned::Git(src) => src.fmt(f),
            Pinned::Registry(_src) => todo!("registry dependencies not yet implemented"),
        }
    }
}

impl FromStr for Pinned {
    type Err = PinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Also check `"root"` to support reading the legacy `Forc.lock` format and to
        // avoid breaking old projects.
        let source = if s == "root" || s == "member" {
            Self::Member
        } else if let Ok(src) = path::Pinned::from_str(s) {
            Self::Path(src)
        } else if let Ok(src) = git::Pinned::from_str(s) {
            Self::Git(src)
        } else {
            // TODO: Try parse registry source.
            return Err(PinnedParseError);
        };
        Ok(source)
    }
}

/// Produce a unique ID for a particular fetch pass.
///
/// This is used in the temporary git directory and allows for avoiding contention over the git
/// repo directory.
pub fn fetch_id(path: &Path, timestamp: std::time::Instant) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    hasher.finish()
}
