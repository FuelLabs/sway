use crate::{
    cli::UpdateCommand,
    utils::{
        dependency,
        helpers::{find_manifest_dir, read_manifest},
    },
};

use anyhow::{anyhow, Result};
use dirs::home_dir;
use std::{path::PathBuf, str};

/// Forc update will update the contents inside the Forc dependencies directory.
/// If a dependency `d` is passed as parameter, it will only try and update that specific dependency.
/// Otherwise, it will try and update all GitHub-based dependencies in a project's `Forc.toml`.
/// It won't automatically update dependencies that have a version specified, if you have
/// specified a version for a dependency and want to update it you should, instead,
/// run `forc check-updates` to check for updates for all GitHub-based dependencies, and if
/// a new version is detected and return, manually update your `Forc.toml` with this new version.
pub async fn update(command: UpdateCommand) -> Result<()> {
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

    let dependencies = dependency::get_detailed_dependencies(&mut manifest);

    match target_dependency {
        // Target dependency (`-d`) specified
        Some(target_dep) => match dependencies.get(&target_dep) {
            Some(dep) => Ok(update_dependency(&target_dep, dep).await?),
            None => return Err(anyhow!("dependency {} not found", target_dep)),
        },
        // No target dependency specified, try and update all dependencies
        None => {
            for (dependency_name, dep) in dependencies {
                update_dependency(&dependency_name, dep).await?;
            }
            Ok(())
        }
    }
}

async fn update_dependency(
    dependency_name: &str,
    dep: &dependency::DependencyDetails,
) -> Result<()> {
    let home_dir = match home_dir() {
        None => return Err(anyhow!("Couldn't find home directory (`~/`)")),
        Some(p) => p.to_str().unwrap().to_owned(),
    };

    // Currently we only handle updates on github-based dependencies
    if let Some(git) = &dep.git {
        match &dep.version {
            // Automatically updating a dependency that has a tag/version specified in `Forc.toml`
            // would mean to update the `Forc.toml` file, which I believe isn't a very
            // nice behavior. Instead, if a tag/version is specified, the user should
            // lookup for a desired version and manually specify it in `Forc.toml`.
            Some(version) => println!("Ignoring update for {} at version {}: Forc update not implemented for dependencies with specified tag. To update to another tag, change the tag in `Forc.toml` and run the build command.", dependency_name, version),
            None => {
                let target_directory = match &dep.branch {
                    Some(b) => format!("{}/.forc/{}/{}", home_dir, dependency_name, &b),
                    None => format!("{}/.forc/{}/default", home_dir, dependency_name),
                };

                let current = dependency::get_current_dependency_version(&target_directory)?;

                let latest_hash = dependency::get_latest_commit_sha(git, &dep.branch).await?;

                if current.hash == latest_hash {
                      println!("{} is up-to-date", dependency_name);
                } else {
                    dependency::replace_dep_version(&target_directory, git, dep)?;
                    println!("{}: {} -> {}", dependency_name, current.hash, latest_hash);
                }
            }
        }
    }
    Ok(())
}
