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

impl From<LocalCmd> for Config {
    fn from(cmd: LocalCmd) -> Self {
        let mut config = Config::local_node();

        config.name = "fuel-core".to_string();

        // Handle chain config/snapshot
        let snapshot_path = cmd
            .chain_config
            .unwrap_or_else(|| ChainConfig::Local.into());
        if snapshot_path.exists() {
            if let Ok(metadata) = SnapshotMetadata::read(&snapshot_path) {
                if let Ok(reader) = SnapshotReader::open(metadata) {
                    config.snapshot_reader = reader;
                }
            }
        }

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
