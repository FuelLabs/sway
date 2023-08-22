pub mod cmd;
mod constants;
pub mod op;
mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};
use util::target::Target;

/// Flags for specifying the node to target.
#[derive(Debug, Default, Parser, Deserialize, Serialize)]
pub struct NodeTarget {
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to [`crate::default::NODE_URL`].
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Use preset configurations for deploying to a specific target.
    ///
    /// Possible values are: [beta-1, beta-2, beta-3, beta-4, local]
    #[clap(long)]
    pub target: Option<Target>,
    /// Use preset configuration for the latest testnet.
    #[clap(long)]
    pub testnet: bool,
}
