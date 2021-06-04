use std::path::PathBuf;

/// Configuration for the overall build and compilation process.
#[derive(Clone)]
pub struct BuildConfig {
    pub(crate) dir_of_code: PathBuf,
}

impl BuildConfig {
    // note this is intentionally not the trait Default
    // since we need at least a manifest path to work with
    pub fn root_from_manifest_path(canonicalized_manifest_path: PathBuf) -> Self {
        let mut path = canonicalized_manifest_path.clone();
        path.push("src");
        Self { dir_of_code: path }
    }
}
