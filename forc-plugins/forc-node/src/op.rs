use crate::{
    cmd::{ForcNodeCmd, Mode},
    consts::MIN_FUEL_CORE_VERSION,
};
use anyhow::anyhow;
use forc_util::forc_result_bail;
use semver::Version;
use std::{
    fmt::Display,
    process::{Child, Command, Stdio},
};

/// Checks the local fuel-core's version that `forc-node` will be runnning.
fn get_fuel_core_version() -> anyhow::Result<Version> {
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

impl From<Command> for HumanReadableCommand {
    fn from(value: Command) -> Self {
        Self(value)
    }
}

/// First checks locally installed `forc-node` version and compares it with
/// `consts::MIN_FUEL_CORE_VERSION`. If local version is acceptable, proceeding
/// with the correct mode of operation.
pub(crate) async fn run(cmd: ForcNodeCmd) -> anyhow::Result<Option<Child>> {
    let current_version = get_fuel_core_version()?;
    let supported_min_version = Version::parse(MIN_FUEL_CORE_VERSION)?;
    if current_version < supported_min_version {
        forc_result_bail!(format!(
            "Minimum supported fuel core version is {MIN_FUEL_CORE_VERSION}, system version: {}",
            current_version
        ));
    }
    let forc_node_handle = match cmd.mode {
        Mode::Local(local) => crate::local::op::run(local, cmd.dry_run)?,
        Mode::Testnet(testnet) => crate::testnet::op::run(testnet, cmd.dry_run)?,
        Mode::Ignition(ignition) => crate::ignition::op::run(ignition, cmd.dry_run)?,
    };
    Ok(forc_node_handle)
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
