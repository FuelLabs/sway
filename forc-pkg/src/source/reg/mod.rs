pub mod file_location;
pub mod index_file;

use super::IPFSNode;
use crate::{
    manifest::{self, GenericManifestFile, PackageManifestFile},
    source::{
        self,
        ipfs::{ipfs_client, Cid},
    },
};
use anyhow::{anyhow, bail, Context};
use file_location::{location_from_root, Namespace};
use forc_tracing::println_action_green;
use index_file::IndexFile;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    thread,
    time::Duration,
};

/// Name of the folder containing fetched registry sources.
pub const REG_DIR_NAME: &str = "registry";

/// A package from the official registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Source {
    /// The name of the specified package.
    pub name: String,
    /// The base version specified for the package.
    pub version: semver::Version,
    /// The namespace this package resides in, if no there is no namespace in
    /// registry setup, this will be `None`.
    pub namespace: Namespace,
}

/// A pinned instance of the registry source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned {
    /// The registry package with base version.
    pub source: Source,
    /// The corresponding CID for this registry entry.
    pub cid: Cid,
}

/// A resolver for registry index hosted as a github repo.
///
/// Given a package name and a version, a `GithubRegistryResolver` will be able
/// to resolve, fetch, pin a package through using the index hosted on a github
/// repository.
pub struct GithubRegistryResolver {
    /// Name of the github organization holding the registry index repository.
    repo_org: String,
    /// Name of git repository holding the registry index.
    repo_name: String,
    /// The number of letters used to chunk package name.
    ///
    /// Example:
    /// If set to 2, and package name is "foobar", the index file location
    /// will be ".../fo/ob/ar/foobar".
    chunk_size: usize,
    /// Type of the namespacing is needed to determine whether to add domain at
    /// the beginning of the file location.
    namespace: Namespace,
    /// Branch name of the registry repo, the resolver is going to be using.
    branch_name: String,
}

/// Error returned upon failed parsing of `Pinned::from_str`.
#[derive(Clone, Debug)]
pub enum PinnedParseError {
    Prefix,
    PackageName,
    PackageVersion,
    Cid,
    Namespace,
}

impl GithubRegistryResolver {
    /// Default github organization name that holds the registry git repo.
    pub const DEFAULT_GITHUB_ORG: &str = "FuelLabs";
    /// Default name of the repository that holds the registry git repo.
    pub const DEFAULT_REPO_NAME: &str = "forc.pub-index";
    /// Default chunking size of the repository that holds registry git repo.
    pub const DEFAULT_CHUNKING_SIZE: usize = 2;
    /// Default branch name for the repository repo.
    const DEFAULT_BRANCH_NAME: &str = "master";
    /// Default timeout for each github look-up request. If exceeded request is
    /// dropped.
    const DEFAULT_TIMEOUT_MS: u64 = 10000;

    pub fn new(
        repo_org: String,
        repo_name: String,
        chunk_size: usize,
        namespace: Namespace,
        branch_name: String,
    ) -> Self {
        Self {
            repo_org,
            repo_name,
            chunk_size,
            namespace,
            branch_name,
        }
    }

    /// Returns a `GithubRegistryResolver` that automatically uses
    /// `Self::DEFAULT_GITHUB_ORG` and `Self::DEFAULT_REPO_NAME`.
    pub fn with_default_github(namespace: Namespace) -> Self {
        Self {
            repo_org: Self::DEFAULT_GITHUB_ORG.to_string(),
            repo_name: Self::DEFAULT_REPO_NAME.to_string(),
            chunk_size: Self::DEFAULT_CHUNKING_SIZE,
            namespace,
            branch_name: Self::DEFAULT_BRANCH_NAME.to_string(),
        }
    }

    /// Returns the namespace associated with this `GithubRegistryResolver`.
    ///
    /// See `[GithubRegistryResolver::namespace]` for details.
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Returns the branch name used by this `GithubRegistryResolver`.
    ///
    /// See `[GithubRegistryResolver::branch_name]` for details.
    pub fn branch_name(&self) -> &str {
        &self.branch_name
    }

    /// Returns the chunk size used by this `GithubRegistryResolver`.
    ///
    /// See `[GithubRegistryResolver::chunk_size]` for details.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Returns the owner of the repo this `GithubRegistryResolver` configured
    /// to fetch from.
    ///
    /// See `[GithubRegistryResolver::repo_org]` for details.
    pub fn repo_org(&self) -> &str {
        &self.repo_org
    }

