use crate::ops::forc_build;
use structopt::{self, StructOpt};

/// Compile the current or target project.
///
/// The output produced will depend on the project's program type. Building script, predicate and
/// contract projects will produce their bytecode in binary format `<project-name>.bin`. Building
/// contracts and libraries will also produce the public ABI in JSON format
/// `<project-name>-abi.json`.
#[derive(Debug, Default, StructOpt)]
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
    /// The directory in which the sway compiler output artifacts are placed.
    ///
    /// By default, this is `<project-root>/out`.
    #[structopt(long)]
    pub output_directory: Option<String>,
    /// By default the JSON for ABIs is formatted for human readability. By using this option JSON
    /// output will be "minified", i.e. all on one line without whitespace.
    #[structopt(long)]
    pub minify_json_abi: bool,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    forc_build::build(command)?;
    Ok(())
}
