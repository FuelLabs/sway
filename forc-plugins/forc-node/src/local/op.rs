use anyhow::Context;
use forc_tracing::println_green;

use crate::{
    chain_config::{create_chainconfig_dir, ChainConfig},
    run_opts::{DbType, RunOpts},
    util::HumanReadableCommand,
};
use std::process::{Child, Command};

use super::cmd::LocalCmd;

/// Local is a local node suited for local development.
/// By default, the node is in `debug` mode and the db used is `in-memory`.
/// Returns `None` if this is a dry_run and no child process created for fuel-core.
pub(crate) fn run(cmd: LocalCmd, dry_run: bool) -> anyhow::Result<Option<Child>> {
    create_chainconfig_dir(ChainConfig::Local)
        .context("Failed to create chain config directory")?;

    let run_opts = RunOpts::from(cmd);
    let params = run_opts.generate_params();

    let mut fuel_core_command = Command::new("fuel-core");
    fuel_core_command.arg("run");
    fuel_core_command.args(params.as_slice());

    if dry_run {
        println_green(&format!(
            "{}",
            HumanReadableCommand::from(fuel_core_command)
        ));
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
        Self {
            db_type: DbType::InMemory,
            debug: true,
            snapshot: path,
            poa_instant: true,
            ..Default::default()
        }
    }
}