    /// Returns the name of the repo this `GithubRegistryResolver` configured
    /// to fetch from.
    ///
    /// See `[GithubRegistryResolver::repo_name]` for details.
    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }
}

impl Pinned {
    pub const PREFIX: &str = "registry";
}

impl Display for Pinned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // registry+<package_name>?v<version>#<cid>!namespace
        write!(
            f,
            "{}+{}?{}#{}!{}",
            Self::PREFIX,
            self.source.name,
            self.source.version,
            self.cid.0,
            self.source.namespace
        )
    }
}

impl FromStr for Pinned {
    type Err = PinnedParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // registry+<package_name>?v<version>#<cid>!<namespace>
        let s = s.trim();

        // Check for "registry+" at the start.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus).is_some_and(|loc| loc != 0) {
            return Err(PinnedParseError::Prefix);
        }

        let without_prefix = &s[prefix_plus.len()..];

        // Parse the package name.
        let pkg_name = without_prefix
            .split('?')
            .next()
            .ok_or(PinnedParseError::PackageName)?;

        let without_package_name = &without_prefix[pkg_name.len() + "?".len()..];
        let mut s_iter = without_package_name.split('#');

        // Parse the package version
        let pkg_version = s_iter.next().ok_or(PinnedParseError::PackageVersion)?;
        let pkg_version =
            semver::Version::from_str(pkg_version).map_err(|_| PinnedParseError::PackageVersion)?;

        // Parse the CID and namespace.
        let cid_and_namespace = s_iter.next().ok_or(PinnedParseError::Cid)?;
        let mut s_iter = cid_and_namespace.split('!');

        let cid = s_iter.next().ok_or(PinnedParseError::Cid)?;
        if !validate_cid(cid) {
            return Err(PinnedParseError::Cid);
        }
        let cid = Cid::from_str(cid).map_err(|_| PinnedParseError::Cid)?;

        // If there is a namespace string after ! and if it is not empty
        // get a `Namespace::Domain` otherwise return a `Namespace::Flat`.
        let namespace = s_iter
            .next()
            .filter(|ns| !ns.is_empty())
            .map_or_else(|| Namespace::Flat, |ns| Namespace::Domain(ns.to_string()));

        let source = Source {
            name: pkg_name.to_string(),
            version: pkg_version,
            namespace,
        };

        Ok(Self { source, cid })
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+{}", self.name, self.version)
    }
}
fn registry_dir() -> PathBuf {
    forc_util::user_forc_directory().join(REG_DIR_NAME)
}

fn registry_with_namespace_dir(namespace: &Namespace) -> PathBuf {
    let base = registry_dir();
    match namespace {
        Namespace::Flat => base,
        Namespace::Domain(ns) => base.join(ns),
    }
}

fn registry_package_dir(
    namespace: &Namespace,
    pkg_name: &str,
    pkg_version: &semver::Version,
) -> PathBuf {
    registry_with_namespace_dir(namespace).join(format!("{pkg_name}-{pkg_version}"))
}

/// The name to use for a package's identifier entry under the user's forc directory.
fn registry_package_dir_name(name: &str, pkg_version: &semver::Version) -> String {
    use std::hash::{Hash, Hasher};
    fn hash_version(pkg_version: &semver::Version) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        pkg_version.hash(&mut hasher);
        hasher.finish()
    }
    let package_ver_hash = hash_version(pkg_version);
    format!("{name}-{package_ver_hash:x}")
}

/// Validates if the cid string is valid by checking the initial 2 letters and
/// length.
///
/// For CIDs to be marked as valid:
/// 1. Must start with `Qm`.
/// 2. Must be 46 chars long.
///
/// For more details see: https://docs.ipfs.tech/concepts/content-addressing/#version-0-v0
fn validate_cid(cid: &str) -> bool {
    let cid = cid.trim();
    let starts_with_qm = cid.starts_with("Qm");
    starts_with_qm && cid.len() == 46
}

