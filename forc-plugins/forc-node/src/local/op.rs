use super::cmd::LocalCmd;
use crate::{
    chain_config::{check_and_update_chain_config, ChainConfig},
    run_opts::{DbType, RunOpts},
    util::HumanReadableCommand,
};
use anyhow::Context;
use forc_tracing::println_green;
use std::process::{Child, Command};

/// Local is a local node suited for local development.
/// By default, the node is in `debug` mode and the db used is `in-memory`.
/// Returns `None` if this is a dry_run and no child process created for fuel-core.
pub(crate) async fn run(cmd: LocalCmd, dry_run: bool) -> anyhow::Result<Option<Child>> {
    check_and_update_chain_config(ChainConfig::Local).await?;

    let run_opts = RunOpts::from(cmd);
    let params = run_opts.generate_params();

    let mut fuel_core_command = Command::new("fuel-core");
    fuel_core_command.arg("run");
    fuel_core_command.args(params.as_slice());

    println_green(&format!(
        "{}",
        HumanReadableCommand::from(&fuel_core_command)
    ));

    if dry_run {
        Ok(None)
    } else {
        // Spawn the process with proper error handling
        let handle = fuel_core_command
            .spawn()
            .with_context(|| "Failed to spawn fuel-core process:".to_string())?;

        Ok(Some(handle))
    }
}

impl From<LocalCmd> for RunOpts {
    fn from(value: LocalCmd) -> Self {
        let path = value
            .chain_config
            .unwrap_or_else(|| ChainConfig::Local.into());
        let db_type = value
            .db_path
            .as_ref()
            .map_or(DbType::InMemory, |_| DbType::RocksDb);
        Self {
            db_type,
            debug: true,
            snapshot: path,
            poa_instant: true,
            db_path: value.db_path,
            port: value.port,
            ..Default::default()
        }
    }
}
