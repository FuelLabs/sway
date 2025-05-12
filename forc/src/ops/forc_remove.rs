use crate::cli::RemoveCommand;
use anyhow::Result;
use forc_pkg::{self as pkg};

pub fn remove(cmd: RemoveCommand) -> Result<()> {
    let opts = opts_from_cmd(cmd);
    pkg::manifest::dep_modifier::modify_dependencies(opts)?;
    Ok(())
}

fn opts_from_cmd(cmd: RemoveCommand) -> pkg::manifest::dep_modifier::ModifyOpts {
    pkg::manifest::dep_modifier::ModifyOpts {
        action: pkg::manifest::dep_modifier::Action::Remove,
        manifest_path: cmd.manifest.manisfest_path,
        package: cmd.package.package,
        source_path: None,
        git: None,
        branch: None,
        tag: None,
        rev: None,
        ipfs: None,
        contract_deps: cmd.section.contract_deps,
        salt: cmd.section.salt,
        ipfs_node: cmd.ipfs_node,
        dependencies: cmd.dependencies,
        dry_run: cmd.dry_run,
        offline: cmd.offline,
    }
}
