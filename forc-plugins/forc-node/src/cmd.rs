use std::net::IpAddr;

use crate::{
    consts::{DEFAULT_PEERING_PORT, DEFAULT_PORT},
    ignition::cmd::IgnitionCmd,
    local::cmd::LocalCmd,
    testnet::cmd::TestnetCmd,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "forc node", version)]
/// Forc node is a wrapper around fuel-core with sensible defaults to provide
/// easy way of bootstrapping a node for local development, testnet or mainnet.
pub struct ForcNodeCmd {
    /// Print the fuel-core command without running it.
    #[arg(long)]
    pub dry_run: bool,
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Starts a local node for development purposes.
    Local(LocalCmd),
    /// Starts a node that will connect to latest testnet.
    Testnet(TestnetCmd),
    /// Starts a node that will connect to ignition network.
    Ignition(IgnitionCmd),
}

/// Set of shared node settings, specifically related to connections.
#[derive(Parser, Debug, Clone)]
pub struct ConnectionSettings {
    #[clap(long)]
    pub peer_id: Option<String>,
    #[clap(long)]
    pub secret: Option<String>,
    #[clap(long)]
    pub relayer: Option<String>,
    #[clap(long, default_value = "0.0.0.0")]
    pub ip: IpAddr,
    #[clap(long, default_value_t = DEFAULT_PORT, value_parser = clap::value_parser!(u16).range(1..=65535))]
    pub port: u16,
    #[clap(long, default_value_t = DEFAULT_PEERING_PORT, value_parser = clap::value_parser!(u16).range(1..=65535))]
    pub peering_port: u16,
}
