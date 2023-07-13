use std::str::FromStr;

pub(crate) mod encode;
pub(crate) mod pkg;
pub(crate) mod tx;

use crate::default::{BETA_2_ENDPOINT_URL, BETA_3_ENDPOINT_URL, NODE_URL};

#[derive(Debug, Clone)]
/// Possible target values that forc-client can interact with.
pub enum Target {
    Beta2,
    Beta3,
    LATEST,
}

impl Default for Target {
    fn default() -> Self {
        Self::LATEST
    }
}

impl Target {
    pub fn target_url(&self) -> &str {
        match self {
            Target::Beta2 => BETA_2_ENDPOINT_URL,
            Target::Beta3 => BETA_3_ENDPOINT_URL,
            Target::LATEST => NODE_URL,
        }
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "latest" {
            Ok(Target::LATEST)
        } else if s == "beta-2" {
            Ok(Target::Beta2)
        } else if s == "beta-3" {
            Ok(Target::Beta3)
        } else {
            anyhow::bail!(
                "invalid testnet name provided. Possible values are 'beta-2', 'beta-3', 'latest'."
            )
        }
    }
}
