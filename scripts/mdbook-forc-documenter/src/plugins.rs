use anyhow::{anyhow, Error};
use std::fs;
use std::path::PathBuf;

fn find_forc_plugins_dir() -> Result<PathBuf, Error> {
    let curr_path_buf = std::env::current_dir().unwrap();
    let mut curr_dir = curr_path_buf.as_path();
    while fs::read_dir(curr_dir)?.any(|f| {
        let file = f.unwrap();
        file.file_name().to_str().unwrap() == "Cargo.toml"
    }) {
        if fs::read_dir(curr_dir).unwrap().any(|f| {
            let file = f.unwrap();
            file.file_type().unwrap().is_dir()
                && file.file_name().to_str().unwrap() == "forc-plugins"
        }) {
            return Ok(curr_dir.join("forc-plugins"));
        }

        curr_dir = curr_path_buf.parent().unwrap();
    }

    Err(anyhow!(
        "Could not find Cargo.toml in the project directory"
    ))
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
