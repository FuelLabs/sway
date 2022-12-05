use std::{path::PathBuf, sync::Arc};

/// Configuration for the overall build and compilation process.
#[derive(Clone)]
pub struct BuildConfig {
    // The canonical file path to the root module.
    // E.g. `/home/user/project/src/main.sw`.
    pub(crate) canonical_root_module: Arc<PathBuf>,
    pub(crate) print_dca_graph: bool,
    pub(crate) print_intermediate_asm: bool,
    pub(crate) print_finalized_asm: bool,
    pub(crate) print_ir: bool,
    pub(crate) include_tests: bool,
}

impl BuildConfig {
    /// Construct a `BuildConfig` from a relative path to the root module and the canonical path to
    /// the manifest directory.
    ///
    /// The `root_module` path must be either canonical, or relative to the directory containing
    /// the manifest. E.g. `project/src/main.sw` or `project/src/lib.sw`.
    ///
    /// The `canonical_manifest_dir` must be the canonical (aka absolute) path to the directory
    /// containing the `Forc.toml` file for the project. E.g. `/home/user/project`.
    pub fn root_from_file_name_and_manifest_path(
        root_module: PathBuf,
        canonical_manifest_dir: PathBuf,
    ) -> Self {
        assert!(
            canonical_manifest_dir.has_root(),
            "manifest dir must be a canonical path",
        );
        let canonical_root_module = match root_module.has_root() {
            true => root_module,
            false => {
                assert!(
                    root_module.starts_with(canonical_manifest_dir.file_stem().unwrap()),
                    "file_name must be either absolute or relative to manifest directory",
                );
                canonical_manifest_dir
                    .parent()
                    .expect("unable to retrieve manifest directory parent")
                    .join(&root_module)
            }
        };
        Self {
            canonical_root_module: Arc::new(canonical_root_module),
            print_dca_graph: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
            include_tests: false,
        }
    }

    pub fn print_dca_graph(self, a: bool) -> Self {
        Self {
            print_dca_graph: a,
            ..self
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

    pub fn print_ir(self, a: bool) -> Self {
        Self {
            print_ir: a,
            ..self
        }
    }

    /// Whether or not to include test functions in parsing, type-checking and codegen.
    ///
    /// This should be set to `true` by invocations like `forc test` or `forc check --tests`.
    ///
    /// Default: `false`
    pub fn include_tests(self, include_tests: bool) -> Self {
        Self {
            include_tests,
            ..self
        }
    }

    pub fn canonical_root_module(&self) -> Arc<PathBuf> {
        self.canonical_root_module.clone()
    }
}
