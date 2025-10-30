use crate::cli;
use ansiterm::Colour;
use clap::Parser;
use forc_pkg as pkg;
use forc_test::{GasCostsSource, TestFilter, TestResult, TestRunnerCount, TestedPackage};
use forc_tracing::println_action_green;
use forc_util::{
    tx_utils::{decode_fuel_vm_log_data, format_log_receipts},
    ForcError, ForcResult,
};
use fuel_abi_types::{abi::program::PanickingCall, revert_info::RevertKind};
use sway_core::{asm_generation::ProgramABI, fuel_prelude::fuel_tx::Receipt};
use tracing::info;

forc_util::cli_examples! {
    crate::cli::Opt {
        [ Run test => "forc test" ]
        [ Run test with a filter => "forc test $filter" ]
        [ Run test without any output => "forc test --silent" ]
        [ Run test without creating or update the lock file  => "forc test --locked" ]
    }
}

/// Run the Sway unit tests for the current project.
///
/// NOTE: Previously this command was used to support Rust integration testing, however the
/// provided behavior served no benefit over running `cargo test` directly. The proposal to change
/// the behavior to support unit testing can be found at the following link:
/// https://github.com/FuelLabs/sway/issues/1833
///
/// Sway unit tests are functions decorated with the `#[test]` attribute. Each test is compiled as
/// a unique entry point for a single program and has access to the namespace of the module in
/// which it is declared.
///
/// Upon successful compilation, test scripts are executed to their completion. A test is
/// considered a failure in the case that a revert (`rvrt`) instruction is encountered during
/// execution, unless it is specified as `#[test(should_revert)]`. Otherwise, it is considered a success.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc test", version, after_help = help())]
pub struct Command {
    #[clap(flatten)]
    pub build: cli::shared::Build,
    #[clap(flatten)]
    pub test_print: TestPrintOpts,
    /// When specified, only tests containing the given string will be executed.
    pub filter: Option<String>,
    #[clap(long)]
    /// When specified, only the test exactly matching the given string will be executed.
    pub filter_exact: bool,
    #[clap(long)]
    /// Number of threads to utilize when running the tests. By default, this is the number of
    /// threads available in your system.
    pub test_threads: Option<usize>,
    #[clap(flatten)]
    pub experimental: sway_features::CliFields,
    /// Source of the gas costs values used to calculate gas costs of test executions.
    ///
    /// If not provided, a built-in set of gas costs values will be used.
    /// These are the gas costs values of the Fuel mainnet as of time of
    /// the release of the `forc` version being used.
    ///
    /// The mainnet and testnet options will fetch the current gas costs values from
    /// their respective networks.
    ///
    /// Alternatively, the gas costs values can be specified as a file path
    /// to a local JSON file containing the gas costs values.
    ///
    /// [possible values: built-in, mainnet, testnet, <FILE_PATH>]
    #[clap(long)]
    pub gas_costs: Option<GasCostsSource>,
}

/// The set of options provided for controlling output of a test.
#[derive(Parser, Debug, Clone)]
pub struct TestPrintOpts {
    #[clap(long = "pretty")]
    /// Pretty-print the logs emitted from tests.
    pub pretty_print: bool,
    /// Print decoded `Log` and `LogData` receipts for tests.
    #[clap(long, short = 'l')]
    pub logs: bool,
    /// Print the raw logs for tests.
    #[clap(long)]
    pub raw_logs: bool,
    /// Print the revert information for tests.
    #[clap(long)]
    pub reverts: bool,
    /// Print the output of debug ecals for tests.
    #[clap(long)]
    pub dbgs: bool,
}

