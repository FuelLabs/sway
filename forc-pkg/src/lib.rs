//! Items and logic related to the locking, building and updating of Forc packages.
//!
//! A forc "package" represents a Sway project with a `Forc.toml` "manifest" file declared at its
//! root. The project should consist of one or more Sway modules under a `src` directory. It may
//! also declare a set of forc package dependencies within its manifest.

pub mod lock;
pub mod manifest;
mod pkg;

pub use lock::Lock;
pub use manifest::Manifest;
#[doc(inline)]
pub use pkg::*;
