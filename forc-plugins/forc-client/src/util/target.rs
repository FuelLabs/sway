use crate::constants::{BETA_2_ENDPOINT_URL, BETA_3_ENDPOINT_URL, BETA_4_ENDPOINT_URL, NODE_URL};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// Possible target values that forc-client can interact with.
pub enum Target {
    Beta2,
    Beta3,
    Beta4,
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
            Target::Local => NODE_URL,
        };
        url.to_string()
    }

    pub fn from_target_url(target_url: &str) -> Option<Self> {
        match target_url {
            BETA_2_ENDPOINT_URL => Some(Target::Beta2),
            BETA_3_ENDPOINT_URL => Some(Target::Beta3),
            BETA_4_ENDPOINT_URL => Some(Target::Beta4),
            NODE_URL => Some(Target::Local),
            _ => None,
        }
    }

    pub fn is_testnet(&self) -> bool {
        match self {
            Target::Beta2 | Target::Beta3 | Target::Beta4 => true,
            Target::Local => false,
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
            "local" => Ok(Target::Local),
            _ => bail!(
                "'{s}' is not a valid target name. Possible values: '{}', '{}', '{}', '{}'",
                Target::Beta2,
                Target::Beta3,
                Target::Beta4,
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
            Target::Local => "local",
        };
        write!(f, "{}", s)
    }
}
