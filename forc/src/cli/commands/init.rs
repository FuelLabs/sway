use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project.
#[derive(Debug, Parser)]
pub(crate) struct Command {
    project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let project_name = command.project_name;
    forc_init::init_new_project(project_name).map_err(|e| e)
}
