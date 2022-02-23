use crate::{cli::UpdateCommand, utils::helpers::read_manifest};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use sway_utils::find_manifest_dir;

/// Running `forc update` will check for updates for the entire dependency graph and commit new
/// semver-compatible versions to the `Forc.lock` file. For git dependencies, the commit is updated
/// to the HEAD of the specified branch, or remains unchanged in the case a tag is specified. Path
/// dependencies remain unchanged as they are always sourced directly.
///
/// Run `forc update --check` to perform a dry-run and produce a list of updates that will be
/// performed across all dependencies without actually committing them to the lock file.
///
/// Use the `--package <package-name>` flag to update only a specific package throughout the
/// dependency graph.
pub async fn update(command: UpdateCommand) -> Result<()> {
    if command.check {
        // TODO
        unimplemented!(
            "When set, output whether target dep may be updated but don't commit to lock file"
        );
    }

    let UpdateCommand {
        path,
        check: _,
        // TODO: Use `package` here rather than `target_dependency`
        ..
    } = command;

    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
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

    let _manifest = read_manifest(&manifest_dir).unwrap();

    // TODO
    unimplemented!(
        "Check the graph for git and registry changes and update the `Forc.lock` file accordingly"
    )
}
