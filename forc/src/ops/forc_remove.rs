use crate::cli::RemoveCommand;
use anyhow::Result;
use forc_pkg::{self as pkg};

pub fn remove(cmd: RemoveCommand) -> Result<()> {
    let opts = opts_from_cmd(cmd);
    pkg::manifest::manager::remove_dependencies(opts)?;
    Ok(())
}

fn opts_from_cmd(cmd: RemoveCommand) -> pkg::manifest::manager::RemoveOpts {
    pkg::manifest::manager::RemoveOpts {
        // === Manifest Options ===
        manifest_path: cmd.manifest.manisfest_path,

        // === Package Selection ===
        package: cmd.package.package,

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
