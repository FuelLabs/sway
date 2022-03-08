#![allow(dead_code)]
mod cli;
mod lock;
mod ops;
mod pkg;
mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::cli::{BuildCommand, DeployCommand, JsonAbiCommand, RunCommand};
    pub use crate::ops::{forc_abi_json, forc_build, forc_deploy, forc_run};
}

#[cfg(feature = "util")]
pub mod util {
    pub use crate::utils::client::start_fuel_core;
    pub use crate::utils::helpers;
    pub use sway_utils::constants;
}
