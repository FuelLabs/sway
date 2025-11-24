//! Related to pinning, fetching, validating and caching the source for packages.
//!
//! To add a new source kind:
//!
//! 1. Add a new module.
//! 2. Create types providing implementations for each of the traits in this module.
//! 3. Add a variant to the `Source` and `Pinned` types in this module.
//! 4. Add variant support to the `from_manifest_dep` and `FromStr` implementations.

pub mod git;
pub(crate) mod ipfs;
mod member;
pub mod path;
pub mod reg;

use self::git::Url;
use crate::manifest::GenericManifestFile;
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
use sway_utils::{DEFAULT_IPFS_GATEWAY_URL, DEFAULT_REGISTRY_IPFS_GATEWAY_URL};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IPFSNode {
    Local,
    WithUrl(String),
}

impl Default for IPFSNode {
    fn default() -> Self {
        Self::WithUrl(DEFAULT_IPFS_GATEWAY_URL.to_string())
    }
}

impl IPFSNode {
    /// Returns an IPFSNode configured to use the Fuel-operated IPFS gateway.
    pub fn fuel() -> Self {
        Self::WithUrl(DEFAULT_REGISTRY_IPFS_GATEWAY_URL.to_string())
    }

    /// Returns an IPFSNode configured to use the public IPFS gateway.
    pub fn public() -> Self {
        Self::WithUrl(DEFAULT_IPFS_GATEWAY_URL.to_string())
    }
}

impl FromStr for IPFSNode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PUBLIC" => {
                let url = sway_utils::constants::DEFAULT_IPFS_GATEWAY_URL;
                Ok(IPFSNode::WithUrl(url.to_string()))
            }
            "FUEL" => {
                let url = sway_utils::constants::DEFAULT_REGISTRY_IPFS_GATEWAY_URL;
                Ok(IPFSNode::WithUrl(url.to_string()))
            }
            "LOCAL" => Ok(IPFSNode::Local),
            url => Ok(IPFSNode::WithUrl(url.to_string())),
        }
    }
}

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
    Member(member::Source),
    /// A git repo with a `Forc.toml` manifest at its root.
    Git(git::Source),
    /// A path to a directory with a `Forc.toml` manifest at its root.
    Path(path::Source),
    /// A package described by its IPFS CID.
    Ipfs(ipfs::Source),
    /// A forc project hosted on the official registry.
    Registry(reg::Source),
}

