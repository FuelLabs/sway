use anyhow::{bail, Result};
use std::fs;
use std::path::PathBuf;

fn find_forc_plugins_dir() -> Result<PathBuf> {
    let sway_dir = crate::find_sway_repo_root()?;
    let plugins_dir = sway_dir.join("forc-plugins");
    if !plugins_dir.exists() || !plugins_dir.is_dir() {
        bail!(
            "Failed to find plugins directory at {}",
            plugins_dir.display()
        );
    }
    Ok(plugins_dir)
}

pub fn plugin_commands() -> Vec<String> {
    let plugins_dir = find_forc_plugins_dir().expect("Failed to find plugins directory");
    let mut plugins: Vec<String> = Vec::new();

    for entry in fs::read_dir(&plugins_dir)
        .expect("Failed to read plugins directory")
        .flatten()
    {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();

            if name.starts_with("forc-") {
                let plugin = name.split_once('-').unwrap().1;
                plugins.push(plugin.to_string());
            }
        }
    }
    plugins
}
