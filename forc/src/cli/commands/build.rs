use crate::ops::forc_build;
use structopt::{self, StructOpt};

/// Compile the current or target project.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,
    /// Whether to compile using the IR pipeline.
    #[structopt(long)]
    pub use_ir: bool,
    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[structopt(long)]
    pub print_finalized_asm: bool,
    /// Whether to compile to bytecode (false) or to print out the generated ASM (true).
    #[structopt(long)]
    pub print_intermediate_asm: bool,
    /// Whether to compile to bytecode (false) or to print out the generated IR (true).
    #[structopt(long)]
    pub print_ir: bool,
    /// If set, outputs a binary file representing the script bytes.
    #[structopt(short = "o")]
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    #[structopt(short = "g", long)]
    pub debug_outfile: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[structopt(long = "offline")]
    pub offline_mode: bool,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[structopt(long = "silent", short = "s")]
    pub silent_mode: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_build::build(command)?;
    Ok(())
}
