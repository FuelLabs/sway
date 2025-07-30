use crate::consts::{
    DB_FOLDER, IGNITION_CONFIG_FOLDER_NAME, LOCAL_CONFIG_FOLDER_NAME, TESTNET_CONFIG_FOLDER_NAME,
};
use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use forc_util::user_forc_directory;
use fuel_crypto::{
    rand::{prelude::StdRng, SeedableRng},
    SecretKey,
};
use libp2p_identity::{secp256k1, Keypair, PeerId};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::PathBuf,
    process::{Command, Stdio},
};
use std::{
    io::{Read, Write},
    ops::Deref,
};

pub enum DbConfig {
    Local,
    Testnet,
    Ignition,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    pub peer_id: String,
    pub secret: String,
}

/// Given a `Command`, wrap it to enable generating the actual string that would
/// create this command.
/// Example:
/// ```rust
/// use std::process::Command;
/// use forc_node::util::HumanReadableCommand;
///
/// let mut command = Command::new("fuel-core");
/// command.arg("run");
/// let command = HumanReadableCommand::from(&command);
/// let formatted = format!("{command}");
/// assert_eq!(&formatted, "fuel-core run");
/// ```
pub struct HumanReadableCommand<'a>(&'a Command);

impl From<DbConfig> for PathBuf {
    fn from(value: DbConfig) -> Self {
        let user_db_dir = user_forc_directory().join(DB_FOLDER);
        match value {
            DbConfig::Local => user_db_dir.join(LOCAL_CONFIG_FOLDER_NAME),
            DbConfig::Testnet => user_db_dir.join(TESTNET_CONFIG_FOLDER_NAME),
            DbConfig::Ignition => user_db_dir.join(IGNITION_CONFIG_FOLDER_NAME),
        }
    }
}

impl Display for HumanReadableCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dbg_out = format!("{:?}", self.0);
        // This is in the ""command-name" "param-name" "param-val"" format.
        let parsed = dbg_out
            .replace("\" \"", " ") // replace " " between items with space
            .replace("\"", ""); // remove remaining quotes at start/end
        write!(f, "{parsed}")
    }
}

impl<'a> From<&'a Command> for HumanReadableCommand<'a> {
    fn from(value: &'a Command) -> Self {
        Self(value)
    }
}

impl KeyPair {
    pub fn random() -> Self {
        let mut rng = StdRng::from_entropy();
        let secret = SecretKey::random(&mut rng);

        let mut bytes = *secret.deref();
        let p2p_secret = secp256k1::SecretKey::try_from_bytes(&mut bytes)
            .expect("Should be a valid private key");
        let p2p_keypair = secp256k1::Keypair::from(p2p_secret);
        let libp2p_keypair = Keypair::from(p2p_keypair);
        let peer_id = PeerId::from_public_key(&libp2p_keypair.public());
        Self {
            peer_id: format!("{peer_id}"),
            secret: format!("{secret}"),
        }
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

/// Print a string to an alternate screen, so the string isn't printed to the terminal.
pub(crate) fn display_string_discreetly(
    discreet_string: &str,
    continue_message: &str,
) -> Result<()> {
    use termion::screen::IntoAlternateScreen;
    let mut screen = std::io::stdout().into_alternate_screen()?;
    writeln!(screen, "{discreet_string}")?;
    screen.flush()?;
    println!("{continue_message}");
    wait_for_keypress();
    Ok(())
}

pub(crate) fn wait_for_keypress() {
    let mut single_key = [0u8];
    std::io::stdin().read_exact(&mut single_key).unwrap();
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
        println!("Generating new keypair...");
        let pair = KeyPair::random();
        display_string_discreetly(
            &format!(
                "Generated keypair:\n PeerID: {}, secret: {}",
                pair.peer_id, pair.secret
            ),
            "### Do not share or lose this private key! Press any key to complete. ###",
        )?;
        Ok(pair)
    }
}

/// Checks the local fuel-core's version that `forc-node` will be running.
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

#[cfg(unix)]
pub fn check_open_fds_limit(max_files: u64) -> Result<(), Box<dyn std::error::Error>> {
    use std::mem;

    unsafe {
        let mut fd_limit = mem::zeroed();
        let mut err = libc::getrlimit(libc::RLIMIT_NOFILE, &mut fd_limit);
        if err != 0 {
            return Err("check_open_fds_limit failed".into());
        }
        if fd_limit.rlim_cur >= max_files {
            return Ok(());
        }

        let prev_limit = fd_limit.rlim_cur;
        fd_limit.rlim_cur = max_files;
        if fd_limit.rlim_max < max_files {
            // If the process is not started by privileged user, this will fail.
            fd_limit.rlim_max = max_files;
        }
        err = libc::setrlimit(libc::RLIMIT_NOFILE, &fd_limit);
        if err == 0 {
            return Ok(());
        }
        Err(format!(
            "the maximum number of open file descriptors is too \
             small, got {}, expect greater or equal to {}",
            prev_limit, max_files
        )
        .into())
    }
}

#[cfg(not(unix))]
pub fn check_open_fds_limit(_max_files: u64) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_basic_command() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        let human_readable = HumanReadableCommand(&command);
        assert_eq!(format!("{human_readable}"), "fuel-core run");
    }

    #[test]
    fn test_command_with_multiple_args() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        command.arg("--config");
        command.arg("config.toml");
        let human_readable = HumanReadableCommand(&command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core run --config config.toml"
        );
    }

    #[test]
    fn test_command_no_args() {
        let command = Command::new("fuel-core");
        let human_readable = HumanReadableCommand(&command);
        assert_eq!(format!("{human_readable}"), "fuel-core");
    }

    #[test]
    fn test_command_with_path() {
        let mut command = Command::new("fuel-core");
        command.arg("--config");
        command.arg("/path/to/config.toml");
        let human_readable = HumanReadableCommand(&command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core --config /path/to/config.toml"
        );
    }
}
