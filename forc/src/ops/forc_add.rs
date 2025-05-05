use crate::cli::AddCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, manifest::manager::Action};

pub fn add(cmd: AddCommand) -> Result<()> {
    let opts = opts_from_cmd(cmd);
    pkg::manifest::manager::modify_dependencies(opts)?;
    Ok(())
}

fn opts_from_cmd(cmd: AddCommand) -> pkg::manifest::manager::ModifyOpts {
    pkg::manifest::manager::ModifyOpts {
        // === Action ====
        action: Action::Add,
        // === Manifest Options ===
        manifest_path: cmd.manifest.manisfest_path,

        // === Package Selection ===
        package: cmd.package.package,

        // === Source ===
        source_path: cmd.source.path,
        git: cmd.source.git,
        branch: cmd.source.git_ref.branch,
        tag: cmd.source.git_ref.tag,
        rev: cmd.source.git_ref.rev,
        ipfs: cmd.source.ipfs,

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