/// A temporary directory that we can use for cloning a registry-sourced package's index file and discovering
/// the corresponding CID for that package.
///
/// The resulting directory is:
///
/// ```ignore
/// $HOME/.forc/registry/cache/tmp/<fetch_id>-name-<version_hash>
/// ```
///
/// A unique `fetch_id` may be specified to avoid contention over the registry directory in the
/// case that multiple processes or threads may be building different projects that may require
/// fetching the same dependency.
fn tmp_registry_package_dir(
    fetch_id: u64,
    name: &str,
    version: &semver::Version,
    namespace: &Namespace,
) -> PathBuf {
    let repo_dir_name = format!(
        "{:x}-{}",
        fetch_id,
        registry_package_dir_name(name, version)
    );
    registry_with_namespace_dir(namespace)
        .join("tmp")
        .join(repo_dir_name)
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, ctx: source::PinCtx) -> anyhow::Result<(Self::Pinned, PathBuf)> {
        let pkg_name = ctx.name.to_string();

        let fetch_id = ctx.fetch_id();
        let source = self.clone();
        let pkg_name = pkg_name.clone();

        let cid = block_on_any_runtime(async move {
            with_tmp_fetch_index(fetch_id, &pkg_name, &source, |index_file| {
                let version = source.version.clone();
                let pkg_name = pkg_name.clone();
                async move {
                    let pkg_entry = index_file
                        .get(&version)
                        .ok_or_else(|| anyhow!("No {} found for {}", version, pkg_name))?;
                    Cid::from_str(pkg_entry.source_cid()).map_err(anyhow::Error::from)
                }
            })
            .await
        })?;

        let path = registry_package_dir(&self.namespace, ctx.name, &self.version);
        let pinned = Pinned {
            source: self.clone(),
            cid,
        };
        Ok((pinned, path))
    }
}

impl source::Fetch for Pinned {
    fn fetch(&self, ctx: source::PinCtx, path: &Path) -> anyhow::Result<PackageManifestFile> {
        // Co-ordinate access to the registry checkout directory using an advisory file lock.
        let mut lock = forc_util::path_lock(path)?;
        // TODO: Here we assume that if the local path already exists, that it contains the
        // full and correct source for that registry entry and hasn't been tampered with. This is
        // probably fine for most cases as users should never be touching these
        // directories, however we should add some code to validate this. E.g. can we
        // recreate the ipfs cid by hashing the directory or something along these lines?
        // https://github.com/FuelLabs/sway/issues/7075
        {
            let _guard = lock.write()?;
            if !path.exists() {
                println_action_green(
                    "Fetching",
                    &format!(
                        "{} {}",
                        ansiterm::Style::new().bold().paint(ctx.name),
                        self.source.version
                    ),
                );
                let pinned = self.clone();
                let fetch_id = ctx.fetch_id();
                let ipfs_node = ctx.ipfs_node().clone();

                block_on_any_runtime(async move { fetch(fetch_id, &pinned, &ipfs_node).await })?;
            }
        }
        let path = {
            let _guard = lock.read()?;
            manifest::find_within(path, ctx.name())
                .ok_or_else(|| anyhow!("failed to find package `{}` in {}", ctx.name(), self))?
        };
        PackageManifestFile::from_file(path)
    }
}

impl source::DepPath for Pinned {
    fn dep_path(&self, _name: &str) -> anyhow::Result<source::DependencyPath> {
        bail!("dep_path: registry dependencies are not yet supported");
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Registry(p)
    }
}

/// Resolve a CID from index file and pinned package. Basically goes through
/// the index file to find corresponding entry described by the pinned instance.
fn resolve_to_cid(index_file: &IndexFile, pinned: &Pinned) -> anyhow::Result<Cid> {
    let other_versions = index_file
        .versions()
        .filter(|ver| **ver != pinned.source.version)
        .map(|ver| format!("{}.{}.{}", ver.major, ver.minor, ver.patch))
        .collect::<Vec<_>>()
        .join(",");

    let package_entry = index_file.get(&pinned.source.version).ok_or_else(|| {
        anyhow!(
            "Version {} not found for {}. Other available versions: [{}]",
            pinned.source.version,
            pinned.source.name,
            other_versions
        )
    })?;

    let cid = Cid::from_str(package_entry.source_cid()).with_context(|| {
        format!(
            "Invalid CID {}v{}: `{}`",
            package_entry.name(),
            package_entry.version(),
            package_entry.source_cid()
        )
    })?;
    if package_entry.yanked() {
        bail!(
            "Version {} of {} is yanked. Other avaiable versions: [{}]",
            pinned.source.version,
            pinned.source.name,
            other_versions
        );
    }
    Ok(cid)
}

