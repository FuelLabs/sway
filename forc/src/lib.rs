pub mod cli;
pub mod ops;
pub mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::{
        cli::BuildCommand,
        ops::{forc_build, forc_check},
    };
}

#[cfg(feature = "util")]
pub mod util {
    pub use sway_utils::constants;
}
