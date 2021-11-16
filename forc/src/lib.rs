#![allow(dead_code)]
mod cli;
mod ops;
mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::cli::{BuildCommand, DeployCommand, RunCommand};
    pub use crate::ops::{forc_build, forc_deploy, forc_run};
}

#[cfg(feature = "util")]
pub mod util {
    pub use crate::utils::client::start_fuel_core;
    pub use crate::utils::constants;
    pub use crate::utils::helpers;
}
