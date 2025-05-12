use crate::cli::AddCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, manifest::dep_modifier::Action};

pub fn add(cmd: AddCommand) -> Result<()> {
    let opts = opts_from_cmd(cmd);
    pkg::manifest::dep_modifier::modify_dependencies(opts)?;
    Ok(())
}

fn opts_from_cmd(cmd: AddCommand) -> pkg::manifest::dep_modifier::ModifyOpts {
    pkg::manifest::dep_modifier::ModifyOpts {
        action: Action::Add,
        manifest_path: cmd.manifest.manisfest_path,
        package: cmd.package.package,
        source_path: cmd.source.path,
        git: cmd.source.git,
        branch: cmd.source.git_ref.branch,
        tag: cmd.source.git_ref.tag,
        rev: cmd.source.git_ref.rev,
        ipfs: cmd.source.ipfs,
        contract_deps: cmd.section.contract_deps,
        salt: cmd.section.salt,
        ipfs_node: cmd.ipfs_node,
        dependencies: cmd.dependencies,
        dry_run: cmd.dry_run,
        offline: cmd.offline,
    }
}
