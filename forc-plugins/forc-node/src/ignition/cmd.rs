use crate::{
    chain_config::DbConfig,
    consts::{DEFAULT_PEERING_PORT, DEFAULT_PORT, MAINNET_BOOTSTRAP_NODE},
};
use clap::Parser;
use std::{net::IpAddr, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct IgnitionCmd {
    #[clap(long = "peer-id")]
    pub peer_id: Option<String>,
    #[clap(long = "secret")]
    pub secret: Option<String>,
    #[clap(long = "relayer")]
    pub relayer: Option<String>,
    #[clap(long = "ip", default_value = "0.0.0.0")]
    pub ip: IpAddr,
    #[clap(long = "port", default_value_t = DEFAULT_PORT)]
    pub port: u16,
    #[clap(long = "peering-port", default_value_t = DEFAULT_PEERING_PORT)]
    pub peering_port: u16,
    #[clap(long = "db-path", default_value = default_ignition_db_path().into_os_string())]
    pub db_path: PathBuf,
    #[clap(long = "bootstrap-node", default_value_t = MAINNET_BOOTSTRAP_NODE.to_string())]
    pub bootstrap_node: String,
}

fn default_ignition_db_path() -> PathBuf {
    DbConfig::Ignition.into()
}
