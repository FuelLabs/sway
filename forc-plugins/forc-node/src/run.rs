use clap::Parser;
use fuel_core_bin::cli::run::Command as RunCmd;
use std::{path::PathBuf, str::FromStr};

use crate::{
    local::cmd::LocalCmd,
    pkg::ChainConfig,
    testnet::{cmd::TestnetCmd, op::TestnetOpts},
};

#[derive(Debug, Clone)]
pub struct RunOpts {
    command: Box<RunCmd>,
}

impl Default for RunOpts {
    fn default() -> Self {
        let default_input = vec![""];
        let command = Box::new(RunCmd::parse_from(default_input));
        Self { command }
    }
}

impl FromStr for RunOpts {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.split(" ");
        let command = Box::new(RunCmd::parse_from(s));

        let run_opts = RunOpts { command };
        Ok(run_opts)
    }
}

/// Supported mode of operations by `forc-node`.
#[derive(Debug)]
pub enum Mode {
    /// Local is a local node suited for local development.
    /// By default, the node is in `debug` mode and the db used is `in-memory`.
    Local(LocalCmd),
    /// Testnet is the configuration to connect the node to latest testnet.
    Testnet(TestnetOpts),
    /// Custom is basically equivalent to running `fuel-core run` with some params.
    Custom(RunOpts),
}

impl From<Mode> for RunOpts {
    fn from(value: Mode) -> Self {
        let run_cmd = match value {
            Mode::Local(local_cmd) => {
                let mut run_cmd = vec!["--db-type in-memory".to_string(), "--debug".to_string()];
                let path = local_cmd
                    .chain_config
                    .map(|path| format!("{}", path.display()))
                    .unwrap_or_else(|| {
                        let path: PathBuf = ChainConfig::Local.into();
                        format!("{}", path.display())
                    });
                run_cmd.push("--snapshot".to_string());
                run_cmd.push(path);
                Box::new(RunCmd::parse_from(run_cmd))
            }
            Mode::Testnet(testnet_cmd) => {
                const SERVICE_NAME: &str = "fuel-sepolia-testnet-node";
                let mut run_cmd = vec!["--peer-id".to_string()];
                /*
                let mut opts = RunOpts::default();
                opts.command.service_name = SERVICE_NAME.to_string();
                opts.command.graphql.ip = IpAddr::from_str("0.0.0.0").unwrap();
                if let Some(port) = testnet_cmd.port {
                    opts.command.graphql.port = port;
                }
                testnet_cmd.and_then(|keypair_arg| opts.command.p2p_args.keypair = keypair_arg);
                opts.command
                */
                todo!()
            }
            Mode::Custom(cmd) => cmd.command,
        };
        Self { command: run_cmd }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Custom(RunOpts::default())
    }
}

pub async fn run_mode(mode: Mode) -> anyhow::Result<()> {
    let opts: RunOpts = mode.into();
    fuel_core_bin::cli::run::exec(*opts.command).await?;
    Ok(())
}
