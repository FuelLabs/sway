use crate::utils::manifest::Manifest;
use anyhow::{anyhow, bail, Context, Result};
use curl::easy::Easy;
use dirs::home_dir;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use sway_utils::constants;
use tar::Archive;

// A collection of remote dependency related functions

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
}
pub enum OfflineMode {
    Yes,
    No,
}

impl From<bool> for OfflineMode {
    fn from(v: bool) -> OfflineMode {
        match v {
            true => OfflineMode::Yes,
            false => OfflineMode::No,
        }
    }
}

pub type GitHubAPICommitsResponse = Vec<GithubCommit>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCommit {
    pub sha: String,
}
/// VersionedDependencyDirectory holds the path to the directory where a given
/// GitHub-based dependency is installed and its respective git hash.
#[derive(Debug)]
pub struct VersionedDependencyDirectory {
    pub hash: String,
    pub path: PathBuf,
}

pub type GitHubRepoReleases = Vec<TaggedRelease>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaggedRelease {
    #[serde(rename = "tag_name")]
    pub tag_name: String,
    #[serde(rename = "target_commitish")]
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "published_at")]
    pub published_at: String,
}

/// Downloads a non-local dependency that's hosted on GitHub.
/// By default, it stores the dependency in `~/.forc/`.
/// A given dependency `dep` is stored under `~/.forc/dep/default/$owner-$repo-$hash`.
/// If no hash (nor any other type of reference) is provided, Forc
/// will download the default branch at the latest commit.
/// If a branch is specified, it will go in `~/.forc/dep/$branch/$owner-$repo-$hash.
/// If a version is specified, it will go in `~/.forc/dep/$version/$owner-$repo-$hash.
/// Version takes precedence over branch reference.
pub fn download_github_dep(
    dep_name: &str,
    repo_base_url: &str,
    branch: &Option<String>,
    version: &Option<String>,
    offline_mode: OfflineMode,
) -> Result<String> {
    let home_dir = match home_dir() {
        None => return Err(anyhow!("Couldn't find home directory (`~/`)")),
        Some(p) => p.to_str().unwrap().to_owned(),
    };

    // Version tag takes precedence over branch reference.
    let out_dir = match &version {
        Some(v) => PathBuf::from(format!(
            "{}/{}/{}/{}",
            home_dir,
            constants::FORC_DEPENDENCIES_DIRECTORY,
            dep_name,
            v
        )),
        // If no version specified, check if a branch was specified
        None => match &branch {
            Some(b) => PathBuf::from(format!(
                "{}/{}/{}/{}",
                home_dir,
                constants::FORC_DEPENDENCIES_DIRECTORY,
                dep_name,
                b
            )),
            // If no version and no branch, use default
            None => PathBuf::from(format!(
                "{}/{}/{}/default",
                home_dir,
                constants::FORC_DEPENDENCIES_DIRECTORY,
                dep_name
            )),
        },
    };

    // Check if dependency is already installed, if so, return its path.
    if out_dir.exists() {
        for entry in fs::read_dir(&out_dir)? {
            let path = entry?.path();
            // If the path to that dependency at that branch/version already
            // exists and there's a directory inside of it,
            // this directory should be the installation path.

            if path.is_dir() {
                return Ok(path.to_str().unwrap().to_string());
            }
        }
    }

    // If offline mode is enabled, don't proceed as it will
    // make use of the network to download the dependency from
    // GitHub.
    // If it's offline mode and the dependency already exists
    // locally, then it would've been returned in the block above.
    if let OfflineMode::Yes = offline_mode {
        return Err(anyhow!(
            "Can't build dependency: dependency {} doesn't exist locally and offline mode is enabled",
            dep_name
        ));
    }

    let github_api_url = build_github_repo_api_url(repo_base_url, branch, version);

    println!("Downloading {:?} into {:?}", dep_name, out_dir);

    match download_tarball(&github_api_url, &out_dir) {
        Ok(downloaded_dir) => Ok(downloaded_dir),
        Err(e) => Err(anyhow!("couldn't download from {}: {}", &github_api_url, e)),
    }
}

/// Builds a proper URL that's used to call GitHub's API.
/// The dependency is specified as `https://github.com/:owner/:project`
/// And the API URL must be like `https://api.github.com/repos/:owner/:project/tarball`
/// Adding a `:ref` at the end makes it download a branch/tag based repo.
/// Omitting it makes it download the default branch at latest commit.
pub fn build_github_repo_api_url(
    dependency_url: &str,
    branch: &Option<String>,
    version: &Option<String>,
) -> String {
    let dependency_url = dependency_url.trim_end_matches('/');
    let mut pieces = dependency_url.rsplit('/');

    let project_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    let owner_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    // Version tag takes precedence over branch reference.
    match version {
        Some(v) => {
            format!(
                "https://api.github.com/repos/{}/{}/tarball/{}",
                owner_name, project_name, v
            )
        }
        // If no version specified, check if a branch was specified
        None => match branch {
            Some(b) => {
                format!(
                    "https://api.github.com/repos/{}/{}/tarball/{}",
                    owner_name, project_name, b
                )
            }
            // If no version and no branch, download default branch at latest commit
            None => {
                format!(
                    "https://api.github.com/repos/{}/{}/tarball",
                    owner_name, project_name
                )
            }
        },
    }
}

