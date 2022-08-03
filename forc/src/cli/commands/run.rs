use crate::ops::forc_run;
use anyhow::{bail, Result};
use clap::Parser;

/// Run script project.
/// Crafts a script transaction then sends it to a running node.
#[derive(Debug, Default, Parser)]
pub struct Command {
    /// Hex string of data to input to script.
    #[clap(short, long)]
    pub data: Option<String>,

    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,

    /// Whether to compile using the original (pre- IR) pipeline.
    #[clap(long, hide = true)]
    pub use_orig_asm: bool,

    /// Only craft transaction and print it out.
    #[clap(long)]
    pub dry_run: bool,

    /// URL of the Fuel Client Node
    #[clap(env = "FUEL_NODE_URL")]
    pub node_url: Option<String>,

    /// Kill Fuel Node Client after running the code.
    /// This is only available if the node is started from `forc run`
    #[clap(short, long)]
    pub kill_node: bool,

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

    /// Silent mode. Don't output any warnings or errors to the command line.
    #[clap(long = "silent", short = 's')]
    pub silent_mode: bool,

    /// Output the time elapsed over each part of the compilation process.
    #[clap(long)]
    pub time_phases: bool,

    /// Pretty-print the outputs from the node.
    #[clap(long = "pretty-print", short = 'r')]
    pub pretty_print: bool,

    /// 32-byte contract ID that will be called during the transaction.
    #[clap(long = "contract")]
    pub contract: Option<Vec<String>>,

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

    /// Set the transaction byte price. Defaults to 0.
    #[clap(long)]
    pub byte_price: Option<u64>,

    /// Set the transaction gas limit. Defaults to the maximum gas limit.
    #[clap(long)]
    pub gas_limit: Option<u64>,

    /// Set the transaction gas price. Defaults to 0.
    #[clap(long)]
    pub gas_price: Option<u64>,

    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error
    #[clap(long)]
    pub locked: bool,

    /// Execute the transaction and return the final mutated transaction along with receipts
    /// (which includes whether the transaction reverted or not). The transaction is not inserted
    /// in the node's view of the blockchain, (i.e. it does not affect the chain state).
    #[clap(long)]
    pub simulate: bool,
}

pub(crate) async fn exec(command: Command) -> Result<()> {
    match forc_run::run(command).await {
        Err(e) => bail!("{}", e),
        _ => Ok(()),
    }
}
