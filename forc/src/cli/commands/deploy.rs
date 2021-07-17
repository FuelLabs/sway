use crate::ops::forc_deploy;
use structopt::{self, StructOpt};

/// Deploy contract project.
/// Crafts a contract deployment transaction then sends it to a running node.
#[derive(Debug, StructOpt)]
pub struct Command {}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_deploy::deploy(command).await {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
