use crate::{
    cli::UpdateCommand,
    utils::{
        helpers::{find_manifest_dir, read_manifest},
        manifest::Manifest,
    },
};

// TODO: refactor dependency stuff out of forc_build
use crate::ops::forc_build;
use crate::utils::manifest::{Dependency, DependencyDetails};
use anyhow::{anyhow, bail, Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, str};

pub type GitHubAPICommitsResponse = Vec<GithubCommit>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCommit {
    pub sha: String,
}
#[derive(Debug)]
pub struct VersionedDependency {
    pub hash: String,
    pub path: String,
}

/// Forc update will update the contents inside the Forc dependencies directory.
/// If a dependency `d` is passed as parameter, it will only try and update that specific dependency.
/// Otherwise, it will try and update all GitHub-based dependencies in a project's `Forc.toml`.
/// It won't automatically update dependencies that have a version specified, if you have
/// specified a version for a dependency and want to update it you should, instead,
/// run `forc check-updates` to check for updates for all GitHub-based dependencies, and if
/// a new version is detected and return, manually update your `Forc.toml` with this new version.
pub fn update(command: UpdateCommand) -> Result<()> {
    let UpdateCommand {
        path,
        target_dependency,
    } = command;
    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(anyhow!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };

    let mut manifest = read_manifest(&manifest_dir).unwrap();

    let dependencies = get_detailed_dependencies(&mut manifest);

    match target_dependency {
        // Target dependency (`-d`) specified
        Some(target_dep) => match dependencies.get(&target_dep) {
            Some(dep) => Ok(update_dependency(&target_dep, dep).unwrap()),
            None => return Err(anyhow!("dependency {} not found", target_dep)),
        },
        // No target dependency specified, try and update all dependencies
        None => {
            for (dependency_name, dep) in dependencies {
                update_dependency(&dependency_name, dep)?;
            }
            Ok(())
        }
    }
}

fn update_dependency(dependency_name: &str, dep: &DependencyDetails) -> Result<()> {
    let home_dir = match home_dir() {
        None => return Err(anyhow!("Couldn't find home directory (`~/`)")),
        Some(p) => p.to_str().unwrap().to_owned(),
    };

    // Currently we only handle updates on github-based dependencies
    if let Some(git) = &dep.git {
        // Automatically updating a dependency that has a tag/version specified in `Forc.toml`
        // would mean to update the `Forc.toml` file, which I believe isn't a very
        // nice behavior. Instead, if a tag/version is specified, the user should
        // lookup for a desired version and manually specify it in `Forc.toml`.
        if let Some(version) = &dep.version {
            println!("Ignoring update for {} at version {}: Forc update not implemented for dependencies with specified tag. To update to another tag, change the tag in `Forc.toml` and run the build command.", dependency_name, version);
        }

        let target_directory = match &dep.branch {
            Some(b) => format!("{}/.forc/{}/{}", home_dir, dependency_name, &b),
            None => format!("{}/.forc/{}/default", home_dir, dependency_name),
        };

        let current = get_current_dependency_version(&target_directory)?;

        let latest_hash = get_latest_commit_sha(git, &dep.branch)?;

        if current.hash == latest_hash {
            println!("{} is up-to-date", dependency_name);
        } else {
            replace_dep_version(&target_directory, git, dep).unwrap();
            println!("{}: {} -> {}", dependency_name, current.hash, latest_hash);
        }
    }
    Ok(())
}

fn replace_dep_version(target_directory: &str, git: &str, dep: &DependencyDetails) -> Result<()> {
    let current = get_current_dependency_version(&target_directory).unwrap();

    let api_url = forc_build::build_github_api_url(git, &dep.branch, &dep.version);
    forc_build::download_tarball(&api_url, &target_directory).unwrap();
    // Delete old one
    match fs::remove_dir_all(current.path) {
        Ok(_) => Ok(()), // TODO: Test this
        Err(e) => return Err(anyhow!("failed to update dep {}: {}", git, e)),
    }
}

fn get_current_dependency_version(dep_dir: &str) -> Result<VersionedDependency> {
    for entry in fs::read_dir(dep_dir).context(format!("couldn't read directory {}", dep_dir))? {
        let path = entry?.path();
        if !path.is_dir() {
            bail!("{} isn't a directory.", dep_dir)
        }

        let path_str = path.to_str().unwrap().to_string();

        // Getting the base of the path (the dependency directory name)
        let mut pieces = path_str.rsplit("/");
        match pieces.next() {
            Some(p) => {
                return Ok(VersionedDependency {
                    // Dependencies directories are named as "$repo_owner-$repo-$concatenated_hash"
                    // Here we're grabbing the hash.
                    hash: p.to_owned().split("-").last().unwrap().into(),
                    path: path_str,
                });
            }
            None => bail!("Unexpected dependency naming scheme: {}", path_str),
        }
    }
    bail!("Dependency directory is empty. Run `forc build` to install dependencies.")
}

// Returns the _truncated_ (e.g `e6940e4`) latest commit hash of a
// GitHub repository given a branch. If branch is None, the default branch is used.
fn get_latest_commit_sha(dependency_url: &str, branch: &Option<String>) -> Result<String> {
    // Quick protection against `git` dependency URL ending with `/`.
    let dependency_url = dependency_url.trim_end_matches("/");

    let mut pieces = dependency_url.rsplit("/");

    let project_name: &str = match pieces.next() {
        Some(p) => p.into(),
        None => dependency_url.into(),
    };

    let owner_name: &str = match pieces.next() {
        Some(p) => p.into(),
        None => dependency_url.into(),
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

    let client = reqwest::blocking::Client::builder()
        .user_agent("forc-builder")
        .build()?;

    let resp = client.get(&api_endpoint).send()?;

    let hash_vec = resp.json::<GitHubAPICommitsResponse>().context(format!(
        "couldn't parse GitHub API response. API endpoint crafted: {}",
        api_endpoint
    ))?;

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
fn get_detailed_dependencies(manifest: &mut Manifest) -> HashMap<String, &DependencyDetails> {
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