/// A pinned instance of the package source.
///
/// Specifies an exact version to use, or an exact commit in the case of git dependencies. The
/// pinned version or commit is updated upon creation of the lock file and on `forc update`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Pinned {
    Member(member::Pinned),
    Git(git::Pinned),
    Path(path::Pinned),
    Ipfs(ipfs::Pinned),
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
    /// The IPFS node to use for fetching IPFS sources.
    pub(crate) ipfs_node: &'a IPFSNode,
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
    /// Construct a source from path information collected from manifest file.
    fn with_path_dependency(
        relative_path: &Path,
        manifest_dir: &Path,
        member_manifests: &MemberManifestFiles,
    ) -> Result<Self> {
        let path = manifest_dir.join(relative_path);
        let canonical_path = path
            .canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize dependency path {:?}: {}", path, e))?;
        // Check if path is a member of a workspace.
        if member_manifests
            .values()
            .any(|pkg_manifest| pkg_manifest.dir() == canonical_path)
        {
            Ok(Source::Member(member::Source(canonical_path)))
        } else {
            Ok(Source::Path(canonical_path))
        }
    }

    /// Construct a source from version information collected from manifest file.
    fn with_version_dependency(
        pkg_name: &str,
        version: &str,
        namespace: &reg::file_location::Namespace,
    ) -> Result<Self> {
        // TODO: update here once we are supporting non-exact versions (non `x.y.z` versions)
        // see: https://github.com/FuelLabs/sway/issues/7060
        let semver = semver::Version::parse(version)?;
        let source = reg::Source {
            version: semver,
            namespace: namespace.clone(),
            name: pkg_name.to_string(),
        };
        Ok(Source::Registry(source))
    }

    /// Convert the given manifest `Dependency` declaration to a `Source`.
    pub fn from_manifest_dep(
        manifest_dir: &Path,
        dep_name: &str,
        dep: &manifest::Dependency,
        member_manifests: &MemberManifestFiles,
    ) -> Result<Self> {
        let source = match dep {
            manifest::Dependency::Simple(ref ver_str) => Source::with_version_dependency(
                dep_name,
                ver_str,
                &reg::file_location::Namespace::Flat,
            )?,
            manifest::Dependency::Detailed(ref det) => {
                match (&det.path, &det.version, &det.git, &det.ipfs) {
                    (Some(relative_path), _, _, _) => {
                        let relative_path = PathBuf::from_str(relative_path)?;
                        Source::with_path_dependency(
                            &relative_path,
                            manifest_dir,
                            member_manifests,
                        )?
                    }
                    (_, _, Some(repo), _) => {
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
                        let repo = Url::from_str(repo)?;
                        let source = git::Source { repo, reference };
                        Source::Git(source)
                    }
                    (_, _, _, Some(ipfs)) => {
                        let cid = ipfs.parse()?;
                        let source = ipfs::Source(cid);
                        Source::Ipfs(source)
                    }
                    (None, Some(version), _, _) => {
                        let namespace = det.namespace.as_ref().map_or_else(
                            || reg::file_location::Namespace::Flat,
                            |ns| reg::file_location::Namespace::Domain(ns.to_string()),
                        );
                        Source::with_version_dependency(dep_name, version, &namespace)?
                    }
                    _ => {
                        bail!("unsupported set of fields for dependency: {:?}", dep);
                    }
                }
            }
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
        let unpatched = Self::from_manifest_dep(manifest.dir(), dep_name, dep, members)?;
        unpatched.apply_patch(dep_name, manifest, members)
    }

    /// If a patch exists for this dependency source within the given project
    /// manifest, this returns the patch.
    ///
    /// Supports patching both Git and Registry dependencies:
    /// - Git: [patch.'https://github.com/org/repo']
    /// - Registry: [patch.'forc.pub']
    ///
    /// Note: Quotes are required around patch keys containing dots to follow TOML spec.
    /// Without quotes, `[patch.forc.pub]` creates nested tables instead of a single key.
    fn dep_patch(
        &self,
        dep_name: &str,
        manifest: &PackageManifestFile,
    ) -> Result<Option<manifest::Dependency>> {
        // Helper to check if a patch exists for the given key
        let check_patches = |patch_key: &str| -> Result<Option<manifest::Dependency>> {
            let patches = manifest.resolve_patch(patch_key)?;
            Ok(patches.and_then(|p| p.get(dep_name).cloned()))
        };

        match self {
            Source::Git(git) => {
                let git_url = git.repo.to_string();
                check_patches(&git_url)
            }
            Source::Registry(reg_source) => {
                // Try namespace-specific patch first (more specific takes priority)
                if let reg::file_location::Namespace::Domain(ns) = &reg_source.namespace {
                    let namespaced_key = format!("{}/{}", reg::REGISTRY_PATCH_KEY, ns);
                    if let Some(patch) = check_patches(&namespaced_key)? {
                        return Ok(Some(patch));
                    }
                }

                // Fall back to generic registry patch
                check_patches(reg::REGISTRY_PATCH_KEY)
            }
            _ => Ok(None),
        }
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
        match self.dep_patch(dep_name, manifest)? {
            Some(patch) => Self::from_manifest_dep(manifest.dir(), dep_name, &patch, members),
            None => Ok(self.clone()),
        }
    }

    /// Attempt to determine the pinned version or commit for the source.
    ///
    /// Also updates the manifest map with a path to the local copy of the pkg.
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
            let id = PinnedId::new(ctx.name(), &Pinned::from(pinned.clone()));
            if let hash_map::Entry::Vacant(entry) = manifests.entry(id) {
                entry.insert(pinned.fetch(ctx, &fetch_path)?);
            }
            Ok(pinned)
        }
        match self {
            Source::Member(source) => Ok(Pinned::Member(f(source, ctx, manifests)?)),
            Source::Path(source) => Ok(Pinned::Path(f(source, ctx, manifests)?)),
            Source::Git(source) => Ok(Pinned::Git(f(source, ctx, manifests)?)),
            Source::Ipfs(source) => Ok(Pinned::Ipfs(f(source, ctx, manifests)?)),
            Source::Registry(source) => Ok(Pinned::Registry(f(source, ctx, manifests)?)),
        }
    }
}

