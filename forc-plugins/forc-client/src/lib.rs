pub mod cmd;
pub mod constants;
pub mod op;
pub mod util;

use clap::Parser;
use forc_pkg::manifest::Network;
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
    /// Returns the URL to use for connecting to Fuel Core node.
    pub fn get_node_url(&self, manifest_network: &Option<Network>) -> anyhow::Result<String> {
        let options_count = [
            self.mainnet,
            self.testnet,
            self.devnet,
            self.target.is_some(),
            self.node_url.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        // ensure at most one option is specified
        if options_count > 1 {
            anyhow::bail!("Only one of `--mainnet`, `--testnet`, `--devnet`, `--target`, or `--node-url` should be specified");
        }

        let node_url = match () {
            _ if self.mainnet => Target::mainnet().target_url(),
            _ if self.testnet => Target::testnet().target_url(),
            _ if self.devnet => Target::devnet().target_url(),
            _ if self.target.is_some() => self.target.as_ref().unwrap().target_url(),
            _ if self.node_url.is_some() => self.node_url.as_ref().unwrap().clone(),
            _ => manifest_network
                .as_ref()
                .map(|nw| &nw.url[..])
                .unwrap_or(crate::constants::NODE_URL)
                .to_string(),
        };

        Ok(node_url)
    }

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

    #[test]
    fn test_get_node_url_testnet() {
        let node = NodeTarget {
            target: None,
            node_url: None,
            mainnet: false,
            testnet: true,
            devnet: false,
        };

        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("https://testnet.fuel.network", actual);
    }

    #[test]
    fn test_get_node_url_mainnet() {
        let node = NodeTarget {
            target: None,
            node_url: None,
            mainnet: true,
            testnet: false,
            devnet: false,
        };

        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("https://mainnet.fuel.network", actual);
    }

    #[test]
    fn test_get_node_url_target_mainnet() {
        let node = NodeTarget {
            target: Some(Target::Mainnet),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("https://mainnet.fuel.network", actual);
    }

    #[test]
    fn test_get_node_url_target_testnet() {
        let node = NodeTarget {
            target: Some(Target::Testnet),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };

        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("https://testnet.fuel.network", actual);
    }

    #[test]
    fn test_get_node_url_default() {
        let node = NodeTarget {
            target: None,
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };

        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("http://127.0.0.1:4000", actual);
    }

    #[test]
    fn test_get_node_url_local() {
        let node = NodeTarget {
            target: Some(Target::Local),
            node_url: None,
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        let actual = node.get_node_url(&None).unwrap();
        assert_eq!("http://127.0.0.1:4000", actual);
    }

    #[test]
    #[should_panic(
        expected = "Only one of `--mainnet`, `--testnet`, `--devnet`, `--target`, or `--node-url` should be specified"
    )]
    fn test_get_node_url_local_testnet() {
        let node = NodeTarget {
            target: Some(Target::Local),
            node_url: None,
            mainnet: false,
            testnet: true,
            devnet: false,
        };
        node.get_node_url(&None).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Only one of `--mainnet`, `--testnet`, `--devnet`, `--target`, or `--node-url` should be specified"
    )]
    fn test_get_node_url_same_url() {
        let node = NodeTarget {
            target: Some(Target::Testnet),
            node_url: Some("testnet.fuel.network".to_string()),
            mainnet: false,
            testnet: false,
            devnet: false,
        };
        node.get_node_url(&None).unwrap();
    }
}
