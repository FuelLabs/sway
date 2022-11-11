use crate::ops::forc_edit;
use anyhow::Result;
use clap::Parser;
// A utility for managing forc dependencies from the command line.

/// Add or remove dependencies to a Forc.toml manifest file.
#[derive(Debug, Parser)]
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
pub struct Command {
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
    forc_edit::add(command)?;

    Ok(())
}
