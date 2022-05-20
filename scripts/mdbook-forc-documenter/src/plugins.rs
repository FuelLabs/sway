use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

fn find_forc_plugins_dir() -> Result<PathBuf> {
    let mut curr_path = std::env::current_dir().unwrap();

    loop {
        if let Ok(entries) = fs::read_dir(&curr_path) {
            if !curr_path.join("Cargo.toml").exists() {
                continue;
            }
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() && path.ends_with("forc-plugins") {
                    return Ok(path);
                }
            }
        }
        curr_path = curr_path
            .parent()
            .ok_or_else(|| anyhow!("Could not find Cargo.toml in the project directory"))?
            .to_path_buf();
    }
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
