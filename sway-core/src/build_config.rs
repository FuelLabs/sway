use std::{path::PathBuf, sync::Arc};

/// Configuration for the overall build and compilation process.
#[derive(Clone)]
pub struct BuildConfig {
    pub(crate) file_name: Arc<PathBuf>,
    pub(crate) dir_of_code: Arc<PathBuf>,
    pub(crate) manifest_path: Arc<PathBuf>,
    pub(crate) use_ir: bool,
    pub(crate) print_intermediate_asm: bool,
    pub(crate) print_finalized_asm: bool,
    pub(crate) print_ir: bool,
}

impl BuildConfig {
    // note this is intentionally not the trait Default
    // since we need at least a manifest path to work with
    pub fn root_from_file_name_and_manifest_path(
        file_name: PathBuf,
        canonicalized_manifest_path: PathBuf,
    ) -> Self {
        let mut path = canonicalized_manifest_path.clone();
        path.push("src");
        Self {
            file_name: Arc::new(file_name),
            dir_of_code: Arc::new(path),
            manifest_path: Arc::new(canonicalized_manifest_path),
            use_ir: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
        }
    }

    pub fn use_ir(self, a: bool) -> Self {
        Self { use_ir: a, ..self }
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

    pub fn print_ir(self, a: bool) -> Self {
        Self {
            print_ir: a,
            ..self
        }
    }

    pub fn path(&self) -> Arc<PathBuf> {
        self.file_name.clone()
    }
}
