//! The command line interface for `forc migrate`.
mod commands;
mod shared;

use anyhow::Result;
use clap::{Parser, Subcommand};
use forc_diagnostic::{init_tracing_subscriber, LevelFilter, TracingSubscriberOptions};

use self::commands::{check, run, show};

use check::Command as CheckCommand;
use run::Command as RunCommand;
use show::Command as ShowCommand;

fn help() -> &'static str {
    Box::leak(
        format!(
            "Examples:\n{}{}{}",
            show::examples(),
            check::examples(),
            run::examples(),
        )
        .trim_end()
        .to_string()
        .into_boxed_str(),
    )
}

/// Forc plugin for migrating Sway projects to the next breaking change version of Sway.
#[derive(Debug, Parser)]
#[clap(
    name = "forc-migrate",
    after_help = help(),
    version
)]
pub(crate) struct Opt {
    /// The command to run
    #[clap(subcommand)]
    command: ForcMigrate,
}

impl Opt {
    fn silent(&self) -> bool {
        match &self.command {
            ForcMigrate::Show(_) => true,
            ForcMigrate::Check(command) => command.check.silent,
            ForcMigrate::Run(command) => command.run.silent,
        }
    }
}

#[derive(Subcommand, Debug)]
enum ForcMigrate {
    Show(ShowCommand),
    Check(CheckCommand),
    Run(RunCommand),
}

pub fn run_cli() -> Result<()> {
    let opt = Opt::parse();

    let tracing_options = TracingSubscriberOptions {
        silent: Some(opt.silent()),
        log_level: Some(LevelFilter::INFO),
        ..Default::default()
    };

    init_tracing_subscriber(tracing_options);

    match opt.command {
        ForcMigrate::Show(command) => show::exec(command),
        ForcMigrate::Check(command) => check::exec(command),
        ForcMigrate::Run(command) => run::exec(command),
    }
}
