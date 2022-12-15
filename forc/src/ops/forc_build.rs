use crate::cli::BuildCommand;
use anyhow::Result;
use forc_pkg as pkg;

pub fn build(cmd: BuildCommand) -> Result<pkg::Built> {
    let opts = opts_from_cmd(cmd);
    let built = pkg::build_with_options(opts)?;
    Ok(built)
}

fn opts_from_cmd(cmd: BuildCommand) -> pkg::BuildOpts {
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.build.path,
            offline: cmd.build.offline_mode,
            terse: cmd.build.terse_mode,
            locked: cmd.build.locked,
            output_directory: cmd.build.output_directory,
        },
        print: pkg::PrintOpts {
            ast: cmd.build.print_ast,
            dca_graph: cmd.build.print_dca_graph,
            finalized_asm: cmd.build.print_finalized_asm,
            intermediate_asm: cmd.build.print_intermediate_asm,
            ir: cmd.build.print_ir,
        },
        minify: pkg::MinifyOpts {
            json_abi: cmd.build.minify_json_abi,
            json_storage_slots: cmd.build.minify_json_storage_slots,
        },
        build_profile: cmd.build.build_profile,
        release: cmd.build.release,
        time_phases: cmd.build.time_phases,
        binary_outfile: cmd.build.binary_outfile,
        debug_outfile: cmd.build.debug_outfile,
        tests: cmd.tests,
    }
}
