use crate::cli::PredicateRootCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, build_with_options};
use sway_core::BuildTarget;

pub fn predicate_root(command: PredicateRootCommand) -> Result<()> {
    let build_options = build_opts_from_cmd(command);
    // Building predicates will output the predicate root by default.
    // So to display all predicate roots in the current workspace we just need to build the
    // workspace with a member filter that filters out every project type other than predicates.
    build_with_options(build_options)?;
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
            dca_graph_url_format: cmd.print.dca_graph_url_format.clone(),
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
        debug_outfile: cmd.build_output.debug_file,
        build_target: BuildTarget::default(),
        tests: false,
        member_filter: pkg::MemberFilter::only_predicates(),
        experimental_private_modules: cmd.build_profile.experimental_private_modules,
    }
}
