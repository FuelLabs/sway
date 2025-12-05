pub mod cmd;

use crate::{
    chain_config::{check_and_update_chain_config, ChainConfig},
    util::HumanReadableConfig,
};
use forc_diagnostic::println_green;
use fuel_core::service::FuelService;

/// Local is a local node suited for local development.
/// By default, the node is in `debug` mode and the db used is `in-memory`.
/// Returns `None` if this is a dry_run and no service created for fuel-core.
pub async fn run(cmd: cmd::LocalCmd, dry_run: bool) -> anyhow::Result<Option<FuelService>> {
    check_and_update_chain_config(ChainConfig::Local).await?;

    let config = fuel_core::service::Config::from(cmd);

    if dry_run {
        // For dry run, display the configuration that would be used
        println_green(&format!("{}", HumanReadableConfig::from(&config)));
        return Ok(None);
    }
    println_green("Starting fuel-core service...");
    let service = FuelService::new_node(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start fuel-core service: {}", e))?;

    println_green(&format!("Service started on: {}", service.bound_address));
    Ok(Some(service))
}
