//! Based on `rustfmt`, `swayfmt` aims to be a transparent approach to formatting Sway code.
//!
//! `swayfmt` configurations can be adjusted with a `swayfmt.toml` config file declared at the root of a Sway project,
//! however the defualt formatter does not require the presence of one and any fields omitted will remain as default.

#![allow(dead_code)]
pub mod config;
mod constants;
mod error;
mod fmt;
mod items;
mod utils;

pub use crate::fmt::{Format, Formatter};
pub use error::FormatterError;
