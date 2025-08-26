//! A forc plugin to start a fuel core instance, preconfigured for generic
//! usecases.
use clap::Parser;
use forc_node::{
    cmd::{ForcNodeCmd, Mode},
    consts::{MINIMUM_OPEN_FILE_DESCRIPTOR_LIMIT, MIN_FUEL_CORE_VERSION},
    ignition, local, testnet,
    util::{check_open_fds_limit, get_fuel_core_version},
};
use forc_tracing::init_tracing_subscriber;
use forc_util::forc_result_bail;
use semver::Version;
use std::{env, process::Child, str::FromStr};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry, Layer};

/// Initialize logging with the same setup as fuel-core CLI
fn init_logging() {
    const LOG_FILTER: &str = "RUST_LOG";
    const HUMAN_LOGGING: &str = "HUMAN_LOGGING";

    let filter = match env::var_os(LOG_FILTER) {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    let human_logging = env::var_os(HUMAN_LOGGING)
        .map(|s| {
            bool::from_str(s.to_str().unwrap())
                .expect("Expected `true` or `false` to be provided for `HUMAN_LOGGING`")
        })
        .unwrap_or(true);

    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);

    let fmt = if human_logging {
        // use pretty logs
        layer
            .with_ansi(true)
            .with_level(true)
            .with_line_number(true)
            .boxed()
    } else {
        // use machine parseable structured logs
        layer
            // disable terminal colors
            .with_ansi(false)
            .with_level(true)
            .with_line_number(true)
            // use json
            .json()
            .boxed()
    };

    let subscriber = registry::Registry::default() // provide underlying span data store
        .with(filter) // filter out low-level debug tracing (eg tokio executor)
        .with(fmt); // log to stdout

    tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
}

/// Initialize common setup for testnet and ignition modes
fn init_cli_setup() -> anyhow::Result<()> {
    init_tracing_subscriber(Default::default());
    let current_version = get_fuel_core_version()?;
    let supported_min_version = Version::parse(MIN_FUEL_CORE_VERSION)?;
    if current_version < supported_min_version {
        forc_result_bail!(format!(
            "Minimum supported fuel core version is {MIN_FUEL_CORE_VERSION}, system version: {}",
            current_version
        ));
    }
    check_open_fds_limit(MINIMUM_OPEN_FILE_DESCRIPTOR_LIMIT)
        .map_err(|e| anyhow::anyhow!("Failed to check open file descriptor limit: {}", e))?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = ForcNodeCmd::parse();
    let mut handle: Option<Child> = match command.mode {
        Mode::Local(local) => {
            // Local uses embedded fuel-core
            init_logging();
            let service = local::run(local, command.dry_run).await?;
            if service.is_some() {
                // For local, we keep the service alive by waiting for ctrl-c
                tokio::signal::ctrl_c()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to listen for ctrl-c: {e}"))?;
            }
            return Ok(());
        }
        Mode::Testnet(testnet) => {
            init_cli_setup()?;
            testnet::op::run(testnet, command.dry_run).await?
        }
        Mode::Ignition(ignition) => {
            init_cli_setup()?;
            ignition::op::run(ignition, command.dry_run).await?
        }
    };

    // If not dry run, wait for the kill signal and kill fuel-core process
    if let Some(handle) = &mut handle {
        // Wait for the kill signal, if that comes we should kill child fuel-core
        // process.
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to listen for ctrl-c: {e}"))?;

        handle.kill()?;
    }

    Ok(())
}
