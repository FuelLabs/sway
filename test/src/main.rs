mod e2e_vm_tests;
mod ir_generation;
mod reduced_std_libs;
mod snapshot;
mod test_consistency;

use anyhow::Result;
use clap::Parser;
use forc::cli::shared::{IrCliOpt, PrintAsmCliOpt};
use forc_test::GasCostsSource;
use forc_tracing::init_tracing_subscriber;
use fuel_vm::prelude::GasCostsValues;
use std::str::FromStr;
use sway_core::{BuildTarget, IrCli, PrintAsm};
use tracing::Instrument;

#[derive(Parser)]
struct Cli {
    /// Only run tests matching this regex
    #[arg(value_name = "REGEX")]
    include: Option<regex::Regex>,

    /// Exclude tests matching this regex
    #[arg(long, short, value_name = "REGEX")]
    exclude: Option<regex::Regex>,

    /// Skip all tests until a test matches this regex
    #[arg(long, value_name = "REGEX")]
    skip_until: Option<regex::Regex>,

    /// Only run tests with ABI JSON output validation
    #[arg(long, visible_alias = "abi")]
    abi_only: bool,

    /// Only run tests with no `std` dependencies
    #[arg(long, visible_alias = "no_std")]
    no_std_only: bool,

    /// Only run tests that deploy contracts
    #[arg(long, visible_alias = "contract")]
    contract_only: bool,

    /// Only run tests that run "forc test"
    #[arg(long, visible_alias = "forc-test")]
    forc_test_only: bool,

    /// Only run the first test
    #[arg(long, visible_alias = "first")]
    first_only: bool,

    /// Only run the tests that emit performance data (gas usages and bytecode sizes)
    #[arg(long)]
    perf_only: bool,

    /// Print out warnings, errors, and output of print options
    ///
    /// This option is ignored if tests are run in parallel.
    #[arg(long, env = "SWAY_TEST_VERBOSE")]
    verbose: bool,

    /// Compile Sway code in release mode
    #[arg(long, short)]
    release: bool,

    /// Intended for use in CI to ensure test lock files are up to date
    #[arg(long)]
    locked: bool,

    /// Build target
    #[arg(long, visible_alias = "target")]
    build_target: Option<String>,

    /// Update all output files
    #[arg(long)]
    update_output_files: bool,

    /// Print out the specified IR (separate options with comma), if the verbose option is on
    ///
    /// This option is ignored if tests are run in parallel.
    #[arg(long, num_args(1..=18), value_parser = clap::builder::PossibleValuesParser::new(IrCliOpt::cli_options()))]
    print_ir: Option<Vec<String>>,

    /// Verify the generated Sway IR (Intermediate Representation).
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(IrCliOpt::cli_options()))]
    verify_ir: Option<Vec<String>>,

    /// Print out the specified ASM (separate options with comma), if the verbose option is on
    ///
    /// This option is ignored if tests are run in parallel.
    #[arg(long, num_args(1..=5), value_parser = clap::builder::PossibleValuesParser::new(&PrintAsmCliOpt::CLI_OPTIONS))]
    print_asm: Option<Vec<String>>,

    /// Print out the final bytecode, if the verbose option is on
    ///
    /// This option is ignored if tests are run in parallel.
    #[arg(long)]
    print_bytecode: bool,

    #[command(flatten)]
    experimental: sway_features::CliFields,

    /// Only run tests of a particular kind
    #[arg(long, short, num_args(1..=4), value_parser = clap::builder::PossibleValuesParser::new(&TestKindOpt::CLI_OPTIONS))]
    kind: Option<Vec<String>>,

    /// Run only the exact test provided by an absolute path to a `test.toml` or `test.<feature>.toml` file
    ///
    /// This flag is used internally for parallel test execution, and is not intended for general use.
    #[arg(long, hide = true)]
    exact: Option<String>,

    /// Run tests sequentially (not in parallel)
    #[arg(long, short)]
    sequential: bool,

    /// Write compilation output (e.g., bytecode, ABI JSON, storage slots JSON, etc.) to the filesystem
    ///
    /// This is primarily useful for troubleshooting test failures.
    /// Output files are written to the `out` directory within each test's directory.
    ///
    /// This option is ignored if tests are run in parallel.
    #[arg(long)]
    write_output: bool,

    /// Write performance data (gas usages and bytecode sizes) to the filesystem
    ///
    /// Output files are written to the `test/perf_out` directory.
    #[arg(long)]
    perf: bool,

    /// Source of the gas costs values used to calculate gas costs of
    /// unit tests and scripts executions.
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
    /// This option is ignored if tests are run in parallel.
    ///
    /// [possible values: built-in, mainnet, testnet, <FILE_PATH>]
    #[clap(long)]
    pub gas_costs: Option<GasCostsSource>,
}

#[derive(Default, Debug, Clone)]
pub struct TestKind {
    pub e2e: bool,
    pub ir: bool,
    pub snapshot: bool,
}

impl TestKind {
    fn all() -> Self {
        Self {
            e2e: true,
            ir: true,
            snapshot: true,
        }
    }
}

pub struct TestKindOpt(pub TestKind);

