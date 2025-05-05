use crate::cli::RemoveCommand;
use anyhow::Result;
use forc_pkg::{self as pkg};

pub fn remove(cmd: RemoveCommand) -> Result<()> {
    let opts = opts_from_cmd(cmd);
    pkg::manifest::manager::modify_dependencies(opts)?;
    Ok(())
}

fn opts_from_cmd(cmd: RemoveCommand) -> pkg::manifest::manager::ModifyOpts {
    pkg::manifest::manager::ModifyOpts {
        // === Action ====
        action: pkg::manifest::manager::Action::Add,
        // === Manifest Options ===
        manifest_path: cmd.manifest.manisfest_path,

        // === Package Selection ===
        package: cmd.package.package,

        // === Source ===
        source_path: None,
        git: None,
        branch: None,
        tag: None,
        rev: None,
        ipfs: None,

        // === Section ===
        contract_deps: cmd.section.contract_deps,
        salt: cmd.section.salt,

        // === IPFS Node ===
        ipfs_node: cmd.ipfs_node,

        // === Dependencies & Flags ===
        dependencies: cmd.dependencies,
        dry_run: cmd.dry_run,
        offline: cmd.offline,
    }
}