pub(crate) fn exec(cmd: Command) -> ForcResult<()> {
    let test_runner_count = match cmd.test_threads {
        Some(runner_count) => TestRunnerCount::Manual(runner_count),
        None => TestRunnerCount::Auto,
    };

    let test_print_opts = cmd.test_print.clone();
    let test_filter_phrase = cmd.filter.clone();
    let test_filter = test_filter_phrase.as_ref().map(|filter_phrase| TestFilter {
        filter_phrase,
        exact_match: cmd.filter_exact,
    });
    let gas_costs_values = cmd
        .gas_costs
        .as_ref()
        .unwrap_or(&GasCostsSource::BuiltIn)
        .provide_gas_costs()?;
    let opts = opts_from_cmd(cmd);
    let built_tests = forc_test::build(opts)?;
    let start = std::time::Instant::now();
    let test_count = built_tests.test_count(test_filter.as_ref());
    let num_tests_running = test_count.total - test_count.ignored;
    let num_tests_ignored = test_count.ignored;
    println_action_green(
        "Running",
        &format!(
            "{} {}, filtered {} {}",
            num_tests_running,
            formatted_test_count_string(num_tests_running),
            num_tests_ignored,
            formatted_test_count_string(num_tests_ignored)
        ),
    );
    let tested = built_tests.run(test_runner_count, test_filter, gas_costs_values)?;
    let duration = start.elapsed();

    // Eventually we'll print this in a fancy manner, but this will do for testing.
    let all_tests_passed = match tested {
        forc_test::Tested::Workspace(pkgs) => {
            for pkg in &pkgs {
                let built = &pkg.built.descriptor.name;
                info!("\ntested -- {built}\n");
                print_tested_pkg(pkg, &test_print_opts)?;
            }
            info!("");
            println_action_green("Finished", &format!("in {duration:?}"));
            pkgs.iter().all(|pkg| pkg.tests_passed())
        }
        forc_test::Tested::Package(pkg) => {
            print_tested_pkg(&pkg, &test_print_opts)?;
            pkg.tests_passed()
        }
    };

    if all_tests_passed {
        Ok(())
    } else {
        let forc_error: ForcError = "Some tests failed.".into();
        const FAILING_UNIT_TESTS_EXIT_CODE: u8 = 101;
        Err(forc_error.exit_code(FAILING_UNIT_TESTS_EXIT_CODE))
    }
}

fn print_tested_pkg(pkg: &TestedPackage, test_print_opts: &TestPrintOpts) -> ForcResult<()> {
    let succeeded = pkg.tests.iter().filter(|t| t.passed()).count();
    let failed = pkg.tests.len() - succeeded;
    let mut failed_tests = Vec::new();

    let program_abi = match &pkg.built.program_abi {
        ProgramABI::Fuel(fuel_abi) => Some(fuel_abi),
        _ => None,
    };

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

        print_test_output(test, program_abi, Some(test_print_opts))?;

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
        info!("\n   {}", Colour::Red.paint("failures:"));
        for failed_test in failed_tests {
            let failed_test_name = &failed_test.name;
            let failed_test_details = failed_test.details()?;
            let path = &*failed_test_details.file_path;
            let line_number = failed_test_details.line_number;

            info!("      test {failed_test_name}, {path:?}:{line_number}");

            print_test_output(failed_test, program_abi, None)?;
            info!("\n");
        }
    }

    let pkg_test_durations: std::time::Duration = pkg
        .tests
        .iter()
        .map(|test_result| test_result.duration)
        .sum();
    info!(
        "\ntest result: {}. {} passed; {} failed; finished in {:?}",
        color.paint(state),
        succeeded,
        failed,
        pkg_test_durations
    );

    Ok(())
}

