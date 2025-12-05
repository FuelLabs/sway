use crate::cli::UpdateCommand;
use anyhow::{anyhow, Result};
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::{self as pkg, lock, Lock};
use pkg::manifest::ManifestFile;
use std::{fs, path::PathBuf};
use sway_utils::constants;
use tracing::info;

/// Running `forc update` will check for updates for the entire dependency graph and commit new
/// semver-compatible versions to the `Forc.lock` file. For git dependencies, the commit is updated
/// to the HEAD of the specified branch, or remains unchanged in the case a tag is specified. Path
/// dependencies remain unchanged as they are always sourced directly.
///
/// This is called during `forc build` in the case that there is no existing `Forc.lock` file for
/// the project.
///
/// Run `forc update --check` to perform a dry-run and produce a list of updates that will be
/// performed across all dependencies without actually committing them to the lock file.
///
/// Use the `--package <package-name>` flag to update only a specific package throughout the
/// dependency graph.
pub fn update(command: UpdateCommand) -> Result<()> {
    let UpdateCommand {
        path,
        check,
        // TODO: Use `package` here rather than `target_dependency`
        target_dependency: _,
        ..
    } = command;

    let this_dir = match path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };

    let manifest = ManifestFile::from_dir(this_dir)?;
    let lock_path = manifest.dir().join(constants::LOCK_FILE_NAME);
    let old_lock = Lock::from_path(&lock_path).ok().unwrap_or_default();
    let offline = false;
    let member_manifests = manifest.member_manifests()?;
    let ipfs_node = command.ipfs_node.unwrap_or_default();
    let new_plan = pkg::BuildPlan::from_manifests(&member_manifests, offline, &ipfs_node)?;
    let new_lock = Lock::from_graph(new_plan.graph());
    let diff = new_lock.diff(&old_lock);
    let member_names = member_manifests
        .values()
        .map(|manifest| manifest.project.name.clone())
        .collect();
    lock::print_diff(&member_names, &diff);

    // If we're not only `check`ing, write the updated lock file.
    if !check {
        let string = toml::ser::to_string_pretty(&new_lock)
            .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, string).map_err(|e| anyhow!("failed to write lock file: {}", e))?;
        info!("   Created new lock file at {}", lock_path.display());
    } else {
        info!(" `--check` enabled: `Forc.lock` was not changed");
    }

    Ok(())
}
