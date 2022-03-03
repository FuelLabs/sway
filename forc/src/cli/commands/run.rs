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

    /// Whether to compile using the IR pipeline.
    #[clap(long)]
    pub use_ir: bool,

    /// Only craft transaction and print it out.
    #[clap(long)]
    pub dry_run: bool,

    /// URL of the Fuel Client Node
    #[clap(env = "FUEL_NODE_URL", default_value = "127.0.0.1:4000")]
    pub node_url: String,

    /// Kill Fuel Node Client after running the code.
    /// This is only available if the node is started from `forc run`
    #[clap(short, long)]
    pub kill_node: bool,

    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[clap(long)]
    pub print_finalized_asm: bool,

    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[clap(long)]
    pub print_intermediate_asm: bool,

    /// Whether to compile to bytecode (false) or to print out the IR (true).
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
}

pub(crate) async fn exec(command: Command) -> Result<()> {
    match forc_run::run(command).await {
        Err(e) => bail!(e.message),
        _ => Ok(()),
    }
}
