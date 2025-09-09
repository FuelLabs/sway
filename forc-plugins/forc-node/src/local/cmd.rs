use crate::chain_config::ChainConfig;
use crate::consts::DEFAULT_PORT;
use clap::Parser;
use fuel_core::{chain_config::default_consensus_dev_key, service::Config};
use fuel_core_chain_config::{SnapshotMetadata, SnapshotReader};
use fuel_core_types::{secrecy::Secret, signer::SignMode};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct LocalCmd {
    #[clap(long)]
    pub chain_config: Option<PathBuf>,
    #[clap(long)]
    pub port: Option<u16>,
    #[clap(long)]
    /// If a db path is provided local node runs in persistent mode.
    pub db_path: Option<PathBuf>,
    #[clap(long)]
    /// Fund accounts with the format: <account-id>:<asset-id>:<amount>
    /// Multiple accounts can be provided via comma separation or multiple --account flags
    pub account: Vec<String>,
}

fn get_coins_per_account(
    account_strings: Vec<String>,
    base_asset_id: &AssetId,
    current_coin_idx: usize,
) -> anyhow::Result<Vec<CoinConfig>> {
    let mut coin_generator = CoinConfigGenerator::new();
    let mut coins = Vec::new();

    for account_string in account_strings {
        let parts: Vec<&str> = account_string.trim().split(':').collect();
        let (owner, asset_id, amount) = match parts.as_slice() {
            [owner_str] => {
                // Only account-id provided, use default asset and amount
                let owner = Address::from_str(owner_str)
                    .map_err(|e| anyhow::anyhow!("Invalid account ID: {}", e))?;
                (owner, *base_asset_id, TESTNET_INITIAL_BALANCE)
            }
            [owner_str, asset_str] => {
                // account-id:asset-id provided, use default amount
                let owner = Address::from_str(owner_str)
                    .map_err(|e| anyhow::anyhow!("Invalid account ID: {}", e))?;
                let asset_id = AssetId::from_str(asset_str)
                    .map_err(|e| anyhow::anyhow!("Invalid asset ID: {}", e))?;
                (owner, asset_id, TESTNET_INITIAL_BALANCE)
            }
            [owner_str, asset_str, amount_str] => {
                // Full format: account-id:asset-id:amount
                let owner = Address::from_str(owner_str)
                    .map_err(|e| anyhow::anyhow!("Invalid account ID: {}", e))?;
                let asset_id = AssetId::from_str(asset_str)
                    .map_err(|e| anyhow::anyhow!("Invalid asset ID: {}", e))?;
                let amount = amount_str
                    .parse::<u64>()
                    .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;
                (owner, asset_id, amount)
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid account format: {}. Expected format: <account-id>[:asset-id[:amount]]",
                    account_string
                ));
            }
        };
        let coin = CoinConfig {
            amount,
            owner,
            asset_id,
            output_index: (current_coin_idx + coins.len()) as u16,
            ..coin_generator.generate()
        };
        coins.push(coin);
    }
    Ok(coins)
}

impl From<LocalCmd> for Config {
    fn from(cmd: LocalCmd) -> Self {
        let snapshot_path = cmd
            .chain_config
            .unwrap_or_else(|| crate::chain_config::ChainConfig::Local.into());
        let chain_config = match SnapshotMetadata::read(&snapshot_path) {
            Ok(metadata) => ChainConfig::from_snapshot_metadata(&metadata).unwrap(),
            Err(e) => {
                tracing::error!("Failed to open snapshot reader: {}", e);
                tracing::warn!("Using local testnet snapshot reader");
                ChainConfig::local_testnet()
            }
        };
        let base_asset_id = chain_config.consensus_parameters.base_asset_id();

        // Parse and validate account funding if provided
        let mut state_config = fuel_core_chain_config::StateConfig::local_testnet();
        state_config
            .coins
            .iter_mut()
            .for_each(|coin| coin.asset_id = *base_asset_id);

        let current_coin_idx = state_config.coins.len();
        if !cmd.account.is_empty() {
            let coins = get_coins_per_account(cmd.account, base_asset_id, current_coin_idx)
                .map_err(|e| anyhow::anyhow!("Error parsing account funding: {}", e))
                .unwrap();
            if !coins.is_empty() {
                tracing::info!("Additional accounts");
                for coin in &coins {
                    tracing::info!(
                        "Address({:#x}), Asset ID({:#x}), Balance({})",
                        coin.owner,
                        coin.asset_id,
                        coin.amount
                    );
                }
                state_config.coins.extend(coins);
            }
        }

        let mut config = Config::local_node_with_configs(chain_config, state_config);
        config.name = "fuel-core".to_string();

        // Local-specific settings
        config.debug = true;
        let key = default_consensus_dev_key();
        config.consensus_signer = SignMode::Key(Secret::new(key.into()));

        // Database configuration
        if let Some(db_path) = cmd.db_path {
            config.combined_db_config.database_type = fuel_core::service::config::DbType::RocksDb;
            config.combined_db_config.database_path = db_path;
        } else {
            config.combined_db_config.database_type = fuel_core::service::config::DbType::InMemory;
            config.historical_execution = false;
        }

        // Network configuration
        let ip = "127.0.0.1".parse().unwrap();
        let port = cmd.port.unwrap_or(DEFAULT_PORT);
        config.graphql_config.addr = std::net::SocketAddr::new(ip, port);

        config.utxo_validation = false; // local development

        config
    }
}
