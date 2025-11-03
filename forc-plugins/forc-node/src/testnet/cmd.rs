use crate::{cmd::ConnectionSettings, consts::TESTNET_BOOTSTRAP_NODE, util::DbConfig};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct TestnetCmd {
    #[clap(flatten)]
    pub connection_settings: ConnectionSettings,
    #[clap(long, default_value = default_testnet_db_path().into_os_string())]
    pub db_path: PathBuf,
    #[clap(long, default_value_t = TESTNET_BOOTSTRAP_NODE.to_string())]
    pub bootstrap_node: String,

    /// Skip interactive prompts (intended for scripted/test environments).
    #[clap(long, hide = true)]
    pub non_interactive: bool,
}

fn default_testnet_db_path() -> PathBuf {
    DbConfig::Testnet.into()
}
