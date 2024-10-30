use crate::constants::{
    MAINNET_ENDPOINT_URL, MAINNET_EXPLORER_URL, NODE_URL, TESTNET_ENDPOINT_URL,
    TESTNET_EXPLORER_URL, TESTNET_FAUCET_URL,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// Possible target values that forc-client can interact with.
pub enum Target {
    Testnet,
    Mainnet,
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
            Target::Testnet => TESTNET_ENDPOINT_URL,
            Target::Mainnet => MAINNET_ENDPOINT_URL,
            Target::Local => NODE_URL,
        };
        url.to_string()
    }

    pub fn from_target_url(target_url: &str) -> Option<Self> {
        match target_url {
            TESTNET_ENDPOINT_URL => Some(Target::Testnet),
            MAINNET_ENDPOINT_URL => Some(Target::Mainnet),
            NODE_URL => Some(Target::Local),
            _ => None,
        }
    }

    pub fn local() -> Self {
        Target::Local
    }

    pub fn testnet() -> Self {
        Target::Testnet
    }

    pub fn mainnet() -> Self {
        Target::Mainnet
    }

    pub fn faucet_url(&self) -> Option<String> {
        match self {
            Target::Testnet => Some(TESTNET_FAUCET_URL.to_string()),
            Target::Mainnet => None,
            Target::Local => Some("http://localhost:3000".to_string()),
        }
    }

    pub fn explorer_url(&self) -> Option<String> {
        match self {
            Target::Testnet => Some(TESTNET_EXPLORER_URL.to_string()),
            Target::Mainnet => Some(MAINNET_EXPLORER_URL.to_string()),
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
            _ => bail!(
                "'{s}' is not a valid target name. Possible values: '{}', '{}', '{}'",
                Target::Testnet,
                Target::Mainnet,
                Target::Local
            ),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Target::Testnet => "Fuel Sepolia Testnet",
            Target::Mainnet => "Ignition",
            Target::Local => "local",
        };
        write!(f, "{}", s)
    }
}
