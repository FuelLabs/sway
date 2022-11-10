use clap::{Parser, Args};
use anyhow::Result;
use crate::ops::forc_edit;
// A utility for managing forc dependencies from the command line.
#[derive(Debug, Parser)]
pub enum Command {
    Add(CommandArgs),
    Remove(CommandArgs),
}
/// Add or remove dependencies to a Forc.toml manifest file.
#[derive(Debug, Args)]
#[clap(version)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
#[clap(after_help = "\
Example:
  $ forc add ./crate/parser/
  $ forc remove ./crate/parser/
")]
#[clap(override_usage = "\
    forc add [OPTIONS] <DEP_PATH> ...
    forc remove [OPTIONS] <DEP_PATH> ...
")]
pub struct CommandArgs {
    /// Reference to a package to add or remove as a dependency
    ///
    /// You can reference a packages by:
    /// - `<path>`, like `forc add ./crates/parser/` or `forc remove ./crates/parser/`
    ///
    #[clap(value_name = "DEP_ID")]
    pub crates: Vec<String>,

    /// Path to `Forc.toml`
    #[clap(long, value_name = "PATH", parse(from_os_str))]
    pub manifest_path: Option<std::path::PathBuf>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    match command {
        Command::Add(args) => forc_edit::add(args)?,
        Command::Remove(args) => forc_edit::remove(args)?,
    }
    Ok(())
}