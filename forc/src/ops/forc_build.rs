use crate::cli::BuildCommand;
use anyhow::Result;
use forc_pkg as pkg;

pub fn build(cmd: BuildCommand) -> Result<pkg::Built> {
    let opts = opts_from_cmd(cmd);
    let built = pkg::build_with_options(opts)?;
    Ok(built)
}

fn opts_from_cmd(cmd: BuildCommand) -> pkg::BuildOpts {
    let const_inject_map = std::collections::HashMap::new();
    pkg::BuildOpts {
        pkg: pkg::PkgOpts {
            path: cmd.build.pkg.path,
            offline: cmd.build.pkg.offline,
            terse: cmd.build.pkg.terse,
            locked: cmd.build.pkg.locked,
            output_directory: cmd.build.pkg.output_directory,
            json_abi_with_callpaths: cmd.build.pkg.json_abi_with_callpaths,
        },
        print: pkg::PrintOpts {
            ast: cmd.build.print.ast,
            dca_graph: cmd.build.print.dca_graph,
            finalized_asm: cmd.build.print.finalized_asm,
            intermediate_asm: cmd.build.print.intermediate_asm,
            ir: cmd.build.print.ir,
        },
        time_phases: cmd.build.print.time_phases,
        minify: pkg::MinifyOpts {
            json_abi: cmd.build.minify.json_abi,
            json_storage_slots: cmd.build.minify.json_storage_slots,
        },
        build_profile: cmd.build.profile.build_profile,
        release: cmd.build.profile.release,
        error_on_warnings: cmd.build.profile.error_on_warnings,
        binary_outfile: cmd.build.output.bin_file,
        debug_outfile: cmd.build.output.debug_file,
        build_target: cmd.build.build_target,
        tests: cmd.tests,
        const_inject_map,
        member_filter: Default::default(),
    }
}
