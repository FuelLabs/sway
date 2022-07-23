use crate::{cli::init::Command as InitCommand, ops::forc_init::init};
use anyhow::{bail, Result};
use clap::Parser;
use std::path::Path;

/// Create a new Forc project at `<path>`.
#[derive(Debug, Parser)]
pub struct Command {
    /// The default program type. Excluding all flags or adding this flag creates a basic contract
    /// program.
    #[clap(long)]
    pub contract: bool,
    /// Adding this flag creates an empty script program.
    #[clap(long)]
    pub script: bool,
    /// Adding this flag creates an empty predicate program.
    #[clap(long)]
    pub predicate: bool,
    /// Adding this flag creates an empty library program.
    #[clap(long)]
    pub library: bool,
    /// Set the package name. Defaults to the directory name
    #[clap(long)]
    pub name: Option<String>,
    /// Use verbose output.
    #[clap(short = 'v', long)]
    pub verbose: bool,
    /// The path at which the project directory will be created.
    pub path: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    // `forc new` is roughly short-hand for `forc init`, but we create the directory first if it
    // doesn't yet exist. Here we create the directory if it doesn't exist then re-use the existing
    // `forc init` logic.
    let Command {
        contract,
        script,
        predicate,
        library,
        name,
        verbose,
        path,
    } = command;

    let dir_path = Path::new(&path);
    if dir_path.exists() {
        bail!(
            "Directory \"{}\" already exists.\nIf you wish to initialise a forc project inside \
            this directory, consider using `forc init --path {}`",
            dir_path.canonicalize()?.display(),
            dir_path.display(),
        );
    } else {
        std::fs::create_dir_all(dir_path)?;
    }

    let init_cmd = InitCommand {
        path: Some(path),
        contract,
        script,
        predicate,
        library,
        verbose,
        name,
    };

    init(init_cmd)
}
