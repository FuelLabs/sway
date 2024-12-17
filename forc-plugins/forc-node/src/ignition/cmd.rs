use crate::{
    consts::{DEFAULT_PEERING_PORT, DEFAULT_PORT, MAINNET_BOOTSTRAP_NODE},
    util::DbConfig,
};
use clap::Parser;
use std::{net::IpAddr, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct IgnitionCmd {
    #[clap(long)]
    pub peer_id: Option<String>,
    #[clap(long)]
    pub secret: Option<String>,
    #[clap(long)]
    pub relayer: Option<String>,
    #[clap(long, default_value = "0.0.0.0")]
    pub ip: IpAddr,
    #[clap(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    #[clap(long, default_value_t = DEFAULT_PEERING_PORT)]
    pub peering_port: u16,
    #[clap(long, default_value = default_ignition_db_path().into_os_string())]
    pub db_path: PathBuf,
    #[clap(long, default_value_t = MAINNET_BOOTSTRAP_NODE.to_string())]
    pub bootstrap_node: String,
}

fn default_ignition_db_path() -> PathBuf {
    DbConfig::Ignition.into()
}
