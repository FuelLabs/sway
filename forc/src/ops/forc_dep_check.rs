use crate::utils::{
    dependency,
    helpers::{read_manifest, user_forc_directory},
};
use anyhow::{bail, Result};
use semver::Version;
use std::{
    path::{Path, PathBuf},
    str,
};
use sway_utils::find_manifest_dir;

/// Forc check will check if there are updates to Github-based dependencies.
/// If a target dependency `-d` is passed, it will check only this one dependency.
/// Otherwise, it will check for all dependencies in the manifest.
/// Note that this won't automatically update the dependencies, it will only
/// point out newer versions of the dependencies.
/// If a dependency was specified in the manifest _without_ a tag/version,
/// `forc update` can automatically update to the latest version.
/// If a dependency has a tag, `forc dep_check` will let you know if there's a newer tag
/// and then you can decide whether to update it in the manifest or not.
pub async fn check(path: Option<String>, target_dependency: Option<String>) -> Result<()> {
    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            bail!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            )
        }
    };

    let mut manifest = read_manifest(&manifest_dir).unwrap();

    let dependencies = dependency::get_detailed_dependencies(&mut manifest);

    match target_dependency {
        // Target dependency (`-d`) specified
        Some(target_dep) => match dependencies.get(&target_dep) {
            Some(dep) => Ok(check_dependency(&target_dep, dep).await?),
            None => bail!("dependency {} not found", target_dep),
        },
        // No target dependency specified, try and update all dependencies
        None => {
            for (dependency_name, dep) in dependencies {
                check_dependency(&dependency_name, dep).await?;
            }
            Ok(())
        }
    }
}

async fn check_dependency(
    dependency_name: &str,
    dep: &dependency::DependencyDetails,
) -> Result<()> {
    let user_forc_dir = user_forc_directory();
    let dep_dir = user_forc_dir.join(dependency_name);
    let target_directory = match &dep.branch {
        Some(branch) => dep_dir.join(branch),
        None => dep_dir.join("default"),
    };

    // Currently we only handle checks on github-based dependencies
    if let Some(git) = &dep.git {
        match &dep.version {
            Some(version) => check_tagged_dependency(dependency_name, version, git).await?,
            None => check_untagged_dependency(git, &target_directory, dependency_name, dep).await?,
        }
    }
    Ok(())
}

async fn check_tagged_dependency(
    dependency_name: &str,
    current_version: &str,
    git_repo: &str,
) -> Result<()> {
    let releases = dependency::get_github_repo_releases(git_repo).await?;

    let current_release = Version::parse(current_version)?;

    let mut latest = current_release.clone();

    for release in &releases {
        let release_version = Version::parse(release)?;

        if release_version.gt(&current_release) {
            latest = release_version;
        }
    }

    if current_release.ne(&latest) {
        println!(
            "[{}] not up-to-date. Current version: {}, latest: {}",
            dependency_name, current_release, latest
        );
    } else {
        println!(
            "[{}] up-to-date. Current version: {}",
            dependency_name, current_release,
        );
    }

    Ok(())
}

async fn check_untagged_dependency(
    git_repo: &str,
    target_directory: &Path,
    dependency_name: &str,
    dep: &dependency::DependencyDetails,
) -> Result<()> {
    let current = dependency::get_current_dependency_version(target_directory)?;

    let latest_hash = dependency::get_latest_commit_sha(git_repo, &dep.branch).await?;

    if current.hash == latest_hash {
        println!("{} is up-to-date", dependency_name);
    } else {
        println!(
            "[{}] not up-to-date. Current version: {}, latest: {}",
            dependency_name, current.hash, latest_hash
        );
    }
    Ok(())
}
