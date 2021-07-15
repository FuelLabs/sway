use crate::ops::forc_run;
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub struct Command {
    #[structopt(short = "d", long = "data")]
    pub data: Option<String>,

    #[structopt(short = "p", long = "path", default_value = "./")]
    pub path: String,

    #[structopt(long = "dry-run")]
    pub dry_run: bool,
}

pub(crate) async fn exec(command: Command) -> Result<(), String> {
    match forc_run::run(command).await {
        Err(e) => Err(e.message),
        _ => Ok(()),
    }
}