async fn fetch(fetch_id: u64, pinned: &Pinned, ipfs_node: &IPFSNode) -> anyhow::Result<PathBuf> {
    let path = with_tmp_fetch_index(
        fetch_id,
        &pinned.source.name,
        &pinned.source,
        |index_file| async move {
            let path = registry_package_dir(
                &pinned.source.namespace,
                &pinned.source.name,
                &pinned.source.version,
            );
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
            }
            fs::create_dir_all(&path)?;

            let cid = resolve_to_cid(&index_file, pinned)?;

            match ipfs_node {
                IPFSNode::Local => {
                    println_action_green("Fetching", "with local IPFS node");
                    cid.fetch_with_client(&ipfs_client(), &path).await?;
                }
                IPFSNode::WithUrl(gateway_url) => {
                    println_action_green(
                        "Fetching",
                        &format!("from {}. Note: This can take several minutes.", gateway_url),
                    );
                    cid.fetch_with_gateway_url(gateway_url, &path).await?;
                }
            }

            Ok(path)
        },
    )
    .await?;
    Ok(path)
}

async fn with_tmp_fetch_index<F, O, Fut>(
    fetch_id: u64,
    pkg_name: &str,
    source: &Source,
    f: F,
) -> anyhow::Result<O>
where
    F: FnOnce(IndexFile) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<O>>,
{
    let tmp_dir = tmp_registry_package_dir(fetch_id, pkg_name, &source.version, &source.namespace);
    if tmp_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    // Add a guard to ensure cleanup happens if we got out of scope whether by
    // returning or panicking.
    let _cleanup_guard = scopeguard::guard(&tmp_dir, |dir| {
        let _ = std::fs::remove_dir_all(dir);
    });

    let github_resolver = GithubRegistryResolver::with_default_github(source.namespace.clone());

    let path = location_from_root(github_resolver.chunk_size, &source.namespace, pkg_name)
        .display()
        .to_string();
    let index_repo_owner = github_resolver.repo_org();
    let index_repo_name = github_resolver.repo_name();
    let reference = format!("refs/heads/{}", github_resolver.branch_name());
    let github_endpoint = format!(
        "https://raw.githubusercontent.com/{index_repo_owner}/{index_repo_name}/{reference}/{path}"
    );
    let client = reqwest::Client::new();
    let timeout_duration = Duration::from_millis(GithubRegistryResolver::DEFAULT_TIMEOUT_MS);
    let index_response = client
        .get(github_endpoint)
        .timeout(timeout_duration)
        .send()
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to send request to github to obtain package index file from registry {e}"
            )
        })?
        .error_for_status()
        .map_err(|_| anyhow!("Failed to fetch {pkg_name}"))?;

    let contents = index_response.text().await?;
    let index_file: IndexFile = serde_json::from_str(&contents).with_context(|| {
        format!(
            "Unable to deserialize a github registry lookup response. Body was: \"{}\"",
            contents
        )
    })?;

    let res = f(index_file).await?;
    Ok(res)
}

/// Execute an async block on a Tokio runtime.
///
/// If we are already in a runtime, this will spawn a new OS thread to create a new runtime.
///
/// If we are not in a runtime, a new runtime is created and the future is blocked on.
pub(crate) fn block_on_any_runtime<F>(future: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        // In a runtime context. Spawn a new thread to run the async code.
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(future)
        })
        .join()
        .unwrap()
    } else {
        // Not in a runtime context. Okay to create a new runtime and block.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(future)
    }
}

#[cfg(test)]
mod tests {
    use super::{file_location::Namespace, resolve_to_cid, Pinned, Source};
    use crate::source::{
        ipfs::Cid,
        reg::index_file::{IndexFile, PackageEntry},
    };
    use std::str::FromStr;

    #[test]
    fn parse_pinned_entry_without_namespace() {
        let pinned_str = "registry+core?0.0.1#QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKS!";
        let pinned = Pinned::from_str(pinned_str).unwrap();

        let expected_source = Source {
            name: "core".to_string(),
            version: semver::Version::new(0, 0, 1),
            namespace: Namespace::Flat,
        };

        let cid = Cid::from_str("QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKS").unwrap();

        let expected_pinned = Pinned {
            source: expected_source,
            cid,
        };

        assert_eq!(pinned, expected_pinned)
    }

