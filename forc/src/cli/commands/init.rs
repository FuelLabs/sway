use structopt::{self, StructOpt};

use crate::ops::forc_init;
#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    #[structopt(name = "init")]
    project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    let project_name = command.project_name;
    forc_init::init_new_project(project_name).map_err(|e| e.to_string())
}
