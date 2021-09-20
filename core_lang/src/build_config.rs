use std::path::PathBuf;

/// Configuration for the overall build and compilation process.
#[derive(Clone)]
pub struct BuildConfig {
    pub(crate) file_path: PathBuf,
    pub(crate) dir_of_code: PathBuf,
    pub(crate) print_intermediate_asm: bool,
    pub(crate) print_finalized_asm: bool,
}

impl BuildConfig {
    // note this is intentionally not the trait Default
    // since we need at least a manifest path to work with
    pub fn root_from_file_path_and_manifest_path(file_path: PathBuf, canonicalized_manifest_path: PathBuf) -> Self {
        let mut path = canonicalized_manifest_path.clone();
        path.push("src");
        Self {
            file_path: file_path,
            dir_of_code: path,
            print_intermediate_asm: false,
            print_finalized_asm: false,
        }
    }

    pub fn print_intermediate_asm(self, a: bool) -> Self {
        Self {
            print_intermediate_asm: a,
            ..self
        }
    }

    pub fn print_finalized_asm(self, a: bool) -> Self {
        Self {
            print_finalized_asm: a,
            ..self
        }
    }
}
