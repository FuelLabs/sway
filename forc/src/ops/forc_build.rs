use crate::cli::BuildCommand;
use anyhow::Result;
use forc_pkg::{self as pkg};

pub fn build(command: BuildCommand) -> Result<pkg::Compiled> {
    let build_options = pkg::BuildOptions {
        path: command.path,
        print_ast: command.print_ast,
        print_finalized_asm: command.print_finalized_asm,
        print_ir: command.print_ir,
        binary_outfile: command.binary_outfile,
        debug_outfile: command.debug_outfile,
        offline_mode: command.offline_mode,
        silent_mode: command.silent_mode,
        output_directory: command.output_directory,
        minify_json_abi: command.minify_json_abi,
        minify_json_storage_slots: command.minify_json_storage_slots,
        locked: command.locked,
        build_profile: command.build_profile,
        release: command.release,
        time_phases: command.time_phases,
        print_intermediate_asm: command.print_intermediate_asm,
        generate_logged_types: command.generate_logged_types,
    };
    let compiled = pkg::build_with_options(build_options)?;
    Ok(compiled)
}
