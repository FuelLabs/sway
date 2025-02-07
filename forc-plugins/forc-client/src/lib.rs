pub mod cmd;
pub mod constants;
pub mod op;
pub mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};
use util::target::Target;

/// Flags for specifying the node to target.
#[derive(Debug, Default, Clone, Parser, Deserialize, Serialize)]
pub struct NodeTarget {
    /// The URL of the Fuel node to which we're submitting the transaction.
    /// If unspecified, checks the manifest's `network` table, then falls back
    /// to `http://127.0.0.1:4000`
    ///
    /// You can also use `--target`, `--devnet`, `--testnet`, or `--mainnet` to specify the Fuel node.
    #[clap(long, env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,

    /// Preset configurations for using a specific target.
    ///
    /// You can also use `--node-url`, `--devnet`, `--testnet`, or `--mainnet` to specify the Fuel node.
    ///
    /// Possible values are: [local, testnet, mainnet]
    #[clap(long)]
    pub target: Option<Target>,

    /// Use preset configuration for mainnet.
    ///
    /// You can also use `--node-url`, `--target`, or `--testnet` to specify the Fuel node.
    #[clap(long)]
    pub mainnet: bool,

    /// Use preset configuration for testnet.
    ///
    /// You can also use `--node-url`, `--target`, or `--mainnet` to specify the Fuel node.
    #[clap(long)]
    pub testnet: bool,

    /// Use preset configuration for devnet.
    ///
    /// You can also use `--node-url`, `--target`, or `--testnet` to specify the Fuel node.
    #[clap(long)]
    pub devnet: bool,
}

impl NodeTarget {
    /// Returns the URL for explorer
    pub fn get_explorer_url(&self) -> Option<String> {
        match (
            self.testnet,
            self.mainnet,
            self.devnet,
            self.target.clone(),
            self.node_url.clone(),
        ) {
            (true, false, _, None, None) => Target::testnet().explorer_url(),
            (false, true, _, None, None) => Target::mainnet().explorer_url(),
            (false, false, _, Some(target), None) => target.explorer_url(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_explorer_url_mainnet() {
        let node = NodeTarget {
            target: Some(Target::Mainnet),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        let actual = node.get_explorer_url().unwrap();
        assert_eq!("https://app.fuel.network", actual);
    }

    #[test]
    fn test_get_explorer_url_testnet() {
        let node = NodeTarget {
            target: Some(Target::Testnet),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        let actual = node.get_explorer_url().unwrap();
        assert_eq!("https://app-testnet.fuel.network", actual);
    }

    #[test]
    fn test_get_explorer_url_devnet() {
        let node = NodeTarget {
            target: Some(Target::Devnet),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: true,
        };
        let actual = node.get_explorer_url();
        assert_eq!(None, actual);
    }

    #[test]
    fn test_get_explorer_url_local() {
        let node = NodeTarget {
            target: Some(Target::Local),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        let actual = node.get_explorer_url();
        assert_eq!(None, actual);
    }
}
