use crate::formatter::format_index_entry;

use commands::{
    get_contents_from_commands, get_forc_command_from_file_name, possible_forc_commands,
};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use plugins::official_plugin_commands;
use std::collections::HashMap;
use std::fs;

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

fn inject_content(chapter: &mut Chapter, content: &str, examples: &HashMap<String, String>) {
    chapter.content = content.to_string();

    if let Some(example_content) = examples.get(&chapter.name) {
        chapter.content += example_content;
    }
}

impl Preprocessor for ForcDocumenter {
    fn name(&self) -> &str {
        "forc-documenter"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let possible_commands: Vec<String> = possible_forc_commands();
        let examples: HashMap<String, String> = load_examples()?;

        let mut command_contents: HashMap<String, String> =
            get_contents_from_commands(&possible_commands);
        let mut plugin_contents: HashMap<String, String> =
            get_contents_from_commands(&official_plugin_commands());
        let mut removed_commands = Vec::new();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if chapter.name == "Plugins" {
                    for sub_item in chapter.sub_items.iter_mut() {
                        if let BookItem::Chapter(ref mut plugin_chapter) = sub_item {
                            if let Some(content) = plugin_contents.remove(&plugin_chapter.name) {
                                inject_content(plugin_chapter, &content, &examples);
                            } else {
                                removed_commands.push(plugin_chapter.name.clone());
                            };
                        }
                    }
                }
                if chapter.name == "Commands" {
                    let mut command_index_content = String::new();

                    for sub_item in chapter.sub_items.iter_mut() {
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

        if !command_contents.is_empty() {
            let missing_entries: String = command_contents
                .keys()
                .map(|c| format_index_entry(c))
                .collect();

            let missing_summary_entries = format!("\nSome forc commands were missing from SUMMARY.md:\n\n{}\n\nTo fix this, add the above command(s) in SUMMARY.md, like so:\n\n{}\n",
                command_contents.into_keys().map(|s| s + "\n").collect::<String>(), missing_entries);
            error_message.push_str(&missing_summary_entries);
        };

        if !plugin_contents.is_empty() {
            let missing_entries: String = plugin_contents
                .keys()
                .map(|c| format_index_entry(c))
                .collect();

            let missing_summary_entries = format!("\nSome forc plugins were missing from SUMMARY.md:\n\n{}\nTo fix this, add the above command(s) in SUMMARY.md, like so:\n\n{}\n",
                plugin_contents.into_keys().map(|s| s + "\n").collect::<String>(), missing_entries);
            error_message.push_str(&missing_summary_entries);
        }

        if !removed_commands.is_empty() {
            let removed_commands_text = format!("\nSome commands were removed from the Forc toolchain, but still exist in SUMMARY.md:\n\n{}\n\nTo fix this, remove the above command(s) from SUMMARY.md.\n", 
            removed_commands
                .iter()
                .map(String::as_str)
                .collect::<String>());
            error_message.push_str(&removed_commands_text);
        };

        if !error_message.is_empty() {
            Err(Error::msg(error_message))
        } else {
            Ok(book)
        }
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn load_examples() -> Result<HashMap<String, String>> {
    let curr_path = std::env::current_dir()
        .unwrap()
        .join("scripts/mdbook-forc-documenter/examples");

    let mut command_examples: HashMap<String, String> = HashMap::new();

    for entry in curr_path
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
