use crate::ops::forc_clean;
use structopt::{self, StructOpt};

/// Removes the default forc compiler output artifact directory, i.e. `<project-name>/out`. Also
/// calls `cargo clean` which removes the `target` directory generated by `cargo` when running
/// tests.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,
}

pub fn exec(command: Command) -> Result<(), String> {
    forc_clean::clean(command)
}
