//! Items related to plugin support for `forc`.

use anyhow::{bail, Result};
use regex::Regex;
use semver::Version;
use std::{
    cmp::Ordering,
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
pub(crate) fn execute_external_subcommand(args: Vec<String>) -> Result<process::Output> {
    let cmd = args.get(0).expect("`args` must not be empty");
    let args = &args[1..];
    let path = find_external_subcommand(cmd, std::env::current_exe().ok());
    let command = match path {
        Some(command) => command,
        None => bail!("no such subcommand: `{}`", cmd),
    };
    let output = process::Command::new(command)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .args(args)
        .output()?;
    Ok(output)
}

/// Find the versions of the given plugin's stdout text.
fn get_version_from_text(stdout: &str) -> Option<Version> {
    Regex::new(r"(\d+\.\d+\.\d+)")
        .map(|pattern| {
            pattern
                .captures(stdout)
                .and_then(|captures| captures.get(1))
                .and_then(|version| Version::parse(version.as_str()).ok())
        })
        .ok()
        .flatten()
}

/// Callback to sort a set of versions.
fn sort_versions(a: &Option<Version>, b: &Option<Version>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => b.cmp(a),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Find an exe called `forc-<cmd>` and return its path.
///
/// The algorithm to select a plugin is as follows:
/// 1. If a plugin with the same name as the command exists in the same
///    directory as the forc exe, use it.
/// 2. Find all plugins in the user's `PATH` and select the one with the, sort
///    them based on their versions. No version found is considered to be the
///    oldest version.
/// 3. Use the newer plugin that is found.
fn find_external_subcommand(cmd: &str, current_exec: Option<PathBuf>) -> Option<PathBuf> {
    let command_exe = format!("forc-{}{}", cmd, env::consts::EXE_SUFFIX);
    if let Some(path) = current_exec {
        if let Some(parent) = path.parent() {
            let command_full_path = parent.join(&command_exe);
            if is_executable(&command_full_path) {
                return Some(command_full_path);
            }
        }
    }

    let mut candidates = search_directories()
        .into_iter()
        .map(|dir| dir.join(&command_exe))
        .filter(|path| is_executable(path))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|path| {
            use std::process::Command;
            (
                Command::new(&path)
                    .arg("--version")
                    .output()
                    .map(|output| get_version_from_text(&String::from_utf8_lossy(&output.stdout)))
                    .ok()
                    .flatten(),
                path,
            )
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|(a, _), (b, _)| sort_versions(a, b));

    candidates.get(0).map(|(_, path)| path.to_owned())
}

/// Search the user's `PATH` for `forc-*` exes.
fn search_directories() -> Vec<PathBuf> {
    if let Some(val) = env::var_os("PATH") {
        return env::split_paths(&val).collect();
    }
    vec![]
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
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
        .map(|entry| entry.path().to_path_buf())
        .filter(|p| is_plugin(p))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_parsing() {
        let stdout = "forc 0.1.0";
        let version = get_version_from_text(stdout);
        assert_eq!(version, Version::parse("0.1.0").ok());

        let stdout = "forc 0.1";
        let version = get_version_from_text(stdout);
        assert_eq!(version, None);

        let stdout =
            "forc with some long text and having the version (0.1.11) somewhere in the middle";
        let version = get_version_from_text(stdout);
        assert_eq!(version, Version::parse("0.1.11").ok());
    }

    #[test]
    fn sort_version() {
        let mut versions = vec![
            Version::parse("1.9.0").ok(),
            Version::parse("1.9.1").ok(),
            Version::parse("1.9.99").ok(),
            Version::parse("1.19.1").ok(),
            Version::parse("9.19.1").ok(),
            None,
        ];
        versions.sort_by(sort_versions);

        assert_eq!(
            versions,
            vec![
                Version::parse("9.19.1").ok(),
                Version::parse("1.19.1").ok(),
                Version::parse("1.9.99").ok(),
                Version::parse("1.9.1").ok(),
                Version::parse("1.9.0").ok(),
                None,
            ]
        )
    }
}
