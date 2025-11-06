use crate::{cmd::ConnectionSettings, consts::MAINNET_BOOTSTRAP_NODE, util::DbConfig};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct IgnitionCmd {
    #[clap(flatten)]
    pub connection_settings: ConnectionSettings,
    #[clap(long, default_value = default_ignition_db_path().into_os_string())]
    pub db_path: PathBuf,
    #[clap(long, default_value_t = MAINNET_BOOTSTRAP_NODE.to_string())]
    pub bootstrap_node: String,

    /// Skip interactive prompts (intended for scripted/test environments).
    #[clap(long, hide = true)]
    pub non_interactive: bool,
}

fn default_ignition_db_path() -> PathBuf {
    DbConfig::Ignition.into()
}
