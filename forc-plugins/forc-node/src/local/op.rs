use super::cmd::LocalCmd;
use crate::{
    pkg::{create_chainconfig_dir, ChainConfig},
    run::{run_mode, Mode},
};

pub(crate) async fn run(cmd: LocalCmd) -> anyhow::Result<()> {
    create_chainconfig_dir(ChainConfig::Local)?;
    let mode = Mode::Local(cmd);
    run_mode(mode).await?;
    Ok(())
}