/// Prints the output of a test result, including debug output, logs, and revert information.
/// If the `test_print_opts` is `None`, it defaults to printing all the output.
fn print_test_output(
    test: &TestResult,
    program_abi: Option<&fuel_abi_types::abi::program::ProgramABI>,
    test_print_opts: Option<&TestPrintOpts>,
) -> Result<(), ForcError> {
    const TEST_NAME_INDENT: &str = "           ";

    let print_reverts = test_print_opts.map(|opts| opts.reverts).unwrap_or(true);
    let print_dbgs = test_print_opts.map(|opts| opts.dbgs).unwrap_or(true);
    let print_logs = test_print_opts.map(|opts| opts.logs).unwrap_or(true);
    let print_raw_logs = test_print_opts.map(|opts| opts.raw_logs).unwrap_or(true);
    let pretty_print = test_print_opts
        .map(|opts| opts.pretty_print)
        .unwrap_or(true);

    if print_reverts {
        if let Some(revert_info) = test.revert_info(program_abi, &test.logs) {
            info!(
                "{TEST_NAME_INDENT}revert code: {:x}",
                revert_info.revert_code
            );
            match revert_info.kind {
                RevertKind::RawRevert => {}
                RevertKind::KnownErrorSignal { err_msg } => {
                    info!("{TEST_NAME_INDENT} └─ error message: {err_msg}");
                }
                RevertKind::Panic {
                    err_msg,
                    err_val,
                    pos,
                    backtrace,
                } => {
                    if let Some(err_msg) = err_msg {
                        info!("{TEST_NAME_INDENT} ├─ panic message: {err_msg}");
                    }
                    if let Some(err_val) = err_val {
                        info!("{TEST_NAME_INDENT} ├─ panic value:   {err_val}");
                    }
                    info!(
                        "{TEST_NAME_INDENT} {} panicked:      in {}",
                        if backtrace.is_empty() {
                            "└─"
                        } else {
                            "├─"
                        },
                        pos.function
                    );
                    info!(
                        "{TEST_NAME_INDENT} {}                 └─ at {}, {}:{}:{}",
                        if backtrace.is_empty() { "  " } else { "│ " },
                        pos.pkg,
                        pos.file,
                        pos.line,
                        pos.column
                    );

                    fn print_backtrace_call(call: &PanickingCall, is_first: bool) {
                        // The `__entry` function is a part of a backtrace,
                        // but we don't want to show it, since it is an internal implementation detail.
                        if call.pos.function.ends_with("::__entry")
                            || call.pos.function.eq("__entry")
                        {
                            return;
                        }

                        let prefix = if is_first {
                            "└─ backtrace:"
                        } else {
                            "             "
                        };

                        info!(
                            "{TEST_NAME_INDENT} {prefix}     called in {}",
                            call.pos.function
                        );
                        info!(
                            "{TEST_NAME_INDENT}                    └─ at {}, {}:{}:{}",
                            call.pos.pkg, call.pos.file, call.pos.line, call.pos.column
                        );
                    }

                    if let Some((first, others)) = backtrace.split_first() {
                        print_backtrace_call(first, true);
                        for call in others {
                            print_backtrace_call(call, false);
                        }
                    }
                }
            }
        }
    }

    if print_dbgs && !test.ecal.captured.is_empty() {
        info!("{TEST_NAME_INDENT}debug output:");
        for captured in test.ecal.captured.iter() {
            captured.apply();
        }
    }

    if print_logs && !test.logs.is_empty() {
        if let Some(program_abi) = program_abi {
            info!("{TEST_NAME_INDENT}decoded log values:");
            for log in &test.logs {
                if let Receipt::LogData {
                    rb,
                    data: Some(data),
                    ..
                } = log
                {
                    let decoded_log_data =
                        decode_fuel_vm_log_data(&rb.to_string(), data, program_abi)?;
                    let var_value = decoded_log_data.value;
                    info!("{var_value}, log rb: {rb}");
                }
            }
        }
    }

    if print_raw_logs && !test.logs.is_empty() {
        let formatted_logs = format_log_receipts(&test.logs, pretty_print)?;
        info!("{TEST_NAME_INDENT}raw logs:\n{}", formatted_logs);
    }

    Ok(())
}

fn opts_from_cmd(cmd: Command) -> forc_test::TestOpts {
    forc_test::TestOpts {
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
        experimental: cmd.experimental.experimental,
        no_experimental: cmd.experimental.no_experimental,
    }
}

fn formatted_test_count_string(count: usize) -> &'static str {
    if count == 1 {
        "test"
    } else {
        "tests"
    }
}
