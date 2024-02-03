use crate::{cli, ops::forc_build};
use clap::Parser;
use forc_util::ForcResult;

forc_util::cli_examples! {
    [ Compile the current projectx => forc "build" ]
    [ Compile the current project from a different path => forc "build --path /tmp/path-to-contract" ]
    [ Compile the current project without updating dependencies => forc "build --path /tmp/path-to-contract --locked" ]
    setup {
        crate::cli::create_project_and_compile("/tmp/path-to-contract", false);
    }
}

/// Compile the current or target project.
///
/// The output produced will depend on the project's program type.
///
/// - `script`, `predicate` and `contract` projects will produce their bytecode in binary format
///   `<project-name>.bin`.
///
/// - `script` projects will also produce a file containing the hash of the bytecode binary
/// `<project-name>-bin-hash` (using `fuel_cypto::Hasher`).
///
/// - `predicate` projects will also produce a file containing the **root** hash of the bytecode
///   binary
/// `<project-name>-bin-root` (using `fuel_tx::Contract::root_from_code`).
///
/// - `contract` and `library` projects will also produce the public ABI in JSON format
/// `<project-name>-abi.json`.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc build", version, after_help = help())]
pub struct Command {
    #[clap(flatten)]
    pub build: cli::shared::Build,
    /// Also build all tests within the project.
    #[clap(long)]
    pub tests: bool,

    #[clap(long)]
    /// Experimental flag for the "new encoding" feature
    pub experimental_new_encoding: bool,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    forc_build::build(command)?;
    Ok(())
}
