use crate::cli::PluginsCommand;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use tracing::info;

/// Find all forc plugins available via `PATH`.
///
/// Prints information about each discovered plugin.
#[derive(Debug, Parser)]
pub struct Command {
    /// Prints the absolute path to each discovered plugin.
    #[clap(long = "paths", short = 'p')]
    print_full_path: bool,
    /// Prints the long description associated with each listed plugin
    #[clap(long = "describe", short = 'd')]
    describe: bool,
}

pub(crate) fn exec(command: PluginsCommand) -> Result<()> {
    let PluginsCommand {
        print_full_path,
        describe,
    } = command;

    info!("Installed Plugins:");
    for path in crate::cli::plugin::find_all() {
        info!("{}", print_plugin(path, print_full_path, describe)?);
    }
    Ok(())
}

/// Find a plugin's description
///
/// Given a cannonical plugin path, returns the description included in the `-h` opt.
/// Returns a generic description if a description cannot be found
fn parse_description_for_plugin(plugin: &Path) -> String {
    use std::process::Command;
    let default_description = "No description found for this plugin.";
    let proc = Command::new(plugin)
        .arg("-h")
        .output()
        .expect("Could not get plugin description.");

    let stdout = String::from_utf8_lossy(&proc.stdout);

    // If the plugin doesn't support a -h flag
    match stdout.split('\n').nth(1) {
        Some(x) => {
            if x.is_empty() {
                default_description.to_owned()
            } else {
                x.to_owned()
            }
        }
        None => default_description.to_owned(),
    }
}

/// # Panics
///
/// Format a given plugin's line to stdout
///
/// Formatting is based on a combination of `print_full_path` and `describe`. Will
/// panic if there is a problem retrieving a plugin's name or path
fn format_print_description(
    path: PathBuf,
    print_full_path: bool,
    describe: bool,
) -> Result<String> {
    let display = if print_full_path {
        path.display().to_string()
    } else {
        path.file_name()
            .expect("Failed to read file name")
            .to_str()
            .expect("Failed to print file name")
            .to_string()
    };

    let description = parse_description_for_plugin(&path);

    if describe {
        Ok(format!("  {} \t\t{}", display, description))
    } else {
        Ok(display)
    }
}

/// # Panics
///
/// This function assumes that file names will never be empty since it is only used with
/// paths yielded from plugin::find_all(), as well as that the file names are in valid
/// unicode format since file names should be prefixed with `forc-`. Should one of these 2
/// assumptions fail, this function panics.
fn print_plugin(path: PathBuf, print_full_path: bool, describe: bool) -> Result<String> {
    format_print_description(path, print_full_path, describe)
        .map_err(|e| anyhow!("Could not get plugin info: {}", e))
}
