mod e2e_vm_tests;
mod ir_generation;
mod test_consistency;

use anyhow::Result;
use clap::Parser;
use forc_tracing::init_tracing_subscriber;
use std::str::FromStr;
use sway_core::{BuildTarget, ExperimentalFlags};
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
    #[arg(long, short, value_name = "REGEX")]
    skip_until: Option<regex::Regex>,

    /// Only run tests with ABI JSON output validation
    #[arg(long, visible_alias = "abi")]
    abi_only: bool,

    /// Only run tests that deploy contracts
    #[arg(long, visible_alias = "contract")]
    contract_only: bool,

    /// Only run the first test
    #[arg(long, visible_alias = "first")]
    first_only: bool,

    /// Print out warnings and errors
    #[arg(long, env = "SWAY_TEST_VERBOSE")]
    verbose: bool,

    /// Compile sway code in release mode
    #[arg(long)]
    release: bool,

    /// Intended for use in `CI` to ensure test lock files are up to date
    #[arg(long)]
    locked: bool,

    /// Build target.
    #[arg(long, visible_alias = "target")]
    build_target: Option<String>,

    /// Experimental flag for new encoding
    #[arg(long)]
    experimental_new_encoding: bool,

    /// Update all output files
    #[arg(long)]
    update_output_files: bool,
}

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub include: Option<regex::Regex>,
    pub exclude: Option<regex::Regex>,
    pub skip_until: Option<regex::Regex>,
    pub abi_only: bool,
    pub contract_only: bool,
    pub first_only: bool,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub build_target: BuildTarget,
    pub locked: bool,
    pub verbose: bool,
    pub release: bool,
    pub experimental: ExperimentalFlags,
    pub update_output_files: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_subscriber(Default::default());

    // Parse args
    let cli = Cli::parse();
    let filter_config = FilterConfig {
        include: cli.include,
        exclude: cli.exclude,
        skip_until: cli.skip_until,
        abi_only: cli.abi_only,
        contract_only: cli.contract_only,
        first_only: cli.first_only,
    };
    let build_target = match cli.build_target {
        Some(target) => match BuildTarget::from_str(target.as_str()) {
            Ok(target) => target,
            _ => panic!("unexpected build target"),
        },
        None => BuildTarget::default(),
    };
    let run_config = RunConfig {
        locked: cli.locked,
        verbose: cli.verbose,
        release: cli.release,
        build_target,
        experimental: sway_core::ExperimentalFlags {
            new_encoding: cli.experimental_new_encoding,
        },
        update_output_files: cli.update_output_files,
    };

    // Check that the tests are consistent
    test_consistency::check()?;

    // Run E2E tests
    e2e_vm_tests::run(&filter_config, &run_config)
        .instrument(tracing::trace_span!("E2E"))
        .await?;

    // Run IR tests
    if !filter_config.first_only {
        println!("\n");
        ir_generation::run(
            filter_config.include.as_ref(),
            cli.verbose,
            sway_ir::ExperimentalFlags {
                new_encoding: run_config.experimental.new_encoding,
            },
        )
        .instrument(tracing::trace_span!("IR"))
        .await?;
    }

    Ok(())
}
