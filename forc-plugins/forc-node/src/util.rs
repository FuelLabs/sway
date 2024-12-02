use anyhow::{anyhow, bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

use semver::Version;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    pub peer_id: String,
    pub secret: String,
}

/// Checks the local fuel-core's version that `forc-node` will be runnning.
pub(crate) fn get_fuel_core_version() -> anyhow::Result<Version> {
    let version_cmd = Command::new("fuel-core")
        .arg("--version")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run fuel-core, make sure that it is installed.");

    let version_output = String::from_utf8_lossy(&version_cmd.stdout).to_string();

    // Version output is `fuel-core <SEMVER VERSION>`. We should split it to only
    // get the version part of it before parsing as semver.
    let version = version_output
        .split_whitespace()
        .last()
        .ok_or_else(|| anyhow!("fuel-core version parse failed"))?;
    let version_semver = Version::parse(version)?;

    Ok(version_semver)
}

/// Given a `Command`, wrap it to enable generating the actual string that would
/// create this command.
/// Example:
/// ```rust
/// let command = Command::new("fuel-core").arg("run");
/// let command = HumanReadableCommand(command);
/// let formatted = format!("{command}");
/// assert_eq!(&formatted, "fuel-core run");
/// ```
pub struct HumanReadableCommand(Command);

impl Display for HumanReadableCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dbg_out = format!("{:?}", self.0);
        // This is in the ""command-name" "param-name" "param-val"" format.
        let parsed = dbg_out
            .replace("\" \"", " ") // replace " " between items with space
            .replace("\"", ""); // remove remaining quotes at start/end
        write!(f, "{parsed}")
    }
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

impl From<Command> for HumanReadableCommand {
    fn from(value: Command) -> Self {
        Self(value)
    }
}

/// Ask if the user has a keypair generated and if so, collect the details.
/// If not, bails out with a help message about how to generate a keypair.
pub(crate) fn ask_user_keypair() -> Result<KeyPair> {
    let has_keypair = ask_user_yes_no_question("Do you have a keypair in hand?")?;
    if has_keypair {
        // ask the keypair
        let peer_id = ask_user_string("Peer Id:")?;
        let secret = ask_user_discreetly("Secret:")?;
        Ok(KeyPair { peer_id, secret })
    } else {
        bail!("Please create a keypair with `fuel-core-keygen new --key-type peering`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_basic_command() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(format!("{human_readable}"), "fuel-core run");
    }

    #[test]
    fn test_command_with_multiple_args() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        command.arg("--config");
        command.arg("config.toml");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core run --config config.toml"
        );
    }

    #[test]
    fn test_command_no_args() {
        let command = Command::new("fuel-core");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(format!("{human_readable}"), "fuel-core");
    }

    #[test]
    fn test_command_with_path() {
        let mut command = Command::new("fuel-core");
        command.arg("--config");
        command.arg("/path/to/config.toml");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core --config /path/to/config.toml"
        );
    }
}
