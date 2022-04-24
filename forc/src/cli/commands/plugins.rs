use crate::cli::PluginsCommand;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Find all forc plugins available via `PATH`.
///
/// Prints information about each discovered plugin.
#[derive(Debug, Parser)]
pub struct Command {
    /// Prints the absolute path to each discovered plugin.
    #[clap(long = "verbose", short = 'v')]
    print_full_path: bool,
}

pub(crate) fn exec(command: PluginsCommand) -> Result<()> {
    let PluginsCommand { print_full_path } = command;

    for path in crate::cli::plugin::find_all() {
        print_plugin(path, print_full_path);
    }
    Ok(())
}

fn print_plugin(path: PathBuf, print_full_path: bool) {
    if print_full_path {
        println!("{}", path.display());
    } else {
        if let Some(file_name) = path.file_name() {
            println!("{}", file_name.to_str().unwrap());
        } else {
            println!("{}", path.display());
        }
    }
}
