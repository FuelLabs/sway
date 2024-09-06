//! This module creates copies of chain configs on the disk so that `forc-node`
//! will always have a pinned instance of a chain config for given `Mode`.
use forc_util::user_forc_directory;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;

static CHAINCONFIG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/chain_configs");

const CONFIG_FOLDER: &str = "chainspecs";
const LOCAL: &str = "local";
const TESTNET: &str = "testnet";

pub enum ChainConfig {
    Local,
    Testnet,
}

impl From<ChainConfig> for PathBuf {
    fn from(value: ChainConfig) -> Self {
        let user_forc_dir = user_forc_directory().join(CONFIG_FOLDER);

        match value {
            ChainConfig::Local => user_forc_dir.join(LOCAL),
            ChainConfig::Testnet => user_forc_dir.join(TESTNET),
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
    }
    Ok(())
}
