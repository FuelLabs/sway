use crate::{
    cli::UpdateCommand,
    lock::Lock,
    pkg,
    utils::helpers::{lock_path, print_lock_diff, read_manifest},
};
use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};
use sway_utils::find_manifest_dir;

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
pub async fn update(command: UpdateCommand) -> Result<()> {
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
    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(anyhow!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };

    let manifest = read_manifest(&manifest_dir).map_err(|e| anyhow!("{}", e))?;
    let lock_path = lock_path(&manifest_dir);
    let old_lock = Lock::from_path(&lock_path).ok().unwrap_or_default();
    let offline = false;
    let new_plan = pkg::BuildPlan::new(&manifest_dir, offline).map_err(|e| anyhow!("{}", e))?;
    let new_lock = Lock::from_graph(&new_plan.graph);
    let diff = new_lock.diff(&old_lock);
    print_lock_diff(&manifest.project.name, &diff);

    // If we're not only `check`ing, write the updated lock file.
    if !check {
        let string = toml::ser::to_string_pretty(&new_lock)
            .map_err(|e| anyhow!("failed to serialize lock file: {}", e))?;
        fs::write(&lock_path, &string).map_err(|e| anyhow!("failed to write lock file: {}", e))?;
        println!("   Created new lock file at {}", lock_path.display());
    } else {
        println!(" `--check` enabled: `Forc.lock` was not changed");
    }

    Ok(())
}
