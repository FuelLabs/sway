use crate::formatter::format_index_entry;

use anyhow::{anyhow, bail};
use commands::{
    get_contents_from_commands, get_forc_command_from_file_name, possible_forc_commands,
};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use plugins::forc_plugins_from_path;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod commands;
mod formatter;
mod plugins;

#[derive(Default)]
pub struct ForcDocumenter;

impl ForcDocumenter {
    pub fn new() -> ForcDocumenter {
        ForcDocumenter
    }
}

impl Preprocessor for ForcDocumenter {
    fn name(&self) -> &str {
        "forc-documenter"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let strict = ctx
            .config
            .get_preprocessor(self.name())
            .and_then(|t| t.get("strict"))
            .and_then(|val| val.as_bool())
            .unwrap_or(false);

        let possible_commands: Vec<String> = possible_forc_commands();
        let examples: HashMap<String, String> = load_examples()?;
        let plugin_commands = forc_plugins_from_path()?;
        let mut command_contents: HashMap<String, String> =
            get_contents_from_commands(&possible_commands);
        let mut plugin_contents: HashMap<String, String> =
            get_contents_from_commands(&plugin_commands);

        let mut removed_commands = Vec::new();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if chapter.name == "Plugins" {
                    for sub_item in &mut chapter.sub_items {
                        if let BookItem::Chapter(ref mut plugin_chapter) = sub_item {
                            if let Some(content) = plugin_contents.remove(&plugin_chapter.name) {
                                inject_content(plugin_chapter, &content, &examples);
                            } else {
                                // When sub_items exist, it means that a plugin installs a group of
                                // commands, and the name of the plugin will not match this group.
                                // Note that this is determined by SUMMARY.md by placing
                                // sub-chapters under a chapter.
                                if plugin_chapter.sub_items.is_empty() {
                                    removed_commands.push(plugin_chapter.name.clone());
                                }
                            };
                            for sub_sub_item in &mut plugin_chapter.sub_items {
                                if let BookItem::Chapter(ref mut plugin_sub_chapter) = sub_sub_item
                                {
                                    // Skip validation for nested documentation entries
                                    // These are documentation-only entries that don't correspond to actual commands
                                    if let Some(content) =
                                        plugin_contents.remove(&plugin_sub_chapter.name)
                                    {
                                        inject_content(plugin_sub_chapter, &content, &examples);
                                    }
                                    // Don't mark nested entries as removed - they're documentation sections
                                }
                            }
                        }
                    }
                }
                if chapter.name == "Commands" {
                    let mut command_index_content = String::new();
                    for sub_item in &mut chapter.sub_items {
                        if let BookItem::Chapter(ref mut command_chapter) = sub_item {
                            if let Some(content) = command_contents.remove(&command_chapter.name) {
                                command_index_content
                                    .push_str(&format_index_entry(&command_chapter.name));
                                inject_content(command_chapter, &content, &examples);
                            } else {
                                removed_commands.push(command_chapter.name.clone());
                            };
                        }
                    }
                    chapter.content.push_str(&command_index_content);
                }
            }
        });

        let mut error_message = String::new();

        if !command_contents.is_empty() || !plugin_contents.is_empty() {
            let mut missing: Vec<String> = command_contents.keys().cloned().collect();
            missing.append(&mut plugin_contents.keys().cloned().collect());
            error_message.push_str(&missing_entries_msg(&missing));
        };

        if !removed_commands.is_empty() {
            error_message.push_str(&dangling_chapters_msg(&removed_commands));
        };

        if strict && !error_message.is_empty() {
            Err(Error::msg(error_message))
        } else {
            if !error_message.is_empty() {
                eprintln!("Warning:");
                eprintln!("{error_message}");
                eprintln!("The book built successfully - if the changes above were intended or if you are editing pages unrelated to Forc, you may ignore this message.");
            }
            Ok(book)
        }
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn inject_content(chapter: &mut Chapter, content: &str, examples: &HashMap<String, String>) {
    chapter.content = content.to_string();

    if let Some(example_content) = examples.get(&chapter.name) {
        chapter.content += example_content;
    }
}

fn missing_entries_msg(missing: &[String]) -> String {
    let missing_commands = missing
        .iter()
        .map(|s| s.to_owned() + "\n")
        .collect::<String>();
    let missing_entries: String = missing.iter().map(|c| format_index_entry(c)).collect();

    format!("\nSome entries were missing from SUMMARY.md:\n\n{missing_commands}\n\nTo fix this, add the above entries under the Commands or Plugins chapter in SUMMARY.md, like so:\n\n{missing_entries}\n")
}

fn dangling_chapters_msg(commands: &[String]) -> String {
    format!("\nSome commands/plugins were removed from the Forc toolchain, but still exist in SUMMARY.md:\n\n{}\n\nTo fix this, remove the corresponding entries from SUMMARY.md.\n",
        commands
        .iter()
        .map(|s| s.to_owned() + "\n")
        .collect::<String>())
}

fn find_sway_repo_root() -> anyhow::Result<PathBuf> {
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.is_dir() {
            // Some heuristics that should pass if we've found the sway repo.
            if curr_path.join("Cargo.toml").exists()
                && curr_path.join("forc-plugins").exists()
                && curr_path
                    .join("scripts")
                    .join("mdbook-forc-documenter")
                    .exists()
            {
                return Ok(curr_path);
            }
        }
        curr_path = curr_path
            .parent()
            .ok_or_else(|| anyhow!("Could not find Sway repo root directory"))?
            .to_path_buf();
    }
}

fn find_forc_cmd_examples_dir() -> anyhow::Result<PathBuf> {
    let sway_dir = find_sway_repo_root()?;
    let examples_dir = sway_dir
        .join("scripts")
        .join("mdbook-forc-documenter")
        .join("examples");
    if !examples_dir.exists() || !examples_dir.is_dir() {
        bail!(
            "Failed to find examples directory at {}",
            examples_dir.display()
        );
    }
    Ok(examples_dir)
}

fn load_examples() -> Result<HashMap<String, String>> {
    let examples_dir = find_forc_cmd_examples_dir().unwrap();

    let mut command_examples: HashMap<String, String> = HashMap::new();

    for entry in examples_dir
        .read_dir()
        .expect("read dir examples failed")
        .flatten()
    {
        let command_name = get_forc_command_from_file_name(entry.file_name());
        let example_content = fs::read_to_string(entry.path())?;
        command_examples.insert(command_name, example_content);
    }

    Ok(command_examples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_entries_msg() {
        let missing = vec!["forc addr2line".to_string(), "forc build".to_string()];
        let expected_msg = r#"
Some entries were missing from SUMMARY.md:

forc addr2line
forc build


To fix this, add the above entries under the Commands or Plugins chapter in SUMMARY.md, like so:

- [forc addr2line](./forc_addr2line.md)
- [forc build](./forc_build.md)

"#;
        assert_eq!(expected_msg, missing_entries_msg(&missing));
    }

    #[test]
    fn test_dangling_chapters_msg() {
        let commands = vec!["forc addr2line".to_string(), "forc build".to_string()];
        let expected_msg = r#"
Some commands/plugins were removed from the Forc toolchain, but still exist in SUMMARY.md:

forc addr2line
forc build


To fix this, remove the corresponding entries from SUMMARY.md.
"#;
        assert_eq!(expected_msg, dangling_chapters_msg(&commands));
    }
}
