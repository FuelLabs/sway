//! Items related to plugin support for `forc`.

use anyhow::{bail, Result};
use forc_tracing::println_warning_verbose;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

/// Attempt to execute the unknown subcommand as an external plugin.
///
/// The subcommand is assumed to be the first element, with the following elements representing
/// following arguments to the external subcommand.
///
/// E.g. given `foo bar baz` where `foo` is an unrecognized subcommand to `forc`, tries to execute
/// `forc-foo bar baz`.
pub(crate) fn execute_external_subcommand(args: &[String]) -> Result<process::Output> {
    let cmd = args
        .first()
        .ok_or_else(|| anyhow::anyhow!("no subcommand provided"))?;
    let args = &args[1..];
    let path = find_external_subcommand(cmd);
    let command = match path {
        Some(command) => command,
        None => bail!("no such subcommand: `{}`", cmd),
    };

    if let Ok(forc_path) = std::env::current_exe() {
        if command.parent() != forc_path.parent() {
            println_warning_verbose(&format!(
                "The {} ({}) plugin is in a different directory than forc ({})\n",
                cmd,
                command.display(),
                forc_path.display(),
            ));
        }
    }

    let output = process::Command::new(command)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .args(args)
        .output()?;
    Ok(output)
}

/// Find an exe called `forc-<cmd>` and return its path.
fn find_external_subcommand(cmd: &str) -> Option<PathBuf> {
    let command_exe = format!("forc-{}{}", cmd, env::consts::EXE_SUFFIX);
    search_directories()
        .iter()
        .map(|dir| dir.join(&command_exe))
        .find(|file| is_executable(file))
}

/// Search the user's `PATH` for `forc-*` exes.
fn search_directories() -> Vec<PathBuf> {
    if let Some(val) = env::var_os("PATH") {
        return env::split_paths(&val).collect();
    }
    vec![]
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Whether or not the given path points to a valid forc plugin.
fn is_plugin(path: &Path) -> bool {
    if let Some(stem) = path.file_name().and_then(|os_str| os_str.to_str()) {
        if stem.starts_with("forc-") && is_executable(path) {
            return true;
        }
    }
    false
}

/// Find all forc plugins available via `PATH`.
pub(crate) fn find_all() -> impl Iterator<Item = PathBuf> {
    search_directories()
        .into_iter()
        .flat_map(walkdir::WalkDir::new)
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            is_plugin(&path).then_some(path)
        })
}
