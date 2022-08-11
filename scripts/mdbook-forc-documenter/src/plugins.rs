use anyhow::{bail, Result};
use std::{collections::BTreeMap, fs, path::PathBuf};

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

#[derive(Debug)]
pub(crate) enum PluginCommand {
    /// The plugin has a single command. Its name matches the plugin.
    Single,
    /// The plugin installs a group of commands with the given names.
    Group(Vec<String>),
}

/// Returns all plugin commands
pub(crate) fn get_all_plugins(plugin_map: &mut BTreeMap<String, PluginCommand>) -> Vec<String> {
    let mut single_plugins = Vec::new();
    for (plugin_name, plugin_type) in plugin_map.iter_mut() {
        if let PluginCommand::Group(child_plugins) = plugin_type {
            single_plugins.append(child_plugins);
        } else {
            single_plugins.push(plugin_name.clone());
        }
    }
    single_plugins
}

/// The list of plugins alongside their associated commands.
pub(crate) fn plugin_commands() -> BTreeMap<String, PluginCommand> {
    let plugins_dir = find_forc_plugins_dir().expect("Failed to find plugins directory");
    let mut plugins: BTreeMap<String, PluginCommand> = BTreeMap::new();

    for entry in fs::read_dir(&plugins_dir)
        .expect("Failed to read plugins directory")
        .flatten()
    {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();

            if name.starts_with("forc-") {
                let plugin = name.split_once('-').unwrap().1;
                // Check for child plugins (like `forc-deploy` and `forc-run` of `forc-client`)
                let child_plugins = collect_child_plugins(&path.join("Cargo.toml")).unwrap();
                if child_plugins.is_empty() {
                    plugins.insert(plugin.to_string(), PluginCommand::Single);
                } else {
                    let mut child_plugin_names = Vec::new();
                    for child_plugin in child_plugins {
                        let plugin = child_plugin.split_once('-').unwrap().1;
                        child_plugin_names.push(plugin.to_string());
                    }
                    plugins.insert(
                        format!("forc {}", plugin),
                        PluginCommand::Group(child_plugin_names),
                    );
                }
            }
        }
    }
    plugins
}

/// Collects child plugins for a given plugin's Cargo.toml path
fn collect_child_plugins(manifest_path: &PathBuf) -> Result<Vec<String>> {
    let mut child_plugins = Vec::new();
    let forc_toml: toml::Value = toml::de::from_str(&fs::read_to_string(manifest_path)?)?;
    if let Some(table) = forc_toml.as_table() {
        if let Some(values) = table.get("bin").and_then(|bin| bin.as_array()) {
            for value in values {
                if let Some(name) = value.as_table().and_then(|table| table.get("name")) {
                    let name_str = name.to_string();
                    let mut name = name_str.chars();
                    name.next();
                    name.next_back();
                    child_plugins.push(String::from(name.as_str()));
                }
            }
        }
    }
    Ok(child_plugins)
}
