use crate::constants::{
    DEVNET_ENDPOINT_URL, DEVNET_FAUCET_URL, MAINNET_ENDPOINT_URL, MAINNET_EXPLORER_URL, NODE_URL,
    TESTNET_ENDPOINT_URL, TESTNET_EXPLORER_URL, TESTNET_FAUCET_URL,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// Possible target values that forc-client can interact with.
pub enum Target {
    Mainnet,
    Testnet,
    Devnet,
    Local,
}

impl Default for Target {
    fn default() -> Self {
        Self::Local
    }
}

impl Target {
    pub fn target_url(&self) -> String {
        let url = match self {
            Target::Mainnet => MAINNET_ENDPOINT_URL,
            Target::Testnet => TESTNET_ENDPOINT_URL,
            Target::Devnet => DEVNET_ENDPOINT_URL,
            Target::Local => NODE_URL,
        };
        url.to_string()
    }

    pub fn from_target_url(target_url: &str) -> Option<Self> {
        match target_url {
            TESTNET_ENDPOINT_URL => Some(Target::Testnet),
            MAINNET_ENDPOINT_URL => Some(Target::Mainnet),
            DEVNET_ENDPOINT_URL => Some(Target::Devnet),
            NODE_URL => Some(Target::Local),
            _ => None,
        }
    }

    pub fn local() -> Self {
        Target::Local
    }

    pub fn devnet() -> Self {
        Target::Devnet
    }

    pub fn testnet() -> Self {
        Target::Testnet
    }

    pub fn mainnet() -> Self {
        Target::Mainnet
    }

    pub fn faucet_url(&self) -> Option<String> {
        match self {
            Target::Mainnet => None,
            Target::Testnet => Some(TESTNET_FAUCET_URL.to_string()),
            Target::Devnet => Some(DEVNET_FAUCET_URL.to_string()),
            Target::Local => Some("http://localhost:3000".to_string()),
        }
    }

    pub fn explorer_url(&self) -> Option<String> {
        match self {
            Target::Mainnet => Some(MAINNET_EXPLORER_URL.to_string()),
            Target::Testnet => Some(TESTNET_EXPLORER_URL.to_string()),
            Target::Devnet => None,
            _ => None,
        }
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fuel Sepolia Testnet" => Ok(Target::Testnet),
            "Ignition" => Ok(Target::Mainnet),
            "local" => Ok(Target::Local),
            "Devnet" | "devnet" => Ok(Target::Devnet),
            _ => bail!(
                "'{s}' is not a valid target name. Possible values: '{}', '{}', '{}', '{}'",
                Target::Testnet,
                Target::Mainnet,
                Target::Local,
                Target::Devnet,
            ),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Target::Mainnet => "Ignition",
            Target::Testnet => "Fuel Sepolia Testnet",
            Target::Devnet => "Devnet",
            Target::Local => "local",
        };
        write!(f, "{s}")
    }
}
