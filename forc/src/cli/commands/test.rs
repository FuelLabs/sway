use crate::cli;
use anyhow::{bail, Result};
use clap::Parser;
use forc_pkg as pkg;

/// Run the Sway unit tests for the current project.
///
/// NOTE: This feature is not yet implemented. Track progress at the following link:
/// https://github.com/FuelLabs/sway/issues/1832
///
/// NOTE: Previously this command was used to support Rust integration testing, however the
/// provided behaviour served no benefit over running `cargo test` directly. The proposal to change
/// the behaviour to support unit testing can be found at the following link:
/// https://github.com/FuelLabs/sway/issues/1833
///
/// Sway unit tests are functions decorated with the `#[test]` attribute. Each test is compiled as
/// a unique entry point for a single program and has access to the namespace of the module in
/// which it is declared.
///
/// Unit tests decorated with the `#[test(script)]` attribute that are declared within `contract`
/// projects may also call directly into their associated contract's ABI.
///
/// Upon successful compilation, test scripts are executed to their completion. A test is
/// considered a failure in the case that a revert (`rvrt`) instruction is encountered during
/// execution. Otherwise, it is considered a success.
#[derive(Debug, Parser)]
pub struct Command {
    #[clap(flatten)]
    pub build: cli::shared::Build,
    /// Enable the incomplete, unstable unit testing support. Warning: May create a black hole!
    ///
    /// This is purely for Sway devs to test the unit testing functionality until it stabilises,
    /// and will be removed upon stabilisation of the command.
    #[clap(long)]
    pub unstable: bool,
    /// When specified, only tests containing the given string will be executed.
    pub filter: Option<String>,
}

pub(crate) fn exec(cmd: Command) -> Result<()> {
    if !cmd.unstable {
        bail!(
            r#"
Sway unit testing is not yet implemented. Track progress at the following link:

https://github.com/FuelLabs/sway/issues/1832

NOTE: Previously this command was used to support Rust integration testing,
however the provided behaviour served no benefit over running `cargo test`
directly. The proposal to change the behaviour to support unit testing can be
found at the following link:

https://github.com/FuelLabs/sway/issues/1833
"#
        );
    }

    if let Some(ref _filter) = cmd.filter {
        bail!("unit test filter not yet supported");
    }

    let opts = opts_from_cmd(cmd);
    let tested = forc_test::test(opts)?;

    // Eventually we'll print this in a fancy manner, but this will do for testing.
    dbg!(&tested);

    Ok(())
}

fn opts_from_cmd(cmd: Command) -> forc_test::Opts {
    forc_test::Opts {
        pkg: pkg::PkgOpts {
            path: cmd.build.path,
            offline: cmd.build.offline_mode,
            terse: cmd.build.terse_mode,
            locked: cmd.build.locked,
            output_directory: cmd.build.output_directory,
        },
        print: pkg::PrintOpts {
            ast: cmd.build.print_ast,
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
    }
}
