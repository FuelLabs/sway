use crate::cli::ContractIdCommand;
use anyhow::{bail, Result};
use forc_pkg::{self as pkg, build_with_options, DumpOpts};
use forc_tracing::println_green;
use sway_core::{fuel_prelude::fuel_tx, BuildTarget};
use tracing::info;

pub fn contract_id(command: ContractIdCommand) -> Result<()> {
    let build_options = build_opts_from_cmd(&command);
    let build_plan = pkg::BuildPlan::from_pkg_opts(&build_options.pkg)?;
    // If a salt was specified but we have more than one member to build, there
    // may be ambiguity in how the salt should be applied, especially if the
    // workspace contains multiple contracts, and especially if one contract
    // member is the dependency of another (in which case salt should be
    // specified under `[contract-dependencies]`). Considering this, we have a
    // simple check to ensure that we only accept salt when working on a single
    // package. In the future, we can consider relaxing this to allow for
    // specifying a salt for workspacs, as long as there is only one
    // root contract member in the package graph.
    if command.salt.salt.is_some() && build_plan.member_nodes().count() > 1 {
        bail!(
            "A salt was specified when attempting to detect the contract id \
            for a workspace with more than one member.
              If you wish to find out contract id for a contract member with\
            salt, run this command for the member individually.
              If you wish to specify the salt for a contract dependency, \
            please do so within the `[contract-dependencies]` table."
        )
    }
    let built = build_with_options(&build_options, None)?;
    for (pinned_contract, built_contract) in built.into_members() {
        let salt = command
            .salt
            .salt
            .or_else(|| build_plan.salt(pinned_contract))
            .unwrap_or_else(fuel_tx::Salt::zeroed);
        let name = &pinned_contract.name;
        let storage_slots = built_contract.storage_slots.clone();
        let contract_id = pkg::contract_id(&built_contract.bytecode.bytes, storage_slots, &salt);
        println_green(&format!(" {name}"));
        info!("      Contract id: 0x{contract_id}");
    }
    Ok(())
}

fn build_opts_from_cmd(cmd: &ContractIdCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            ipfs_node: cmd.pkg.ipfs_node.clone().unwrap_or_default(),
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph.clone(),
            dca_graph_url_format: cmd.print.dca_graph_url_format.clone(),
            asm: cmd.print.asm(),
            bytecode: cmd.print.bytecode,
            bytecode_spans: false,
            ir: cmd.print.ir(),
            reverse_order: cmd.print.reverse_order,
        },
        dump: DumpOpts::default(),
        time_phases: cmd.print.time_phases,
        profile: cmd.print.profile,
        metrics_outfile: cmd.print.metrics_outfile.clone(),
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        error_on_warnings: cmd.build_profile.error_on_warnings,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        hex_outfile: cmd.build_output.hex_file.clone(),
        build_target: BuildTarget::default(),
        tests: false,
        member_filter: pkg::MemberFilter::only_contracts(),
        experimental: cmd.experimental.experimental.clone(),
        no_experimental: cmd.experimental.no_experimental.clone(),
    }
}
