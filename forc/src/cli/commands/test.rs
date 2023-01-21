use crate::cli;
use ansi_term::Colour;
use anyhow::{bail, Result};
use clap::Parser;
use forc_pkg as pkg;
use forc_test::TestedPackage;
use forc_util::format_log_receipts;
use tracing::info;

/// Run the Sway unit tests for the current project.
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
    #[clap(flatten)]
    pub test_print: TestPrintOpts,
    /// When specified, only tests containing the given string will be executed.
    pub filter: Option<String>,
}

/// The set of options provided for controlling output of a test.
#[derive(Parser, Debug, Clone)]
pub struct TestPrintOpts {
    #[clap(long = "pretty-print", short = 'r')]
    /// Pretty-print the logs emiited from tests.
    pub pretty_print: bool,
    /// Print `Log` and `LogData` receipts for tests.
    #[clap(long = "logs", short = 'l')]
    pub print_logs: bool,
}

pub(crate) fn exec(cmd: Command) -> Result<()> {
    if let Some(ref _filter) = cmd.filter {
        bail!("unit test filter not yet supported");
    }

    let test_print_opts = cmd.test_print.clone();
    let opts = opts_from_cmd(cmd);
    let built_tests = forc_test::build(opts)?;
    let start = std::time::Instant::now();
    info!("   Running {} tests", built_tests.test_count());
    let tested = built_tests.run()?;
    let duration = start.elapsed();

    // Eventually we'll print this in a fancy manner, but this will do for testing.
    match tested {
        forc_test::Tested::Workspace(pkgs) => {
            for pkg in pkgs {
                let built = &pkg.built.pkg_name;
                info!("\n   tested -- {built}\n");
                print_tested_pkg(&pkg, &test_print_opts)?;
            }
            info!("\n   Finished in {:?}", duration);
        }
        forc_test::Tested::Package(pkg) => print_tested_pkg(&pkg, &test_print_opts)?,
    };

    Ok(())
}

fn print_tested_pkg(pkg: &TestedPackage, test_print_opts: &TestPrintOpts) -> Result<()> {
    let succeeded = pkg.tests.iter().filter(|t| t.passed()).count();
    let failed = pkg.tests.len() - succeeded;
    let mut failed_test_details = Vec::new();
    for test in &pkg.tests {
        let test_passed = test.passed();
        let (state, color) = match test_passed {
            true => ("ok", Colour::Green),
            false => ("FAILED", Colour::Red),
        };
        info!(
            "      test {} ... {} ({:?})",
            test.name,
            color.paint(state),
            test.duration
        );

        // If logs are enabled, print them.
        if test_print_opts.print_logs {
            let logs = &test.logs;
            let formatted_logs = format_log_receipts(logs, test_print_opts.pretty_print)?;
            info!("{}", formatted_logs);
        }

        // If the test is failing, save details.
        if !test_passed {
            let details = test.details()?;
            failed_test_details.push((test.name.clone(), details));
        }
    }
    let (state, color) = match succeeded == pkg.tests.len() {
        true => ("OK", Colour::Green),
        false => ("FAILED", Colour::Red),
    };
    if failed != 0 {
        info!("\n   failures:");
        for (failed_test_name, failed_test_detail) in failed_test_details {
            let path = &*failed_test_detail.file_path;
            let line_number = failed_test_detail.line_number;
            info!(
                "      - test {}, {:?}:{} ",
                failed_test_name, path, line_number
            );
        }
        info!("\n");
    }

    let pkg_test_durations: std::time::Duration = pkg
        .tests
        .iter()
        .map(|test_result| test_result.duration)
        .sum();
    info!(
        "   Result: {}. {} passed. {} failed. Finished in {:?}.",
        color.paint(state),
        succeeded,
        failed,
        pkg_test_durations
    );

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
            dca_graph: cmd.build.print_dca_graph,
            finalized_asm: cmd.build.print_finalized_asm,
            intermediate_asm: cmd.build.print_intermediate_asm,
            ir: cmd.build.print_ir,
        },
        minify: pkg::MinifyOpts {
            json_abi: cmd.build.minify_json_abi,
            json_storage_slots: cmd.build.minify_json_storage_slots,
        },
        build_target: cmd.build.build_target,
        build_profile: cmd.build.build_profile,
        release: cmd.build.release,
        time_phases: cmd.build.time_phases,
        binary_outfile: cmd.build.binary_outfile,
        debug_outfile: cmd.build.debug_outfile,
    }
}
