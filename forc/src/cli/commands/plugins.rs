use crate::cli::PluginsCommand;
use anyhow::anyhow;
use clap::Parser;
use forc_diagnostic::println_warning;
use forc_types::ForcResult;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::info;

forc_types::cli_examples! {
    crate::cli::Opt {
        [ List all plugins => "forc plugins" ]
        [ List all plugins with their paths => "forc plugins --paths" ]
        [ List all plugins with their descriptions => "forc plugins --describe" ]
        [ List all plugins with their paths and descriptions => "forc plugins --paths --describe" ]
    }
}

/// Find all forc plugins available via `PATH`.
///
/// Prints information about each discovered plugin.
#[derive(Debug, Parser)]
#[clap(name = "forc plugins", about = "List all forc plugins", version, after_help = help())]
pub struct Command {
    /// Prints the absolute path to each discovered plugin.
    #[clap(long = "paths", short = 'p')]
    print_full_path: bool,
    /// Prints the long description associated with each listed plugin
    #[clap(long = "describe", short = 'd')]
    describe: bool,
}

fn get_file_name(path: &Path) -> String {
    if let Some(path_str) = path.file_name().and_then(|path_str| path_str.to_str()) {
        path_str.to_owned()
    } else {
        path.display().to_string()
    }
}

pub(crate) fn exec(command: PluginsCommand) -> ForcResult<()> {
    let PluginsCommand {
        print_full_path,
        describe,
    } = command;

    let mut plugins = crate::cli::plugin::find_all()
        .map(|path| {
            get_plugin_info(path.clone(), print_full_path, describe).map(|info| (path, info))
        })
        .collect::<Result<Vec<(_, _)>, _>>()?
        .into_iter()
        .fold(HashMap::new(), |mut acc, (path, content)| {
            let bin_name = get_file_name(&path);
            acc.entry(bin_name.clone())
                .or_insert_with(|| (bin_name, vec![], content.clone()))
                .1
                .push(path);
            acc
        })
        .into_values()
        .map(|(bin_name, mut paths, content)| {
            paths.sort();
            paths.dedup();
            (bin_name, paths, content)
        })
        .collect::<Vec<_>>();
    plugins.sort_by(|a, b| a.0.cmp(&b.0));

    info!("Installed Plugins:");
    for plugin in plugins {
        info!("{}", plugin.2);
        if plugin.1.len() > 1 {
            println_warning(&format!("Multiple paths found for {}", plugin.0));
            for path in plugin.1 {
                println_warning(&format!("   {}", path.display()));
            }
        }
    }
    Ok(())
}

/// Find a plugin's description
///
/// Given a canonical plugin path, returns the description included in the `-h` opt.
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
) -> ForcResult<String> {
    let display = if print_full_path {
        path.display().to_string()
    } else {
        get_file_name(&path)
    };

    let description = parse_description_for_plugin(&path);

    if describe {
        Ok(format!("  {display} \t\t{description}"))
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
fn get_plugin_info(path: PathBuf, print_full_path: bool, describe: bool) -> ForcResult<String> {
    format_print_description(path, print_full_path, describe)
        .map_err(|e| anyhow!("Could not get plugin info: {}", e.as_ref()).into())
}
