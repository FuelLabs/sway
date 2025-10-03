use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use strum::{Display, EnumString};
use sway_ir::{PassManager, PrintPassesOpts};

#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    Default,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
    clap::ValueEnum,
    EnumString,
)]
pub enum BuildTarget {
    #[default]
    #[serde(rename = "fuel")]
    #[clap(name = "fuel")]
    #[strum(serialize = "fuel")]
    Fuel,
    #[serde(rename = "evm")]
    #[clap(name = "evm")]
    #[strum(serialize = "evm")]
    EVM,
}

impl BuildTarget {
    pub const CFG: &'static [&'static str] = &["evm", "fuel"];
}

#[derive(Default, Clone, Copy)]
pub enum DbgGeneration {
    Full,
    #[default]
    None,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OptLevel {
    #[default]
    Opt0 = 0,
    Opt1 = 1,
}

impl<'de> serde::Deserialize<'de> for OptLevel {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let num = u8::deserialize(d)?;
        match num {
            0 => Ok(OptLevel::Opt0),
            1 => Ok(OptLevel::Opt1),
            _ => Err(serde::de::Error::custom(format!("invalid opt level {num}"))),
        }
    }
}

/// Which ASM to print.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrintAsm {
    #[serde(rename = "virtual")]
    pub virtual_abstract: bool,
    #[serde(rename = "allocated")]
    pub allocated_abstract: bool,
    pub r#final: bool,
}

impl PrintAsm {
    pub fn all() -> Self {
        Self {
            virtual_abstract: true,
            allocated_abstract: true,
            r#final: true,
        }
    }

    pub fn abstract_virtual() -> Self {
        Self {
            virtual_abstract: true,
            ..Self::default()
        }
    }

    pub fn abstract_allocated() -> Self {
        Self {
            allocated_abstract: true,
            ..Self::default()
        }
    }

    pub fn r#final() -> Self {
        Self {
            r#final: true,
            ..Self::default()
        }
    }
}

impl std::ops::BitOrAssign for PrintAsm {
    fn bitor_assign(&mut self, rhs: Self) {
        self.virtual_abstract |= rhs.virtual_abstract;
        self.allocated_abstract |= rhs.allocated_abstract;
        self.r#final |= rhs.r#final;
    }
}

/// Which IR states to print.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PrintIr {
    pub initial: bool,
    pub r#final: bool,
    #[serde(rename = "modified")]
    pub modified_only: bool,
    pub passes: Vec<String>,
}

impl Default for PrintIr {
    fn default() -> Self {
        Self {
            initial: false,
            r#final: false,
            modified_only: true, // Default option is more restrictive.
            passes: vec![],
        }
    }
}

impl PrintIr {
    pub fn all(modified_only: bool) -> Self {
        Self {
            initial: true,
            r#final: true,
            modified_only,
            passes: PassManager::OPTIMIZATION_PASSES
                .iter()
                .map(|pass| pass.to_string())
                .collect_vec(),
        }
    }

    pub fn r#final() -> Self {
        Self {
            r#final: true,
            ..Self::default()
        }
    }
}

impl std::ops::BitOrAssign for PrintIr {
    fn bitor_assign(&mut self, rhs: Self) {
        self.initial |= rhs.initial;
        self.r#final |= rhs.r#final;
        // Both sides must request only passes that modify IR
        // in order for `modified_only` to be true.
        // Otherwise, displaying passes regardless if they
        // are modified or not wins.
        self.modified_only &= rhs.modified_only;
        for pass in rhs.passes {
            if !self.passes.contains(&pass) {
                self.passes.push(pass);
            }
        }
    }
}

