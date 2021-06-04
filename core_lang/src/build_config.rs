
use std::path::PathBuf;

/// Configuration for the overall build and compilation process.
pub struct BuildConfig {
    canonicalized_manifest_path: PathBuf,
}

impl BuildConfig {
    // note this is intentionally not the trait Default
    // since we need at least a manifest path to work with
    fn default(canonicalized_manifest_path: PathBuf) -> Self {
        Self {
            canonicalized_manifest_path
        }
    }
}