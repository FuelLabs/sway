use crate::consts::DEFAULT_PORT;
use anyhow;
use clap::Parser;
use fuel_core::{chain_config::default_consensus_dev_key, service::Config};
use fuel_core_chain_config::{
    coin_config_helpers::CoinConfigGenerator, ChainConfig, CoinConfig, SnapshotMetadata,
    TESTNET_INITIAL_BALANCE,
};
use fuel_core_types::{
    fuel_crypto::fuel_types::{Address, AssetId},
    secrecy::Secret,
    signer::SignMode,
};
use std::{path::PathBuf, str::FromStr};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_coins_per_account_single_account_with_defaults() {
        let base_asset_id = AssetId::default();
        let account_id = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let accounts = vec![account_id.to_string()];

        let result = get_coins_per_account(accounts, &base_asset_id, 0);
        assert!(result.is_ok());

        let coins = result.unwrap();
        assert_eq!(coins.len(), 1);

        let coin = &coins[0];
        assert_eq!(coin.owner, Address::from_str(account_id).unwrap());
        assert_eq!(coin.asset_id, base_asset_id);
        assert_eq!(coin.amount, TESTNET_INITIAL_BALANCE);
        assert_eq!(coin.output_index, 0);
    }

    #[test]
    fn test_get_coins_per_account_with_custom_asset() {
        let base_asset_id = AssetId::default();
        let account_id = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let asset_id = "0x0000000000000000000000000000000000000000000000000000000000000002";
        let accounts = vec![format!("{}:{}", account_id, asset_id)];

        let result = get_coins_per_account(accounts, &base_asset_id, 0);
        assert!(result.is_ok());

        let coins = result.unwrap();
        assert_eq!(coins.len(), 1);

        let coin = &coins[0];
        assert_eq!(coin.owner, Address::from_str(account_id).unwrap());
        assert_eq!(coin.asset_id, AssetId::from_str(asset_id).unwrap());
        assert_eq!(coin.amount, TESTNET_INITIAL_BALANCE);
        assert_eq!(coin.output_index, 0);
    }

    #[test]
    fn test_get_coins_per_account_with_custom_amount() {
        let base_asset_id = AssetId::default();
        let account_id = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let asset_id = "0x0000000000000000000000000000000000000000000000000000000000000002";
        let amount = 5000000u64;
        let accounts = vec![format!("{}:{}:{}", account_id, asset_id, amount)];

        let result = get_coins_per_account(accounts, &base_asset_id, 0);
        assert!(result.is_ok());

        let coins = result.unwrap();
        assert_eq!(coins.len(), 1);

        let coin = &coins[0];
        assert_eq!(coin.owner, Address::from_str(account_id).unwrap());
        assert_eq!(coin.asset_id, AssetId::from_str(asset_id).unwrap());
        assert_eq!(coin.amount, amount);
        assert_eq!(coin.output_index, 0);
    }

    #[test]
    fn test_get_coins_per_account_multiple_accounts() {
        let base_asset_id = AssetId::default();
        let account1 = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let account2 = "0x0000000000000000000000000000000000000000000000000000000000000002";
        let accounts = vec![account1.to_string(), account2.to_string()];

        let result = get_coins_per_account(accounts, &base_asset_id, 5);
        assert!(result.is_ok());

        let coins = result.unwrap();
        assert_eq!(coins.len(), 2);

        let coin1 = &coins[0];
        assert_eq!(coin1.owner, Address::from_str(account1).unwrap());
        assert_eq!(coin1.output_index, 5);

        let coin2 = &coins[1];
        assert_eq!(coin2.owner, Address::from_str(account2).unwrap());
        assert_eq!(coin2.output_index, 6);
    }

    #[test]
    fn test_get_coins_per_account_edge_cases_and_errors() {
        let base_asset_id = AssetId::default();
        let valid_account = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let valid_asset = "0x0000000000000000000000000000000000000000000000000000000000000002";

        // Test empty input
        let result = get_coins_per_account(vec![], &base_asset_id, 0);
        assert!(result.is_ok());
        let coins = result.unwrap();
        assert_eq!(coins.len(), 0);

        // Test invalid account ID
        let result =
            get_coins_per_account(vec!["invalid_account_id".to_string()], &base_asset_id, 0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid account ID: Invalid encoded byte in Address"
        );

        // Test invalid asset ID
        let result = get_coins_per_account(
            vec![format!("{}:invalid_asset", valid_account)],
            &base_asset_id,
            0,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid asset ID: Invalid encoded byte in AssetId"
        );

        // Test invalid amount
        let result = get_coins_per_account(
            vec![format!("{}:{}:not_a_number", valid_account, valid_asset)],
            &base_asset_id,
            0,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid amount: invalid digit found in string"
        );

        // Test too many parts
        let result = get_coins_per_account(
            vec!["part1:part2:part3:part4".to_string()],
            &base_asset_id,
            0,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid account format: part1:part2:part3:part4. Expected format: <account-id>[:asset-id[:amount]]"
        );

        // Test empty account (should fail now)
        let result = get_coins_per_account(vec!["".to_string()], &base_asset_id, 0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid account ID: Invalid encoded byte in Address"
        );
    }
}
