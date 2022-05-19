use std::fs;
use std::path::PathBuf;

fn get_sway_path() -> PathBuf {
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.ends_with("sway") {
            return curr_path;
        }
        curr_path = curr_path.parent().unwrap().to_path_buf()
    }
}

pub fn plugin_commands() -> Vec<String> {
    let plugins_dir = get_sway_path().join("forc-plugins");
    let mut plugins: Vec<String> = Vec::new();

    for entry in fs::read_dir(&plugins_dir)
        .expect("Failed to read plugins directory")
        .flatten()
    {
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let plugin = name.split_once('-').unwrap().1;
        plugins.push(plugin.to_string());
    }
    plugins
}