    #[test]
    fn parse_pinned_entry_with_namespace() {
        let pinned_str =
            "registry+core?0.0.1#QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKS!fuelnamespace";
        let pinned = Pinned::from_str(pinned_str).unwrap();

        let expected_source = Source {
            name: "core".to_string(),
            version: semver::Version::new(0, 0, 1),
            namespace: Namespace::Domain("fuelnamespace".to_string()),
        };

        let cid = Cid::from_str("QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKS").unwrap();

        let expected_pinned = Pinned {
            source: expected_source,
            cid,
        };

        assert_eq!(pinned, expected_pinned)
    }

    #[test]
    fn test_resolve_to_cid() {
        let mut index_file = IndexFile::default();

        // Add a regular version with a valid CID
        let valid_cid = "QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKS";
        let valid_version = semver::Version::new(1, 0, 0);
        let valid_entry = PackageEntry::new(
            "test_package".to_string(),
            valid_version.clone(),
            valid_cid.to_string(),
            None,   // no abi_cid
            vec![], // no dependencies
            false,  // not yanked
        );
        index_file.insert(valid_entry);

        // Add a yanked version
        let yanked_cid = "QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKR";
        let yanked_version = semver::Version::new(0, 9, 0);
        let yanked_entry = PackageEntry::new(
            "test_package".to_string(),
            yanked_version.clone(),
            yanked_cid.to_string(),
            None,   // no abi_cid
            vec![], // no dependencies
            true,   // yanked
        );
        index_file.insert(yanked_entry);

        // Add another version just to have multiple available
        let other_cid = "QmdMVqLqpba2mMB5AUjYCxubC6tLGevQFunpBkbC2UbrKT";
        let other_version = semver::Version::new(1, 1, 0);
        let other_entry = PackageEntry::new(
            "test_package".to_string(),
            other_version.clone(),
            other_cid.to_string(),
            None,   // no abi_cid
            vec![], // no dependencies
            false,  // not yanked
        );
        index_file.insert(other_entry);

        // Test Case 1: Successful resolution
        let valid_source = Source {
            name: "test_package".to_string(),
            version: valid_version.clone(),
            namespace: Namespace::Flat,
        };
        let valid_pinned = Pinned {
            source: valid_source,
            cid: Cid::from_str(valid_cid).unwrap(),
        };

        let result = resolve_to_cid(&index_file, &valid_pinned);
        assert!(result.is_ok());
        let valid_cid = Cid::from_str(valid_cid).unwrap();
        assert_eq!(result.unwrap(), valid_cid);

        // Test Case 2: Error when version doesn't exist
        let nonexistent_version = semver::Version::new(2, 0, 0);
        let nonexistent_source = Source {
            name: "test_package".to_string(),
            version: nonexistent_version,
            namespace: Namespace::Flat,
        };
        let nonexistent_pinned = Pinned {
            source: nonexistent_source,
            // this cid just a placeholder, as this version does not exists
            cid: valid_cid,
        };

        let result = resolve_to_cid(&index_file, &nonexistent_pinned);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Version 2.0.0 not found"));
        assert!(
            error_msg.contains("Other available versions: [1.1.0,0.9.0,1.0.0]")
                || error_msg.contains("Other available versions: [0.9.0,1.0.0,1.1.0]")
                || error_msg.contains("Other available versions: [1.0.0,0.9.0,1.1.0]")
                || error_msg.contains("Other available versions: [0.9.0,1.1.0,1.0.0]")
                || error_msg.contains("Other available versions: [1.0.0,1.1.0,0.9.0]")
                || error_msg.contains("Other available versions: [1.1.0,1.0.0,0.9.0]")
        );

        // Test Case 3: Error when version is yanked
        let yanked_source = Source {
            name: "test_package".to_string(),
            version: yanked_version.clone(),
            namespace: Namespace::Flat,
        };
        let yanked_pinned = Pinned {
            source: yanked_source,
            cid: Cid::from_str(yanked_cid).unwrap(),
        };

        let result = resolve_to_cid(&index_file, &yanked_pinned);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Version 0.9.0 of test_package is yanked"));
        assert!(
            error_msg.contains("Other avaiable versions: [1.1.0,1.0.0]")
                || error_msg.contains("Other avaiable versions: [1.0.0,1.1.0]")
        );
    }
}