impl Pinned {
    pub(crate) const MEMBER: Self = Self::Member(member::Pinned);

    /// Return how the pinned source for a dependency can be found on the local file system.
    pub(crate) fn dep_path(&self, name: &str) -> Result<DependencyPath> {
        match self {
            Self::Member(pinned) => pinned.dep_path(name),
            Self::Path(pinned) => pinned.dep_path(name),
            Self::Git(pinned) => pinned.dep_path(name),
            Self::Ipfs(pinned) => pinned.dep_path(name),
            Self::Registry(pinned) => pinned.dep_path(name),
        }
    }

    /// If the source is associated with a specific semver version, emit it.
    ///
    /// Used solely for the package lock file.
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

    /// Retrieve the unpinned instance of this source.
    pub fn unpinned(&self, path: &Path) -> Source {
        match self {
            Self::Member(_) => Source::Member(member::Source(path.to_owned())),
            Self::Git(git) => Source::Git(git.source.clone()),
            Self::Path(_) => Source::Path(path.to_owned()),
            Self::Ipfs(ipfs) => Source::Ipfs(ipfs::Source(ipfs.0.clone())),
            Self::Registry(reg) => Source::Registry(reg.source.clone()),
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
    fn ipfs_node(&self) -> &'a IPFSNode {
        self.ipfs_node
    }
}

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Member(src) => src.fmt(f),
            Self::Path(src) => src.fmt(f),
            Self::Git(src) => src.fmt(f),
            Self::Ipfs(src) => src.fmt(f),
            Self::Registry(src) => src.fmt(f),
        }
    }
}

impl fmt::Display for DisplayCompiling<'_, Pinned> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.source {
            Pinned::Member(_) => self.manifest_dir.display().fmt(f),
            Pinned::Path(_src) => self.manifest_dir.display().fmt(f),
            Pinned::Git(src) => src.fmt(f),
            Pinned::Ipfs(src) => src.fmt(f),
            Pinned::Registry(src) => src.fmt(f),
        }
    }
}

