#![deny(missing_docs)]
//! This crate provides tooling for generating documentation and docstrings for Sway.

mod documentation;
mod documenter;
mod error;
pub use documentation::*;
pub use error::*;