pub fn download_tarball(url: &str, out_dir: &Path) -> Result<String> {
    let mut data = Vec::new();
    let mut handle = Easy::new();

    // Download the tarball.
    handle.url(url).context("failed to configure tarball URL")?;
    handle
        .follow_location(true)
        .context("failed to configure follow location")?;

    handle
        .useragent("forc-builder")
        .context("failed to configure User-Agent")?;
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .context("failed to write download data")?;
        transfer.perform().context("failed to download tarball")?;
    }

    // Unpack the tarball.
    Archive::new(GzDecoder::new(Cursor::new(data)))
        .unpack(out_dir)
        .with_context(|| {
            format!(
                "failed to unpack tarball in directory: {}",
                out_dir.display()
            )
        })?;

    for entry in fs::read_dir(out_dir)? {
        let path = entry?.path();
        match path.is_dir() {
            true => return Ok(path.to_str().unwrap().to_string()),
            false => (),
        }
    }

    Err(anyhow!(
        "couldn't find downloaded dependency in directory: {}",
        out_dir.display(),
    ))
}

pub fn replace_dep_version(
    target_directory: &Path,
    git: &str,
    dep: &DependencyDetails,
) -> Result<()> {
    let current = get_current_dependency_version(target_directory)?;

    let api_url = build_github_repo_api_url(git, &dep.branch, &dep.version);
    download_tarball(&api_url, target_directory)?;

    // Delete old one
    match fs::remove_dir_all(current.path) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(anyhow!(
                "failed to remove old version of the dependency ({}): {}",
                git,
                e
            ))
        }
    }
}

pub fn get_current_dependency_version(dep_dir: &Path) -> Result<VersionedDependencyDirectory> {
    let mut entries =
        fs::read_dir(dep_dir).context(format!("couldn't read directory {}", dep_dir.display()))?;
    let entry = match entries.next() {
        Some(entry) => entry,
        None => bail!("Dependency directory is empty. Run `forc build` to install dependencies."),
    };

    let path = entry?.path();
    if !path.is_dir() {
        bail!("{} isn't a directory.", dep_dir.display())
    }

    let file_name = path.file_name().unwrap();
    // Dependencies directories are named as "$repo_owner-$repo-$concatenated_hash"
    let hash = file_name
        .to_str()
        .with_context(|| format!("Invalid utf8 in dependency name: {}", path.display()))?
        .split('-')
        .last()
        .with_context(|| format!("Unexpected dependency naming scheme: {}", path.display()))?
        .into();
    Ok(VersionedDependencyDirectory { hash, path })
}

// Returns the _truncated_ (e.g `e6940e4`) latest commit hash of a
// GitHub repository given a branch. If branch is None, the default branch is used.
pub async fn get_latest_commit_sha(
    dependency_url: &str,
    branch: &Option<String>,
) -> Result<String> {
    // Quick protection against `git` dependency URL ending with `/`.
    let dependency_url = dependency_url.trim_end_matches('/');

    let mut pieces = dependency_url.rsplit('/');

    let project_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    let owner_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    let api_endpoint = match branch {
        Some(b) => {
            format!(
                "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=1",
                owner_name, project_name, b
            )
        }
        None => {
            format!(
                "https://api.github.com/repos/{}/{}/commits?per_page=1",
                owner_name, project_name
            )
        }
    };

    let client = reqwest::Client::builder()
        .user_agent("forc-builder")
        .build()?;

    let resp = client.get(&api_endpoint).send().await?;

    let hash_vec = resp.json::<GitHubAPICommitsResponse>().await?;

    // `take(7)` because the truncated SHA1 used by GitHub is 7 chars long.
    let truncated_hash: String = hash_vec[0].sha.chars().take(7).collect();

    if truncated_hash.is_empty() {
        bail!(
            "failed to extract hash from GitHub commit history API, response: {:?}",
            hash_vec
        )
    }

    Ok(truncated_hash)
}

// Helper to get only detailed dependencies (`Dependency::Detailed`).
pub fn get_detailed_dependencies(manifest: &mut Manifest) -> HashMap<String, &DependencyDetails> {
    let mut dependencies: HashMap<String, &DependencyDetails> = HashMap::new();

    if let Some(ref mut deps) = manifest.dependencies {
        for (dep_name, dependency_details) in deps.iter_mut() {
            match dependency_details {
                Dependency::Simple(..) => continue,
                Dependency::Detailed(dep_details) => {
                    dependencies.insert(dep_name.to_owned(), dep_details)
                }
            };
        }
    }

    dependencies
}

pub async fn get_github_repo_releases(dependency_url: &str) -> Result<Vec<String>> {
    // Quick protection against `git` dependency URL ending with `/`.
    let dependency_url = dependency_url.trim_end_matches('/');

    let mut pieces = dependency_url.rsplit('/');

    let project_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    let owner_name: &str = match pieces.next() {
        Some(p) => p,
        None => dependency_url,
    };

    let api_endpoint = format!(
        "https://api.github.com/repos/{}/{}/releases",
        owner_name, project_name
    );

    let client = reqwest::Client::builder()
        .user_agent("forc-builder")
        .build()?;

    let resp = client.get(&api_endpoint).send().await?;

    let releases_vec = resp.json::<GitHubRepoReleases>().await?;

    let semver_releases: Vec<String> = releases_vec.iter().map(|r| r.tag_name.to_owned()).collect();

    Ok(semver_releases)
}
