//! Sets of arguments that are shared between commands.

use clap::{Args, Parser};
use sway_core::BuildTarget;

/// Args that can be shared between all commands that `build` a package. E.g. `build`, `test`,
/// `deploy`.
#[derive(Debug, Default, Parser)]
pub struct Build {
    #[clap(flatten)]
    pub pkg: Pkg,
    #[clap(flatten)]
    pub print: Print,
    #[clap(flatten)]
    pub minify: Minify,
    #[clap(flatten)]
    pub output: BuildOutput,
    #[clap(flatten)]
    pub profile: BuildProfile,
    /// Build target to use for code generation.
    #[clap(long, value_enum, default_value_t = BuildTarget::default(), alias="target")]
    pub build_target: BuildTarget,
}

/// Build output file options.
#[derive(Args, Debug, Default)]
pub struct BuildOutput {
    /// If set, outputs a binary file representing the script bytes.
    #[clap(long = "output-bin", short = 'o')]
    pub bin_file: Option<String>,
    /// If set, outputs source file mapping in JSON format
    #[clap(long = "output-debug", short = 'g')]
    pub debug_file: Option<String>,
}

/// Build profile options.
#[derive(Args, Debug, Default)]
pub struct BuildProfile {
    /// Name of the build profile to use.
    ///
    /// If unspecified, forc will use debug build profile.
    #[clap(long)]
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    ///  If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    #[clap(long)]
    pub release: bool,
    /// Treat warnings as errors.
    #[clap(long)]
    pub error_on_warnings: bool,
}

/// Options related to printing stages of compiler output.
#[derive(Args, Debug, Default)]
pub struct Print {
    /// Print the generated Sway AST (Abstract Syntax Tree).
    #[clap(long)]
    pub ast: bool,
    /// Print the computed Sway DCA graph.
    #[clap(long)]
    pub dca_graph: bool,
    /// Print the finalized ASM.
    ///
    /// This is the state of the ASM with registers allocated and optimisations applied.
    #[clap(long)]
    pub finalized_asm: bool,
    /// Print the generated ASM.
    ///
    /// This is the state of the ASM prior to performing register allocation and other ASM
    /// optimisations.
    #[clap(long)]
    pub intermediate_asm: bool,
    /// Print the generated Sway IR (Intermediate Representation).
    #[clap(long)]
    pub ir: bool,
    /// Output the time elapsed over each part of the compilation process.
    #[clap(long)]
    pub time_phases: bool,
}

/// Package-related options.
#[derive(Args, Debug, Default)]
pub struct Pkg {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long)]
    pub offline: bool,
    /// Terse mode. Limited warning and error output.
    #[clap(long, short = 't')]
    pub terse: bool,
    /// The directory in which the sway compiler output artifacts are placed.
    ///
    /// By default, this is `<project-root>/out`.
    #[clap(long)]
    pub output_directory: Option<String>,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    #[clap(long)]
    pub locked: bool,
    /// Outputs json abi with callpaths instead of names for struct and enums.
    #[clap(long)]
    pub json_abi_with_callpaths: bool,
}

/// Options related to minifying output.
#[derive(Args, Debug, Default)]
pub struct Minify {
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub json_abi: bool,
    /// By default the JSON for initial storage slots is formatted for human readability. By using
    /// this option JSON output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub json_storage_slots: bool,
}
