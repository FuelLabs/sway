pub mod cmd;
pub mod constants;
pub mod op;
pub mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};
use util::target::Target;

/// Flags for specifying the node to target.
#[derive(Debug, Default, Parser, Deserialize, Serialize)]
pub struct NodeTarget {
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to `http://127.0.0.1:4000`
    ///
    /// You can also use `--target` or `--testnet` to specify the Fuel node.
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,
    /// Use preset configurations for deploying to a specific target.
    ///
    /// You can also use `--node-url` or `--testnet` to specify the Fuel node.
    ///
    /// Possible values are: [beta-1, beta-2, beta-3, beta-4, local]
    #[clap(long)]
    pub target: Option<Target>,
    /// Use preset configuration for the latest testnet.
    ///
    /// You can also use `--node-url` or `--target` to specify the Fuel node.
    #[clap(long)]
    pub testnet: bool,
}
