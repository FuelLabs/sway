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
    let mut failed_tests = Vec::new();
    for test in &pkg.tests {
        let test_passed = test.passed();
        let (state, color) = match test_passed {
            true => ("ok", Colour::Green),
            false => ("FAILED", Colour::Red),
        };
        info!(
            "      test {} ... {} ({:?}, {} gas)",
            test.name,
            color.paint(state),
            test.duration,
            test.gas_used
        );

        // If logs are enabled, print them.
        if test_print_opts.print_logs {
            let logs = &test.logs;
            let formatted_logs = format_log_receipts(logs, test_print_opts.pretty_print)?;
            info!("{}", formatted_logs);
        }

        // If the test is failing, save the test result for printing the details later on.
        if !test_passed {
            failed_tests.push(test);
        }
    }
    let (state, color) = match succeeded == pkg.tests.len() {
        true => ("OK", Colour::Green),
        false => ("FAILED", Colour::Red),
    };
    if failed != 0 {
        info!("\n   failures:");
        for failed_test in failed_tests {
            let failed_test_name = &failed_test.name;
            let failed_test_details = failed_test.details()?;
            let path = &*failed_test_details.file_path;
            let line_number = failed_test_details.line_number;
            let logs = &failed_test.logs;
            let formatted_logs = format_log_receipts(logs, test_print_opts.pretty_print)?;
            info!(
                "      - test {}, {:?}:{} ",
                failed_test_name, path, line_number
            );
            if let Some(revert_code) = failed_test.revert_code() {
                // If we have a revert_code, try to get a known error signal
                let mut failed_info_str = format!("        revert code: {revert_code:x}");
                let error_signal = failed_test.error_signal().ok();
                if let Some(error_signal) = error_signal {
                    let error_signal_str = error_signal.to_string();
                    failed_info_str.push_str(&format!(" -- {error_signal_str}"));
                }
                info!("{failed_info_str}");
            }
            info!("        Logs: {}", formatted_logs);
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
    }
}
