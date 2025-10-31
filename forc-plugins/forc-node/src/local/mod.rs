pub mod cmd;
mod fork;

use crate::{
    chain_config::{check_and_update_chain_config, ChainConfig},
    util::HumanReadableConfig,
};
use forc_tracing::println_green;
use fork::{ForkClient, ForkSettings, ForkingOnChainStorage};
use fuel_core::{
    combined_database::CombinedDatabase,
    database::{database_description::on_chain::OnChain, Database, RegularStage},
    service::FuelService,
    state::data_source::{DataSource, DataSourceType},
};
use fuel_core_types::fuel_types::BlockHeight;
use std::sync::Arc;

/// Local is a local node suited for local development.
/// By default, the node is in `debug` mode and the db used is `in-memory`.
/// Returns `None` if this is a dry_run and no service created for fuel-core.
pub async fn run(cmd: cmd::LocalCmd, dry_run: bool) -> anyhow::Result<Option<FuelService>> {
    check_and_update_chain_config(ChainConfig::Local).await?;

    let fork_url = cmd.fork_url.clone();
    let fork_block_number = cmd.fork_block_number;
    let config = fuel_core::service::Config::from(cmd);

    let fork_settings = fork_url.map(|url| {
        let block_height = fork_block_number.map(BlockHeight::new);
        ForkSettings::new(url, block_height)
    });

    if dry_run {
        // For dry run, display the configuration that would be used
        println_green(&format!("{}", HumanReadableConfig::from(&config)));
        if let Some(fork) = fork_settings {
            println_green(&format!(
                "State fork enabled from {} at height {:?}",
                fork.fork_url, fork.fork_block_height
            ));
        }
        return Ok(None);
    }
    println_green("Starting fuel-core service...");

    let service = match fork_settings {
        Some(fork_settings) => {
            let combined_database = CombinedDatabase::from_config(&config.combined_db_config)
                .map_err(|e| anyhow::anyhow!("Failed to start fuel-core service: {}", e))?;

            let fork_client = Arc::new(ForkClient::new(
                fork_settings.fork_url.clone(),
                fork_settings.fork_block_height,
            )?);

            let off_chain = combined_database.off_chain().clone();
            let relayer = combined_database.relayer().clone();
            let gas_price = combined_database.gas_price().clone();
            let compression = combined_database.compression().clone();
            let on_chain = {
                let on_chain = combined_database.on_chain().clone();
                let (_, metadata) = on_chain.clone().into_inner();
                let stage: RegularStage<OnChain> = Default::default();
                let forking_storage: DataSourceType<OnChain> =
                    Arc::new(ForkingOnChainStorage::new(on_chain, fork_client));
                let data_source = DataSource::new(forking_storage, stage);
                Database::from_storage_and_metadata(data_source, metadata)
            };
            let combined_database =
                CombinedDatabase::new(on_chain, off_chain, relayer, gas_price, compression);
            FuelService::from_combined_database(combined_database, config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start fuel-core service: {}", e))?
        }
        None => FuelService::new_node(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start fuel-core service: {}", e))?,
    };

    println_green(&format!("Service started on: {}", service.bound_address));
    Ok(Some(service))
}
