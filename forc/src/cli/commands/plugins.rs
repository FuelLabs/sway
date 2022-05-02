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
    /// Prints the long description associated with each listed plugin
    #[clap(long = "describe", short = 'd')]
    describe: bool,
}

pub(crate) fn exec(command: PluginsCommand) -> Result<()> {
    let PluginsCommand {
        print_full_path,
        describe,
    } = command;

    println!("Installed Plugins:");
    for path in crate::cli::plugin::find_all() {
        print_plugin(path, print_full_path, describe);
    }
    Ok(())
}

fn parse_description_for_plugin(plugin: &str) -> String {
    use std::process::Command;
    let proc = Command::new(plugin)
        .arg("-h")
        .output()
        .expect("Could not get plugin description.");

    let stdout = String::from_utf8_lossy(&proc.stdout);
    stdout
        .split('\n')
        .nth(1)
        .unwrap_or("No description found for this plugin.")
        .to_owned()
}

fn format_print_description(path: PathBuf, print_full_path: bool, describe: bool) -> String {
    let display = if print_full_path {
        path.display().to_string()
    } else {
        path.file_name()
            .expect("Failed to read file name")
            .to_str()
            .expect("Failed to print file name")
            .to_string()
    };

    let description = parse_description_for_plugin(&display);

    if describe {
        return format!("  {} \t\t{}", display, description);
    }

    display
}

/// # Panics
///
/// This function assumes that file names will never be empty since it is only used with
/// paths yielded from plugin::find_all(), as well as that the file names are in valid
/// unicode format since file names should be prefixed with `forc-`. Should one of these 2
/// assumptions fail, this function panics.
fn print_plugin(path: PathBuf, print_full_path: bool, describe: bool) {
    println!(
        "{}",
        format_print_description(path, print_full_path, describe)
    )
}
