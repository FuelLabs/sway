use anyhow::{anyhow, Result};
use std::{collections::HashSet, process};

/// Detects plugins available via `PATH`.
///
/// Note that plugin discovery works reliably within the Sway CI since the installed plugins are in
/// a controlled environment. Building the book locally on your own machine may create a different
/// book depending on what plugins are available on your PATH.
pub(crate) fn forc_plugins_from_path() -> Result<Vec<String>> {
    let output = process::Command::new("forc")
        .arg("plugins")
        .output()
        .expect("Failed running forc plugins");

    if !output.status.success() {
        return Err(anyhow!("Failed to run forc plugins"));
    }

    let mut plugins = HashSet::new();
    let s = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);

    for plugin in s.lines() {
        if let Some(("", command)) = plugin.split_once("forc-") {
            plugins.insert(command.to_string());
        }
    }

    Ok(Vec::from_iter(plugins))
}
