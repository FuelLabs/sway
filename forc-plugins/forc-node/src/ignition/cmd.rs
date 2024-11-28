use crate::pkg::DbConfig;
use clap::Parser;
use std::{net::IpAddr, path::PathBuf};

// TODO: add boostrap node.
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
    #[clap(long = "port", default_value_t = 4000)]
    pub port: u16,
    #[clap(long = "peering-port", default_value_t = 30333)]
    pub peering_port: u16,
    #[clap(long = "db-path", default_value = default_ignition_db_path().into_os_string())]
    pub db_path: PathBuf,
}

fn default_ignition_db_path() -> PathBuf {
    DbConfig::Ignition.into()
}