impl TestKindOpt {
    const E2E: &'static str = "e2e";
    const IR: &'static str = "ir";
    const SNAPSHOT: &'static str = "snapshot";
    const ALL: &'static str = "all";
    pub const CLI_OPTIONS: [&'static str; 4] = [Self::E2E, Self::IR, Self::SNAPSHOT, Self::ALL];
}

impl From<&Vec<String>> for TestKindOpt {
    fn from(value: &Vec<String>) -> Self {
        let contains_opt = |opt: &str| value.iter().any(|val| *val == opt);

        let test_kind = if contains_opt(Self::ALL) {
            TestKind::all()
        } else {
            TestKind {
                e2e: contains_opt(Self::E2E),
                ir: contains_opt(Self::IR),
                snapshot: contains_opt(Self::SNAPSHOT),
            }
        };

        Self(test_kind)
    }
}

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub include: Option<regex::Regex>,
    pub exclude: Option<regex::Regex>,
    pub skip_until: Option<regex::Regex>,
    pub abi_only: bool,
    pub no_std_only: bool,
    pub contract_only: bool,
    pub first_only: bool,
    pub forc_test_only: bool,
    pub perf_only: bool,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub build_target: BuildTarget,
    pub locked: bool,
    pub verbose: bool,
    pub release: bool,
    pub update_output_files: bool,
    pub print_ir: IrCli,
    pub verify_ir: IrCli,
    pub print_asm: PrintAsm,
    pub print_bytecode: bool,
    pub experimental: sway_features::CliFields,
    pub write_output: bool,
    pub perf: bool,
    pub gas_costs_values: GasCostsValues,
}

#[derive(Debug, Clone)]
pub struct RunKindConfig {
    pub kind: TestKind,
    pub sequential: bool,
}

// We want to use the "current_thread" flavor because running
// Tokio runtime on another thread brings only overhead with
// no benefits, especially when running tests in parallel.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing_subscriber(Default::default());

    let cli = Cli::parse();

    let build_target = match cli.build_target {
        Some(target) => match BuildTarget::from_str(target.as_str()) {
            Ok(target) => target,
            _ => panic!("Unexpected build target: {}", target),
        },
        None => BuildTarget::default(),
    };

    if let Some(exact) = &cli.exact {
        if !std::fs::exists(exact).unwrap_or(false) {
            panic!("The --exact test path does not exist: {exact}\nThe --exact path must be an absolute path to an existing `test.toml` or `test.<feature>.toml` file");
        }

        let run_config = RunConfig {
            // Take over options that are supported when running tests in parallel.
            locked: cli.locked,
            release: cli.release,
            build_target,
            experimental: cli.experimental,
            update_output_files: cli.update_output_files,
            verify_ir: cli
                .verify_ir
                .as_ref()
                .map_or(IrCli::default(), |opts| IrCliOpt::from(opts).0),
            perf: cli.perf,
            // Always use the built-in gas costs values when running tests in parallel.
            gas_costs_values: GasCostsValues::default(),
            // Ignore options that are not supported when running tests in parallel.
            print_ir: IrCli::none(),
            print_asm: PrintAsm::none(),
            print_bytecode: false,
            write_output: false,
            verbose: false,
        };

        e2e_vm_tests::run_exact(exact, &run_config).await?;

        return Ok(());
    }

    let run_kind_config = RunKindConfig {
        kind: cli
            .kind
            .as_ref()
            .map_or(TestKind::all(), |opts| TestKindOpt::from(opts).0),
        sequential: cli.sequential,
    };

    let filter_config = FilterConfig {
        include: cli.include.clone(),
        exclude: cli.exclude,
        skip_until: cli.skip_until,
        abi_only: cli.abi_only,
        no_std_only: cli.no_std_only,
        contract_only: cli.contract_only,
        forc_test_only: cli.forc_test_only,
        first_only: cli.first_only,
        perf_only: cli.perf_only,
    };

    let run_config = RunConfig {
        locked: cli.locked,
        verbose: cli.verbose,
        release: cli.release,
        build_target,
        experimental: cli.experimental,
        update_output_files: cli.update_output_files,
        print_ir: cli
            .print_ir
            .as_ref()
            .map_or(IrCli::default(), |opts| IrCliOpt::from(opts).0),
        verify_ir: cli
            .verify_ir
            .as_ref()
            .map_or(IrCli::default(), |opts| IrCliOpt::from(opts).0),
        print_asm: cli
            .print_asm
            .as_ref()
            .map_or(PrintAsm::default(), |opts| PrintAsmCliOpt::from(opts).0),
        print_bytecode: cli.print_bytecode,
        write_output: cli.write_output,
        perf: cli.perf,
        gas_costs_values: cli
            .gas_costs
            .as_ref()
            .map_or(Ok(GasCostsValues::default()), |source| {
                source.provide_gas_costs()
            })?,
    };

    // Check that the tests are consistent
    test_consistency::check()?;

    // Create reduced versions of the `std` library
    reduced_std_libs::create()?;

    // Run E2E tests
    if run_kind_config.kind.e2e {
        if run_kind_config.sequential {
            e2e_vm_tests::run_sequentially(&filter_config, &run_config)
                .instrument(tracing::trace_span!("E2E"))
                .await?;
        } else {
            e2e_vm_tests::run_in_parallel(&filter_config, &run_config)
                .instrument(tracing::trace_span!("E2E"))
                .await?;
        }
    }

    // Run IR tests
    if run_kind_config.kind.ir && !filter_config.first_only {
        println!("\n");
        ir_generation::run(filter_config.include.as_ref(), cli.verbose, &run_config)
            .instrument(tracing::trace_span!("IR"))
            .await?;
    }

    // Run snapshot tests
    if run_kind_config.kind.snapshot && !filter_config.first_only {
        println!("\n");
        snapshot::run(filter_config.include.as_ref())
            .instrument(tracing::trace_span!("SNAPSHOT"))
            .await?;
    }

    Ok(())
}
