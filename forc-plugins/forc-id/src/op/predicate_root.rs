use crate::cmd::predicate_root::Command as PredicateRootCommand;
use anyhow::Result;
use forc_pkg as pkg;
use pkg::build_with_options;
use sway_core::{fuel_prelude::fuel_tx::Contract, BuildTarget};
use tracing::info;

pub fn predicate_root(command: PredicateRootCommand) -> Result<()> {
    let build_options = build_opts_from_cmd(command);
    let built_pkgs = build_with_options(build_options)?;
    for (_, predicate_member) in built_pkgs.into_members(){
        let predicate_root = Contract::root_from_code(&predicate_member.as_ref().bytecode.bytes);
        info!("      Predicate root: {predicate_root}");
    }

    Ok(())
}

fn build_opts_from_cmd(cmd: PredicateRootCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.pkg.path.clone(),
            offline: cmd.pkg.offline,
            terse: cmd.pkg.terse,
            locked: cmd.pkg.locked,
            output_directory: cmd.pkg.output_directory.clone(),
            json_abi_with_callpaths: cmd.pkg.json_abi_with_callpaths,
        },
        print: pkg::PrintOpts {
            ast: cmd.print.ast,
            dca_graph: cmd.print.dca_graph,
            finalized_asm: cmd.print.finalized_asm,
            intermediate_asm: cmd.print.intermediate_asm,
            ir: cmd.print.ir,
        },
        time_phases: cmd.print.time_phases,
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        error_on_warnings: cmd.build_profile.error_on_warnings,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file.clone(),
        build_target: BuildTarget::default(),
        tests: false,
        member_filter: pkg::MemberFilter::only_predicates(),
    }
}
