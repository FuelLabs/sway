use crate::cli::BuildCommand;
use forc_pkg as pkg;
use forc_util::ForcResult;
use pkg::MemberFilter;

pub fn build(cmd: BuildCommand) -> ForcResult<pkg::Built> {
    let opts = opts_from_cmd(cmd);
    let built = pkg::build_with_options(&opts, None)?;
    Ok(built)
}

fn opts_from_cmd(cmd: BuildCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.build.pkg.path,
            offline: cmd.build.pkg.offline,
            terse: cmd.build.pkg.terse,
            locked: cmd.build.pkg.locked,
            output_directory: cmd.build.pkg.output_directory,
            ipfs_node: cmd.build.pkg.ipfs_node.unwrap_or_default(),
        },
        print: pkg::PrintOpts {
            ast: cmd.build.print.ast,
            dca_graph: cmd.build.print.dca_graph.clone(),
            dca_graph_url_format: cmd.build.print.dca_graph_url_format.clone(),
            asm: cmd.build.print.asm(),
            bytecode: cmd.build.print.bytecode,
            bytecode_spans: false,
            ir: cmd.build.print.ir(),
            reverse_order: cmd.build.print.reverse_order,
        },
        time_phases: cmd.build.print.time_phases,
        profile: cmd.build.print.profile,
        metrics_outfile: cmd.build.print.metrics_outfile,
        minify: pkg::MinifyOpts {
            json_abi: cmd.build.minify.json_abi,
            json_storage_slots: cmd.build.minify.json_storage_slots,
        },
        build_profile: cmd.build.profile.build_profile,
        release: cmd.build.profile.release,
        error_on_warnings: cmd.build.profile.error_on_warnings,
        binary_outfile: cmd.build.output.bin_file,
        debug_outfile: cmd.build.output.debug_file,
        hex_outfile: cmd.build.output.hex_file,
        build_target: cmd.build.build_target,
        tests: cmd.tests,
        member_filter: MemberFilter::default(),
        experimental: cmd.experimental.experimental,
        no_experimental: cmd.experimental.no_experimental,
    }
}
