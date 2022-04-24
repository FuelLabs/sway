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
    #[clap(long = "paths", short = 'p')]
    print_full_path: bool,
}

pub(crate) fn exec(command: PluginsCommand) -> Result<()> {
    let PluginsCommand { print_full_path } = command;

    for path in crate::cli::plugin::find_all() {
        print_plugin(path, print_full_path);
    }
    Ok(())
}

/// # Panics
///
/// This function assumes that file names will never be empty since it is only used with
/// paths yielded from plugin::find_all(), as well as that the file names are in valid
/// unicode format since file names should be prefixed with `forc-`. Should one of these 2
/// assumptions fail, this function panics.
fn print_plugin(path: PathBuf, print_full_path: bool) {
    if print_full_path {
        println!("{}", path.display());
    } else {
        println!(
            "{}",
            path.file_name()
                .expect("Failed to read file name")
                .to_str()
                .expect("Failed to print file name")
        );
    }
}
