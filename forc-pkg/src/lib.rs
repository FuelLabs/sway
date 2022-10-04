//! Building, locking, fetching and updating sway projects as Forc packages.
//!
//! A forc package represents a Sway project with a `Forc.toml` manifest file declared at its root.
//! The project should consist of one or more Sway modules under a `src` directory. It may also
//! declare a set of forc package dependencies within its manifest.

pub mod lock;
pub mod manifest;
mod pkg;

pub use lock::Lock;
pub use manifest::{BuildProfile, PackageManifest, PackageManifestFile};
#[doc(inline)]
pub use pkg::*;

const CORE: &str = "core";
const STD: &str = "std";
const PRELUDE: &str = "prelude";
