use crate::{ignition::cmd::IgnitionCmd, local::cmd::LocalCmd, testnet::cmd::TestnetCmd};
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