impl FromStr for Pinned {
    type Err = PinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Also check `"root"` to support reading the legacy `Forc.lock` format and to
        // avoid breaking old projects.
        let source = if s == "root" || s == "member" {
            Self::Member(member::Pinned)
        } else if let Ok(src) = path::Pinned::from_str(s) {
            Self::Path(src)
        } else if let Ok(src) = git::Pinned::from_str(s) {
            Self::Git(src)
        } else if let Ok(src) = ipfs::Pinned::from_str(s) {
            Self::Ipfs(src)
        } else if let Ok(src) = reg::Pinned::from_str(s) {
            Self::Registry(src)
        } else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Dependency, DependencyDetails};
    use std::collections::BTreeMap;

    /// Helper to create a minimal test manifest file with patch table
    fn create_test_manifest_file_with_patches(
        patches: BTreeMap<String, BTreeMap<String, Dependency>>,
    ) -> (tempfile::TempDir, PackageManifestFile) {
        // Create a minimal TOML string
        let mut toml_str = r#"[project]
name = "test_pkg"
license = "Apache-2.0"
entry = "main.sw"
implicit-std = false
"#
        .to_string();

        // Add patches if any
        if !patches.is_empty() {
            toml_str.push('\n');
            for (patch_key, patch_deps) in patches {
                toml_str.push_str(&format!("[patch.'{}']\n", patch_key));
                for (dep_name, dep) in patch_deps {
                    // Manually construct the dependency string
                    let dep_toml = match dep {
                        Dependency::Simple(ver) => format!(r#""{ver}""#),
                        Dependency::Detailed(det) => {
                            let mut parts = Vec::new();
                            if let Some(path) = &det.path {
                                parts.push(format!(r#"path = "{path}""#));
                            }
                            if let Some(git) = &det.git {
                                parts.push(format!(r#"git = "{git}""#));
                            }
                            if let Some(branch) = &det.branch {
                                parts.push(format!(r#"branch = "{branch}""#));
                            }
                            if let Some(tag) = &det.tag {
                                parts.push(format!(r#"tag = "{tag}""#));
                            }
                            if let Some(version) = &det.version {
                                parts.push(format!(r#"version = "{version}""#));
                            }
                            format!("{{ {} }}", parts.join(", "))
                        }
                    };
                    toml_str.push_str(&format!("{} = {}\n", dep_name, dep_toml));
                }
            }
        }

        // Create necessary directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();

        // Create a minimal main.sw file
        let main_sw_path = src_dir.join("main.sw");
        std::fs::write(&main_sw_path, "contract;").unwrap();

        // Write manifest file
        let manifest_path = temp_dir.path().join("Forc.toml");
        std::fs::write(&manifest_path, toml_str).unwrap();

        // Read back as PackageManifestFile
        let manifest_file = PackageManifestFile::from_file(&manifest_path).unwrap();

        (temp_dir, manifest_file)
    }

    /// Helper to create a path dependency
    fn path_dep(path: &str) -> Dependency {
        Dependency::Detailed(DependencyDetails {
            path: Some(path.to_string()),
            ..Default::default()
        })
    }

    /// Helper to create a git dependency
    fn git_dep(repo: &str, branch: &str) -> Dependency {
        Dependency::Detailed(DependencyDetails {
            git: Some(repo.to_string()),
            branch: Some(branch.to_string()),
            ..Default::default()
        })
    }

    #[test]
    fn test_registry_patch_flat_namespace() {
        // Create a registry source with flat namespace
        let source = Source::Registry(reg::Source {
            name: "std".to_string(),
            version: semver::Version::new(0, 63, 0),
            namespace: reg::file_location::Namespace::Flat,
        });

        // Create a manifest with a forc.pub patch
        let mut patches = BTreeMap::new();
        let mut forc_pub_patches = BTreeMap::new();
        forc_pub_patches.insert("std".to_string(), path_dep("../local-std"));
        patches.insert("forc.pub".to_string(), forc_pub_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that the patch is found
        let patch = source.dep_patch("std", &manifest_file).unwrap();
        assert!(
            patch.is_some(),
            "Should find patch for flat namespace registry dependency"
        );

        let patch = patch.unwrap();
        match patch {
            Dependency::Detailed(det) => {
                assert_eq!(det.path, Some("../local-std".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_registry_patch_domain_namespace() {
        // Create a registry source with domain namespace
        let source = Source::Registry(reg::Source {
            name: "fuel-core".to_string(),
            version: semver::Version::new(1, 0, 0),
            namespace: reg::file_location::Namespace::Domain("com/fuel".to_string()),
        });

        // Create a manifest with a namespaced patch
        let mut patches = BTreeMap::new();
        let mut namespaced_patches = BTreeMap::new();
        namespaced_patches.insert("fuel-core".to_string(), path_dep("../local-fuel-core"));
        patches.insert("forc.pub/com/fuel".to_string(), namespaced_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that the patch is found
        let patch = source.dep_patch("fuel-core", &manifest_file).unwrap();
        assert!(
            patch.is_some(),
            "Should find patch for domain namespace registry dependency"
        );

        let patch = patch.unwrap();
        match patch {
            Dependency::Detailed(det) => {
                assert_eq!(det.path, Some("../local-fuel-core".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_registry_patch_namespace_priority() {
        // Create a registry source with domain namespace
        let source = Source::Registry(reg::Source {
            name: "my-lib".to_string(),
            version: semver::Version::new(2, 0, 0),
            namespace: reg::file_location::Namespace::Domain("com/myorg".to_string()),
        });

        // Create a manifest with BOTH namespaced and generic patches
        let mut patches = BTreeMap::new();

        // Namespace-specific patch
        let mut namespaced_patches = BTreeMap::new();
        namespaced_patches.insert("my-lib".to_string(), path_dep("../namespaced-lib"));
        patches.insert("forc.pub/com/myorg".to_string(), namespaced_patches);

        // Generic patch
        let mut generic_patches = BTreeMap::new();
        generic_patches.insert("my-lib".to_string(), path_dep("../generic-lib"));
        patches.insert("forc.pub".to_string(), generic_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that namespace-specific patch takes priority
        let patch = source.dep_patch("my-lib", &manifest_file).unwrap();
        assert!(patch.is_some(), "Should find patch");

        let patch = patch.unwrap();
        match patch {
            Dependency::Detailed(det) => {
                assert_eq!(
                    det.path,
                    Some("../namespaced-lib".to_string()),
                    "Should use namespace-specific patch, not generic patch"
                );
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_registry_patch_fallback_to_generic() {
        // Create a registry source with domain namespace
        let source = Source::Registry(reg::Source {
            name: "common-lib".to_string(),
            version: semver::Version::new(1, 0, 0),
            namespace: reg::file_location::Namespace::Domain("com/myorg".to_string()),
        });

        // Create a manifest with ONLY generic patch (no namespace-specific)
        let mut patches = BTreeMap::new();
        let mut generic_patches = BTreeMap::new();
        generic_patches.insert("common-lib".to_string(), path_dep("../common-lib"));
        patches.insert("forc.pub".to_string(), generic_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that it falls back to generic patch
        let patch = source.dep_patch("common-lib", &manifest_file).unwrap();
        assert!(patch.is_some(), "Should find generic patch as fallback");

        let patch = patch.unwrap();
        match patch {
            Dependency::Detailed(det) => {
                assert_eq!(det.path, Some("../common-lib".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_git_patch_still_works() {
        // Create a git source
        let repo_url = "https://github.com/fuellabs/sway";
        let source = Source::Git(git::Source {
            repo: git::Url::from_str(repo_url).unwrap(),
            reference: git::Reference::Tag("v0.63.0".to_string()),
        });

        // Create a manifest with a git patch
        let mut patches = BTreeMap::new();
        let mut git_patches = BTreeMap::new();
        git_patches.insert(
            "std".to_string(),
            git_dep("https://github.com/fuellabs/sway", "feature-branch"),
        );
        patches.insert(repo_url.to_string(), git_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that git patch still works
        let patch = source.dep_patch("std", &manifest_file).unwrap();
        assert!(
            patch.is_some(),
            "Should find git patch (backward compatibility)"
        );

        let patch = patch.unwrap();
        match patch {
            Dependency::Detailed(det) => {
                assert_eq!(
                    det.git,
                    Some("https://github.com/fuellabs/sway".to_string())
                );
                assert_eq!(det.branch, Some("feature-branch".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_no_patch_found() {
        // Create a registry source
        let source = Source::Registry(reg::Source {
            name: "no-patch-lib".to_string(),
            version: semver::Version::new(1, 0, 0),
            namespace: reg::file_location::Namespace::Flat,
        });

        // Create a manifest with patches for different packages
        let mut patches = BTreeMap::new();
        let mut forc_pub_patches = BTreeMap::new();
        forc_pub_patches.insert("other-lib".to_string(), path_dep("../other-lib"));
        patches.insert("forc.pub".to_string(), forc_pub_patches);

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(patches);

        // Test that no patch is found
        let patch = source.dep_patch("no-patch-lib", &manifest_file).unwrap();
        assert!(
            patch.is_none(),
            "Should not find patch for different package"
        );
    }

    #[test]
    fn test_path_source_no_patch() {
        // Path sources should not have patches
        let source = Source::Path(PathBuf::from("/some/path"));

        let (_temp_dir, manifest_file) = create_test_manifest_file_with_patches(BTreeMap::new());

        // Test that no patch is found for path sources
        let patch = source.dep_patch("anything", &manifest_file).unwrap();
        assert!(patch.is_none(), "Path sources should not support patches");
    }
}
