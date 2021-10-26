#![allow(dead_code)]
mod cli;
mod ops;
mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::cli::{BuildCommand, DeployCommand, RunCommand, AbiSpecCommand};
    pub use crate::ops::{forc_build, forc_deploy, forc_run, forc_abi_spec};
}

#[cfg(feature = "util")]
pub mod util {
    pub use crate::utils::helpers;
}
