//! Sets of arguments that are shared between commands.
use clap::{ArgGroup, Args, Parser};
use forc_pkg::source::IPFSNode;
use sway_core::{BuildTarget, IrCli, PrintAsm};
use sway_ir::PassManager;

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("source")
        .required(false)
        .args(["path", "git", "ipfs"]),
))]
pub struct SourceArgs {
    /// Local path to the package.
    #[arg(long)]
    pub path: Option<String>,

    /// Git URI for the package.
    #[arg(long, value_name = "URI")]
    pub git: Option<String>,

    /// Git reference options like `branch`, `rev`, etc.
    #[clap(flatten)]
    pub git_ref: GitRef,

    /// IPFS CID for the package.
    #[arg(long, value_name = "CID")]
    pub ipfs: Option<String>,
}

#[derive(Args, Debug, Default)]
#[command(group(
    ArgGroup::new("git_ref")
        .args(["branch", "tag", "rev"])
        .multiple(false)
        .requires("git")
))]
pub struct GitRef {
    /// The branch to use.
    #[arg(long)]
    pub branch: Option<String>,

    /// The tag to use.
    #[arg(long)]
    pub tag: Option<String>,

    /// The specific revision to use.
    #[arg(long)]
    pub rev: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct SectionArgs {
    /// Treats dependency as contract dependencies.
    #[arg(long = "contract-dep")]
    pub contract_deps: bool,

    /// Salt value for contract deployment.
    #[arg(long = "salt")]
    pub salt: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct ManifestArgs {
    /// Path to the manifest file.
    #[arg(long, value_name = "PATH")]
    pub manisfest_path: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct PackagesSelectionArgs {
    /// Package to perform action on.
    #[arg(long, short = 'p', value_name = "SPEC")]
    pub package: Option<String>,
}

/// Args that can be shared between all commands that `build` a package. E.g. `build`, `test`,
/// `deploy`.
#[derive(Debug, Default, Parser)]
pub struct Build {
    #[clap(flatten)]
    pub pkg: Pkg,
    #[clap(flatten)]
    pub print: Print,
    /// Verify the generated Sway IR (Intermediate Representation).
    ///
    /// Values that can be combined:
    ///  - initial:     initial IR prior to any optimization passes.
    ///  - final:       final IR after applying all optimization passes.
    ///  - <pass name>: the name of an optimization pass. Verifies the IR state after that pass.
    ///  - all:         short for initial, final, and all the optimization passes.
    ///  - modified:    verify a requested optimization pass only if it has modified the IR.
    #[arg(long, verbatim_doc_comment, num_args(1..=18), value_parser = clap::builder::PossibleValuesParser::new(PrintIrCliOpt::cli_options()))]
    pub verify_ir: Option<Vec<String>>,
    #[clap(flatten)]
    pub minify: Minify,
    #[clap(flatten)]
    pub output: BuildOutput,
    #[clap(flatten)]
    pub profile: BuildProfile,
    /// Build target to use for code generation.
    #[clap(long, value_enum, default_value_t = BuildTarget::default(), alias="target")]
    pub build_target: BuildTarget,
    #[clap(flatten)]
    pub dump: Dump,
}

impl Build {
    pub fn verify_ir(&self) -> IrCli {
        self.verify_ir
            .as_ref()
            .map_or(IrCli::default(), |opts| PrintIrCliOpt::from(opts).0)
    }
}

/// Build output file options.
#[derive(Args, Debug, Default)]
pub struct BuildOutput {
    /// Create a binary file at the provided path representing the final bytecode.
    #[clap(long = "output-bin", short = 'o')]
    pub bin_file: Option<String>,
    /// Create a file at the provided path containing debug information.
    ///
    /// If the file extension is .json, JSON format is used. Otherwise, an .elf file containing DWARF format is emitted.
    #[clap(long = "output-debug", short = 'g')]
    pub debug_file: Option<String>,

    /// Generates a JSON file containing the hex-encoded script binary.
    #[clap(long = "output-hexfile")]
    pub hex_file: Option<String>,
}

/// Build profile options.
#[derive(Args, Debug, Default)]
pub struct BuildProfile {
    /// The name of the build profile to use.
    #[clap(long, conflicts_with = "release", default_value = forc_pkg::BuildProfile::DEBUG)]
    pub build_profile: String,
    /// Use the release build profile.
    ///
    /// The release profile can be customized in the manifest file.
    #[clap(long)]
    pub release: bool,
    /// Treat warnings as errors.
    #[clap(long)]
    pub error_on_warnings: bool,
}

/// Dump options.
#[derive(Args, Debug, Default)]
pub struct Dump {
    /// Dump all trait implementations for the given type name.
    #[clap(long = "dump-impls", value_name = "TYPE")]
    pub dump_impls: Option<String>,
}

/// Options related to printing stages of compiler output.
#[derive(Args, Debug, Default)]
pub struct Print {
    /// Print the generated Sway AST (Abstract Syntax Tree).
    #[clap(long)]
    pub ast: bool,
    /// Print the computed Sway DCA (Dead Code Analysis) graph.
    ///
    /// DCA graph is printed to the specified path.
    /// If specified '' graph is printed to the stdout.
    #[clap(long)]
    pub dca_graph: Option<String>,
    /// URL format to be used in the generated DCA graph .dot file.
    ///
    /// Variables {path}, {line}, and {col} can be used in the provided format.
    /// An example for vscode would be:
    ///   "vscode://file/{path}:{line}:{col}"
    #[clap(long, verbatim_doc_comment)]
    pub dca_graph_url_format: Option<String>,
    /// Print the generated ASM (assembler).
    ///
    /// Values that can be combined:
    ///  - virtual:   initial ASM with virtual registers and abstract control flow.
    ///  - allocated: ASM with registers allocated, but still with abstract control flow.
    ///  - abstract:  short for both virtual and allocated ASM.
    ///  - final:     final ASM that gets serialized to the target VM bytecode.
    ///  - all:       short for virtual, allocated, and final ASM.
    #[arg(long, verbatim_doc_comment, num_args(1..=5), value_parser = clap::builder::PossibleValuesParser::new(&PrintAsmCliOpt::CLI_OPTIONS))]
    pub asm: Option<Vec<String>>,
    /// Print the bytecode.
    ///
    /// This is the final output of the compiler.
    #[clap(long)]
    pub bytecode: bool,
    /// Print the generated Sway IR (Intermediate Representationn).
    ///
    /// Values that can be combined:
    ///  - initial:     initial IR prior to any optimization passes.
    ///  - final:       final IR after applying all optimization passes.
    ///  - <pass name>: the name of an optimization pass. Prints the IR state after that pass.
    ///  - all:         short for initial, final, and all the optimization passes.
    ///  - modified:    print a requested optimization pass only if it has modified the IR.
    #[arg(long, verbatim_doc_comment, num_args(1..=18), value_parser = clap::builder::PossibleValuesParser::new(PrintIrCliOpt::cli_options()))]
    pub ir: Option<Vec<String>>,
    /// Output the time elapsed over each part of the compilation process.
    #[clap(long)]
    pub time_phases: bool,
    /// Profile the compilation process.
    #[clap(long)]
    pub profile: bool,
    /// Output build errors and warnings in reverse order.
    #[clap(long)]
    pub reverse_order: bool,
    /// Output compilation metrics into the specified file.
    #[clap(long)]
    pub metrics_outfile: Option<String>,
}

impl Print {
    pub fn asm(&self) -> PrintAsm {
        self.asm
            .as_ref()
            .map_or(PrintAsm::default(), |opts| PrintAsmCliOpt::from(opts).0)
    }

    pub fn ir(&self) -> IrCli {
        self.ir
            .as_ref()
            .map_or(IrCli::default(), |opts| PrintIrCliOpt::from(opts).0)
    }
}

/// Package-related options.
#[derive(Args, Debug, Default)]
pub struct Pkg {
    /// Path to the project.
    ///
    /// If not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// Offline mode.
    ///
    /// Prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long)]
    pub offline: bool,
    /// Terse mode.
    ///
    /// Limited warning and error output.
    #[clap(long, short = 't')]
    pub terse: bool,
    /// The directory in which Forc output artifacts are placed.
    ///
    /// By default, this is `<project-root>/out`.
    #[clap(long)]
    pub output_directory: Option<String>,
    /// Requires that the Forc.lock file is up-to-date.
    ///
    /// If the lock file is missing, or it needs to be updated, Forc will exit with an error.
    #[clap(long)]
    pub locked: bool,
    /// The IPFS node to use for fetching IPFS sources.
    ///
    /// [possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>]
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,
}

/// Options related to minifying output.
#[derive(Args, Debug, Default)]
pub struct Minify {
    /// Minify JSON ABI files.
    ///
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub json_abi: bool,
    /// Minify JSON storage slot files.
    ///
    /// By default the JSON for initial storage slots is formatted for human readability. By using
    /// this option JSON output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub json_storage_slots: bool,
}

pub struct PrintAsmCliOpt(pub PrintAsm);

impl PrintAsmCliOpt {
    const VIRTUAL: &'static str = "virtual";
    const ALLOCATED: &'static str = "allocated";
    const ABSTRACT: &'static str = "abstract";
    const FINAL: &'static str = "final";
    const ALL: &'static str = "all";
    pub const CLI_OPTIONS: [&'static str; 5] = [
        Self::VIRTUAL,
        Self::ALLOCATED,
        Self::ABSTRACT,
        Self::FINAL,
        Self::ALL,
    ];
}

impl From<&Vec<String>> for PrintAsmCliOpt {
    fn from(value: &Vec<String>) -> Self {
        let contains_opt = |opt: &str| value.iter().any(|val| *val == opt);

        let print_asm = if contains_opt(Self::ALL) {
            PrintAsm::all()
        } else {
            PrintAsm {
                virtual_abstract: contains_opt(Self::ABSTRACT) || contains_opt(Self::VIRTUAL),
                allocated_abstract: contains_opt(Self::ABSTRACT) || contains_opt(Self::ALLOCATED),
                r#final: contains_opt(Self::FINAL),
            }
        };

        Self(print_asm)
    }
}

pub struct PrintIrCliOpt(pub IrCli);

impl PrintIrCliOpt {
    const INITIAL: &'static str = "initial";
    const FINAL: &'static str = "final";
    const ALL: &'static str = "all";
    const MODIFIED: &'static str = "modified";
    pub const CLI_OPTIONS: [&'static str; 4] =
        [Self::INITIAL, Self::FINAL, Self::ALL, Self::MODIFIED];

    pub fn cli_options() -> Vec<&'static str> {
        Self::CLI_OPTIONS
            .iter()
            .chain(PassManager::OPTIMIZATION_PASSES.iter())
            .cloned()
            .collect()
    }
}

impl From<&Vec<String>> for PrintIrCliOpt {
    fn from(value: &Vec<String>) -> Self {
        let contains_opt = |opt: &str| value.iter().any(|val| *val == opt);

        let print_ir = if contains_opt(Self::ALL) {
            IrCli::all(contains_opt(Self::MODIFIED))
        } else {
            IrCli {
                initial: contains_opt(Self::INITIAL),
                r#final: contains_opt(Self::FINAL),
                modified_only: contains_opt(Self::MODIFIED),
                passes: value
                    .iter()
                    .filter(|val| !Self::CLI_OPTIONS.contains(&val.as_str()))
                    .cloned()
                    .collect(),
            }
        };

        Self(print_ir)
    }
}
