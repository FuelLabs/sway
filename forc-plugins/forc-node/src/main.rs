//! A forc plugin to start a fuel core instance, preconfigured for generic
//! usecases.
use anyhow::anyhow;
use clap::Parser;
use forc_node::{
    cmd::{self, ForcNodeCmd},
    op,
};
use forc_tracing::init_tracing_subscriber;
use forc_util::{ForcCliResult, ForcResult};

async fn run(cmd: ForcNodeCmd) -> ForcResult<()> {
    let mut handle = op::run(cmd).await?;

    // if this is not a dry run we should wait for the kill signal and kill
    // fuel-core upon receiving it.
    if let Some(handle) = &mut handle {
        // Wait for the kill signal, if that comes we should kill child fuel-core
        // process.
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| anyhow!("Failed to listen for ctrl-c: {e}"))?;

        handle.kill()?;
    }
    ForcResult::Ok(())
}

#[tokio::main]
async fn main() -> ForcCliResult<()> {
    init_tracing_subscriber(Default::default());

    let command = cmd::ForcNodeCmd::parse();
    run(command).await.into()
}
