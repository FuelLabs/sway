use crate::cli;
use clap::Parser;

/// Run the Sway unit tests for the current project.
///
/// NOTE: This feature is not yet implemented. Track progress at the following link:
/// https://github.com/FuelLabs/sway/issues/1832
///
/// NOTE: Previously this command was used to support Rust integration testing, however the
/// provided behaviour served no benefit over running `cargo test` directly. The proposal to change
/// the behaviour to support unit testing can be found at the following link:
/// https://github.com/FuelLabs/sway/issues/1833
///
/// Sway unit tests are functions decorated with the `#[test]` attribute. Each test is compiled as
/// a unique entry point for a single program and has access to the namespace of the module in
/// which it is declared.
///
/// Unit tests decorated with the `#[test(script)]` attribute that are declared within `contract`
/// projects may also call directly into their associated contract's ABI.
///
/// Upon successful compilation, test scripts are executed to their completion. A test is
/// considered a failure in the case that a revert (`rvrt`) instruction is encountered during
/// execution. Otherwise, it is considered a success.
#[derive(Debug, Parser)]
pub struct Command {
    #[clap(flatten)]
    pub build: cli::shared::Build,
    /// Enable the incomplete, unstable unit testing support. Warning: May create a black hole!
    ///
    /// This is purely for Sway devs to test the unit testing functionality until it stabilises,
    /// and will be removed upon stabilisation of the command.
    #[clap(long)]
    pub unstable: bool,
    /// When specified, only tests containing the given string will be executed.
    pub filter: Option<String>,
}

pub(crate) fn exec(_cmd: Command) -> anyhow::Result<()> {
    anyhow::bail!(
        r#"
Sway unit testing is not yet implemented. Track progress at the following link:

https://github.com/FuelLabs/sway/issues/1832

NOTE: Previously this command was used to support Rust integration testing,
however the provided behaviour served no benefit over running `cargo test`
directly. The proposal to change the behaviour to support unit testing can be
found at the following link:

https://github.com/FuelLabs/sway/issues/1833
    "#
    );
}
