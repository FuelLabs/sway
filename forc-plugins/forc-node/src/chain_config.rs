//! This module creates copies of chain configs on the disk so that `forc-node`
//! will always have a pinned instance of a chain config for given `Mode`.
use forc_util::user_forc_directory;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;

static CHAINCONFIG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/chain_configs");

const CONFIG_FOLDER: &str = "chainspecs";
const LOCAL: &str = "local";
const TESTNET: &str = "testnet";
const IGNITION: &str = "ignition";
const DB_FOLDER: &str = "db";

pub enum ChainConfig {
    Local,
    Testnet,
    Ignition,
}

impl From<ChainConfig> for PathBuf {
    fn from(value: ChainConfig) -> Self {
        let user_forc_dir = user_forc_directory().join(CONFIG_FOLDER);

        match value {
            ChainConfig::Local => user_forc_dir.join(LOCAL),
            ChainConfig::Testnet => user_forc_dir.join(TESTNET),
            ChainConfig::Ignition => user_forc_dir.join(IGNITION),
        }
    }
}

pub fn create_chainconfig_dir(chain_config: ChainConfig) -> anyhow::Result<()> {
    let user_forc_dir = user_forc_directory().join(CONFIG_FOLDER);
    match chain_config {
        ChainConfig::Local => {
            let local = CHAINCONFIG_DIR
                .get_dir(LOCAL)
                .ok_or_else(|| anyhow::anyhow!("failed to locate local-testnet"))?;
            std::fs::create_dir_all(user_forc_dir.join(LOCAL))?;
            local.extract(user_forc_dir)?;
        }
        ChainConfig::Testnet => {
            let local = CHAINCONFIG_DIR
                .get_dir(TESTNET)
                .ok_or_else(|| anyhow::anyhow!("failed to locate testnet"))?;
            std::fs::create_dir_all(user_forc_dir.join(TESTNET))?;
            local.extract(user_forc_dir)?;
        }
        ChainConfig::Ignition => {
            let ignition = CHAINCONFIG_DIR
                .get_dir(IGNITION)
                .ok_or_else(|| anyhow::anyhow!("failed to locate ignition"))?;
            std::fs::create_dir_all(user_forc_dir.join(IGNITION))?;
            ignition.extract(user_forc_dir)?;
        }
    }
    Ok(())
}

pub enum DbConfig {
    Local,
    Testnet,
    Ignition,
}

impl From<DbConfig> for PathBuf {
    fn from(value: DbConfig) -> Self {
        let user_forc_dir = user_forc_directory().join(DB_FOLDER);
        match value {
            DbConfig::Local => user_forc_dir.join(LOCAL),
            DbConfig::Testnet => user_forc_dir.join(TESTNET),
            DbConfig::Ignition => user_forc_dir.join(IGNITION),
        }
    }
}
