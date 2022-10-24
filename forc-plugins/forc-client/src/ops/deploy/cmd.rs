use clap::Parser;
use fuel_crypto::SecretKey;

#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc deploy", version)]
pub struct DeployCommand {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// Print the generated Sway AST (Abstract Syntax Tree).
    #[clap(long)]
    pub print_ast: bool,
    /// Print the finalized ASM.
    ///
    /// This is the state of the ASM with registers allocated and optimisations applied.
    #[clap(long)]
    pub print_finalized_asm: bool,
    /// Print the generated ASM.
    ///
    /// This is the state of the ASM prior to performing register allocation and other ASM
    /// optimisations.
    #[clap(long)]
    pub print_intermediate_asm: bool,
    /// Print the generated Sway IR (Intermediate Representation).
    #[clap(long)]
    pub print_ir: bool,
    /// If set, outputs a binary file representing the script bytes.
    #[clap(short = 'o')]
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    #[clap(short = 'g', long)]
    pub debug_outfile: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long = "offline")]
    pub offline_mode: bool,
    /// Terse mode. Limited warning and error output.
    #[clap(long = "terse", short = 't')]
    pub terse_mode: bool,
    /// The directory in which the sway compiler output artifacts are placed.
    ///
    /// By default, this is `<project-root>/out`.
    #[clap(long)]
    pub output_directory: Option<String>,
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub minify_json_abi: bool,
    /// By default the JSON for initial storage slots is formatted for human readability. By using
    /// this option JSON output will be "minified", i.e. all on one line without whitespace.
    #[clap(long)]
    pub minify_json_storage_slots: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    #[clap(long)]
    pub locked: bool,
    /// The node url to deploy, if not specified uses DEFAULT_NODE_URL.
    /// If url is specified overrides network url in manifest file (if there is one).
    #[clap(long, short)]
    pub url: Option<String>,
    /// Name of the build profile to use.
    /// If it is not specified, forc will use debug build profile.
    #[clap(long)]
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    /// If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    #[clap(long)]
    pub release: bool,
    /// Output the time elapsed over each part of the compilation process.
    #[clap(long)]
    pub time_phases: bool,
    /// Do not sign the transaction
    #[clap(long)]
    pub unsigned: bool,
    /// Set the transaction gas limit. Defaults to the maximum gas limit.
    #[clap(long)]
    pub gas_limit: Option<u64>,
    /// Set the transaction gas price. Defaults to 0.
    #[clap(long)]
    pub gas_price: Option<u64>,
    /// Set the key to be used for signing.
    pub signing_key: Option<SecretKey>,
}
