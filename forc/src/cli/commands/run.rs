use crate::ops::forc_run;
use structopt::{self, StructOpt};

/// Run script project.
/// Crafts a script transaction then sends it to a running node.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Hex string of data to input to script.
    #[structopt(short, long)]
    pub data: Option<String>,

    /// Path to the project, if not specified, current working directory will be used.
    #[structopt(short, long)]
    pub path: Option<String>,

    /// Only craft transaction and print it out.
    #[structopt(long)]
    pub dry_run: bool,

    /// URL of the Fuel Client Node
    #[structopt(env = "FUEL_NODE_URL", default_value = "127.0.0.1:4000")]
    pub node_url: String,

    /// Kill Fuel Node Client after running the code.
    /// This is only available if the node is started from `forc run`
    #[structopt(short, long)]
    pub kill_node: bool,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_run::run(command).await {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
