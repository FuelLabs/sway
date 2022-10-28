use anyhow::{anyhow, Result};
use std::{collections::HashSet, process};

pub(crate) fn forc_plugins_from_path() -> Result<Vec<String>> {
    let output = process::Command::new("forc")
        .arg("plugins")
        .output()
        .expect("Failed running forc plugins");
    let mut plugins = HashSet::new();

    if !output.status.success() {
        return Err(anyhow!("Failed to run forc plugins"));
    }

    let s = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);

    for plugin in s.lines() {
        if let Some(command) = plugin.split_once("forc-") {
            plugins.insert(command.1.to_string());
        }
    }

    Ok(Vec::from_iter(plugins))
}
