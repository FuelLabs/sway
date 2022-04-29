use crate::cli::ListCommand;
use crate::utils::plugin_descriptions::plugin_description;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;


#[derive(Debug, Parser)]
pub struct Command {}

pub(crate) fn exec(command: ListCommand) -> Result<()> {
    let ListCommand {} = command;

    for path in crate::cli::plugin::find_all() {
        print_plugin_and_description(path);
    }
    Ok(())
}

fn print_plugin_and_description(path: PathBuf) {
    let plugin_name = path
        .file_name()
        .expect("Failed to read file name")
        .to_str()
        .expect("Failed to print file name");

    let description = plugin_description(plugin_name);

    println!("{}\t\t{}", plugin_name, description);
}
