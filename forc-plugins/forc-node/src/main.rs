//! A forc plugin to start a fuel core instance, preconfigured for generic
//! usecases.
mod cmd;
mod consts;
mod ignition;
mod local;
mod op;
mod pkg;
mod run_opts;
mod testnet;

use anyhow::anyhow;
use clap::Parser;
use forc_tracing::init_tracing_subscriber;
use forc_util::{forc_result_bail, ForcResult};

#[tokio::main]
async fn main() -> ForcResult<()> {
    init_tracing_subscriber(Default::default());

    let command = cmd::ForcNodeCmd::parse();

    let mut handle = match op::run(command).await {
        Ok(handler) => handler,
        Err(err) => {
            forc_result_bail!(err);
        }
    };

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
