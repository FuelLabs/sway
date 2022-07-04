pub mod cli;
mod ops;
pub mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::cli::{BuildCommand, DeployCommand, JsonAbiCommand, RunCommand};
    pub use crate::ops::{forc_abi_json, forc_build, forc_check, forc_deploy, forc_run};
}

#[cfg(feature = "util")]
pub mod util {
    pub use sway_utils::constants;
}
