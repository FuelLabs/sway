use crate::{ignition::cmd::IgnitionCmd, local::cmd::LocalCmd, testnet::cmd::TestnetCmd};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};

#[derive(Debug, Parser)]
#[clap(name = "forc node", version)]
/// Forc node is a wrapper around fuel-core with sensible defaults to provide
/// easy way of bootstrapping a node for local development, testnet or mainnet.
pub struct ForcNodeCmd {
    /// Instead of directly running the fuel-core instance print the command.
    #[arg(long)]
    pub dry_run: bool,
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Start a local node for development purposes.
    Local(LocalCmd),
    /// Starts a node that will connect to latest testnet.
    Testnet(TestnetCmd),
    /// Starts a node that will connect to ignition network.
    Ignition(IgnitionCmd),
}

pub(crate) fn ask_user_yes_no_question(question: &str) -> anyhow::Result<bool> {
    let answer = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(false)
        .show_default(false)
        .interact()?;
    Ok(answer)
}

pub(crate) fn ask_user_discreetly(question: &str) -> anyhow::Result<String> {
    let discrete = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact()?;
    Ok(discrete)
}

pub(crate) fn ask_user_string(question: &str) -> anyhow::Result<String> {
    let response = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact_text()?;
    Ok(response)
}
