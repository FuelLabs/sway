use crate::cli::{shared::IrCliOpt, PredicateRootCommand};
use anyhow::Result;
use forc_pkg::{self as pkg, build_with_options, DumpOpts};
use sway_core::{BuildBackend, BuildTarget, IrCli};

pub fn predicate_root(command: PredicateRootCommand) -> Result<()> {
    let build_options = build_opts_from_cmd(command);
    // Building predicates will output the predicate root by default.
    // So to display all predicate roots in the current workspace we just need to build the
    // workspace with a member filter that filters out every project type other than predicates.
    build_with_options(&build_options, None)?;
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
            ipfs_node: cmd.pkg.ipfs_node.unwrap_or_default(),
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
        verify_ir: cmd
            .verify_ir
            .as_ref()
            .map_or(IrCli::default(), |opts| IrCliOpt::from(opts).0),
        dump: DumpOpts::default(),
        time_phases: cmd.print.time_phases,
        profile: cmd.print.profile,
        metrics_outfile: cmd.print.metrics_outfile,
        minify: pkg::MinifyOpts {
            json_abi: cmd.minify.json_abi,
            json_storage_slots: cmd.minify.json_storage_slots,
        },
        build_profile: cmd.build_profile.build_profile.clone(),
        release: cmd.build_profile.release,
        error_on_warnings: cmd.build_profile.error_on_warnings,
        binary_outfile: cmd.build_output.bin_file.clone(),
        debug_outfile: cmd.build_output.debug_file,
        hex_outfile: cmd.build_output.hex_file.clone(),
        build_target: BuildTarget::default(),
        backend: BuildBackend::Fuel,
        tests: false,
        member_filter: pkg::MemberFilter::only_predicates(),
        experimental: cmd.experimental.experimental,
        no_experimental: cmd.experimental.no_experimental,
        no_output: false,
    }
}
