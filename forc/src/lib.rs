#![allow(dead_code)]
mod cli;
mod ops;
mod utils;

#[cfg(feature = "test")]
pub mod test {
    pub use crate::cli::BuildCommand;
    pub use crate::ops::forc_build;
}

#[cfg(feature = "util")]
pub mod util {
    pub use crate::utils::helpers;
}