impl From<&PrintIr> for PrintPassesOpts {
    fn from(value: &PrintIr) -> Self {
        Self {
            initial: value.initial,
            r#final: value.r#final,
            modified_only: value.modified_only,
            passes: HashSet::from_iter(value.passes.iter().cloned()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum Backtrace {
    All,
    #[default]
    AllExceptNever,
    OnlyAlways,
    None,
}

impl From<Backtrace> for sway_ir::Backtrace {
    fn from(value: Backtrace) -> Self {
        match value {
            Backtrace::All => sway_ir::Backtrace::All,
            Backtrace::AllExceptNever => sway_ir::Backtrace::AllExceptNever,
            Backtrace::OnlyAlways => sway_ir::Backtrace::OnlyAlways,
            Backtrace::None => sway_ir::Backtrace::None,
        }
    }
}

/// Configuration for the overall build and compilation process.
#[derive(Clone)]
pub struct BuildConfig {
    // Build target for code generation.
    pub(crate) build_target: BuildTarget,
    pub(crate) dbg_generation: DbgGeneration,
    // The canonical file path to the root module.
    // E.g. `/home/user/project/src/main.sw`.
    pub(crate) canonical_root_module: Arc<PathBuf>,
    pub(crate) print_dca_graph: Option<String>,
    pub(crate) print_dca_graph_url_format: Option<String>,
    pub(crate) print_asm: PrintAsm,
    pub(crate) print_bytecode: bool,
    pub(crate) print_bytecode_spans: bool,
    pub(crate) print_ir: PrintIr,
    pub(crate) include_tests: bool,
    pub(crate) optimization_level: OptLevel,
    pub(crate) backtrace: Backtrace,
    pub time_phases: bool,
    pub profile: bool,
    pub metrics_outfile: Option<String>,
    pub lsp_mode: Option<LspConfig>,
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
        build_target: BuildTarget,
        dbg_generation: DbgGeneration,
    ) -> Self {
        assert!(
            canonical_manifest_dir.has_root(),
            "manifest dir must be a canonical path",
        );
        let canonical_root_module = match root_module.has_root() {
            true => root_module,
            false => {
                assert!(
                    root_module.starts_with(canonical_manifest_dir.file_name().unwrap()),
                    "file_name must be either absolute or relative to manifest directory",
                );
                canonical_manifest_dir
                    .parent()
                    .expect("unable to retrieve manifest directory parent")
                    .join(&root_module)
            }
        };
        Self {
            build_target,
            dbg_generation,
            canonical_root_module: Arc::new(canonical_root_module),
            print_dca_graph: None,
            print_dca_graph_url_format: None,
            print_asm: PrintAsm::default(),
            print_bytecode: false,
            print_bytecode_spans: false,
            print_ir: PrintIr::default(),
            include_tests: false,
            time_phases: false,
            profile: false,
            metrics_outfile: None,
            optimization_level: OptLevel::default(),
            backtrace: Backtrace::default(),
            lsp_mode: None,
        }
    }

    /// Dummy build config that can be used for testing.
    /// This is not valid generally, but asm generation will accept it.
    pub fn dummy_for_asm_generation() -> Self {
        Self::root_from_file_name_and_manifest_path(
            PathBuf::from("/"),
            PathBuf::from("/"),
            BuildTarget::default(),
            DbgGeneration::None,
        )
    }

    pub fn with_print_dca_graph(self, a: Option<String>) -> Self {
        Self {
            print_dca_graph: a,
            ..self
        }
    }

    pub fn with_print_dca_graph_url_format(self, a: Option<String>) -> Self {
        Self {
            print_dca_graph_url_format: a,
            ..self
        }
    }

    pub fn with_print_asm(self, print_asm: PrintAsm) -> Self {
        Self { print_asm, ..self }
    }

    pub fn with_print_bytecode(self, bytecode: bool, bytecode_spans: bool) -> Self {
        Self {
            print_bytecode: bytecode,
            print_bytecode_spans: bytecode_spans,
            ..self
        }
    }

    pub fn with_print_ir(self, a: PrintIr) -> Self {
        Self {
            print_ir: a,
            ..self
        }
    }

    pub fn with_time_phases(self, a: bool) -> Self {
        Self {
            time_phases: a,
            ..self
        }
    }

    pub fn with_profile(self, a: bool) -> Self {
        Self { profile: a, ..self }
    }

    pub fn with_metrics(self, a: Option<String>) -> Self {
        Self {
            metrics_outfile: a,
            ..self
        }
    }

    pub fn with_optimization_level(self, optimization_level: OptLevel) -> Self {
        Self {
            optimization_level,
            ..self
        }
    }

    pub fn with_backtrace(self, backtrace: Backtrace) -> Self {
        Self { backtrace, ..self }
    }

    /// Whether or not to include test functions in parsing, type-checking and codegen.
    ///
    /// This should be set to `true` by invocations like `forc test` or `forc check --tests`.
    ///
    /// Default: `false`
    pub fn with_include_tests(self, include_tests: bool) -> Self {
        Self {
            include_tests,
            ..self
        }
    }

    pub fn with_lsp_mode(self, lsp_mode: Option<LspConfig>) -> Self {
        Self { lsp_mode, ..self }
    }

    pub fn canonical_root_module(&self) -> Arc<PathBuf> {
        self.canonical_root_module.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LspConfig {
    // This is set to true if compilation was triggered by a didChange LSP event. In this case, we
    // bypass collecting type metadata and skip DCA.
    //
    // This is set to false if compilation was triggered by a didSave or didOpen LSP event.
    pub optimized_build: bool,
    // The value of the `version` field in the `DidChangeTextDocumentParams` struct.
    // This is used to determine if the file has been modified since the last compilation.
    pub file_versions: BTreeMap<PathBuf, Option<u64>>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_root_from_file_name_and_manifest_path() {
        let root_module = PathBuf::from("mock_path/src/main.sw");
        let canonical_manifest_dir = PathBuf::from("/tmp/sway_project/mock_path");
        BuildConfig::root_from_file_name_and_manifest_path(
            root_module,
            canonical_manifest_dir,
            BuildTarget::default(),
            DbgGeneration::Full,
        );
    }

    #[test]
    fn test_root_from_file_name_and_manifest_path_contains_dot() {
        let root_module = PathBuf::from("mock_path_contains_._dot/src/main.sw");
        let canonical_manifest_dir = PathBuf::from("/tmp/sway_project/mock_path_contains_._dot");
        BuildConfig::root_from_file_name_and_manifest_path(
            root_module,
            canonical_manifest_dir,
            BuildTarget::default(),
            DbgGeneration::Full,
        );
    }
}
