use crate::constants::{
    BETA_2_ENDPOINT_URL, BETA_2_FAUCET_URL, BETA_3_ENDPOINT_URL, BETA_3_FAUCET_URL,
    BETA_4_ENDPOINT_URL, BETA_4_FAUCET_URL, BETA_5_ENDPOINT_URL, BETA_5_FAUCET_URL,
    DEVNET_ENDPOINT_URL, DEVNET_FAUCET_URL, NODE_URL, TESTNET_ENDPOINT_URL, TESTNET_FAUCET_URL,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// Possible target values that forc-client can interact with.
pub enum Target {
    Beta2,
    Beta3,
    Beta4,
    Beta5,
    Devnet,
    Testnet,
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
            Target::Beta2 => BETA_2_ENDPOINT_URL,
            Target::Beta3 => BETA_3_ENDPOINT_URL,
            Target::Beta4 => BETA_4_ENDPOINT_URL,
            Target::Beta5 => BETA_5_ENDPOINT_URL,
            Target::Devnet => DEVNET_ENDPOINT_URL,
            Target::Testnet => TESTNET_ENDPOINT_URL,
            Target::Local => NODE_URL,
        };
        url.to_string()
    }

    pub fn from_target_url(target_url: &str) -> Option<Self> {
        match target_url {
            BETA_2_ENDPOINT_URL => Some(Target::Beta2),
            BETA_3_ENDPOINT_URL => Some(Target::Beta3),
            BETA_4_ENDPOINT_URL => Some(Target::Beta4),
            BETA_5_ENDPOINT_URL => Some(Target::Beta5),
            DEVNET_ENDPOINT_URL => Some(Target::Devnet),
            TESTNET_ENDPOINT_URL => Some(Target::Testnet),
            NODE_URL => Some(Target::Local),
            _ => None,
        }
    }

    pub fn testnet() -> Self {
        Target::Testnet
    }

    pub fn faucet_url(&self) -> String {
        match self {
            Target::Beta2 => BETA_2_FAUCET_URL.to_string(),
            Target::Beta3 => BETA_3_FAUCET_URL.to_string(),
            Target::Beta4 => BETA_4_FAUCET_URL.to_string(),
            Target::Beta5 => BETA_5_FAUCET_URL.to_string(),
            Target::Devnet => DEVNET_FAUCET_URL.to_string(),
            Target::Testnet => TESTNET_FAUCET_URL.to_string(),
            Target::Local => "http://localhost:3000".to_string(),
        }
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "beta-2" => Ok(Target::Beta2),
            "beta-3" => Ok(Target::Beta3),
            "beta-4" => Ok(Target::Beta4),
            "beta-5" => Ok(Target::Beta5),
            "devnet" => Ok(Target::Devnet),
            "testnet" => Ok(Target::Testnet),
            "local" => Ok(Target::Local),
            _ => bail!(
                "'{s}' is not a valid target name. Possible values: '{}', '{}', '{}', '{}', '{}', '{}', '{}'",
                Target::Beta2,
                Target::Beta3,
                Target::Beta4,
                Target::Beta5,
                Target::Devnet,
                Target::Testnet,
                Target::Local
            ),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Target::Beta2 => "beta-2",
            Target::Beta3 => "beta-3",
            Target::Beta4 => "beta-4",
            Target::Beta5 => "beta-5",
            Target::Devnet => "devnet",
            Target::Testnet => "testnet",
            Target::Local => "local",
        };
        write!(f, "{}", s)
    }
}
