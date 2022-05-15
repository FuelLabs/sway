use crate::formatter::{format_header_line, format_index_entry, format_line};

use anyhow::anyhow;
use mdbook::book::{Book, BookItem};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::process;

mod formatter;

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

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let output = process::Command::new("forc")
            .arg("--help")
            .output()
            .expect("Failed running forc --version");

        let s = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);
        let lines = s.lines();

        let mut subcommand_is_parsed = false;
        let mut possible_commands = vec![];

        for line in lines {
            if subcommand_is_parsed {
                let (command, _) = line.trim().split_once(' ').unwrap_or(("", ""));
                possible_commands.push(command);
            }
            if line == "SUBCOMMANDS:" {
                subcommand_is_parsed = true;
            }
        }

        let command_examples: HashMap<String, String> = load_examples()?;
        let mut command_contents: HashMap<String, String> = HashMap::new();
        let mut removed_commands = Vec::new();

        for forc_command in possible_commands.iter() {
            let mut result = match generate_doc_output(forc_command) {
                Ok(output) => output,
                Err(_) => continue,
            };
            result = result.trim().to_string();
            command_contents.insert("forc ".to_owned() + forc_command, result);
        }

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if chapter.name == "Commands" {
                    let mut command_index_content = String::new();

                    for sub_item in chapter.sub_items.iter_mut() {
                        if let BookItem::Chapter(ref mut command_chapter) = sub_item {
                            if let Some(content) = command_contents.remove(&command_chapter.name) {
                                command_index_content
                                    .push_str(&format_index_entry(&command_chapter.name));
                                command_chapter.content = content.to_string();

                                if let Some(example_content) =
                                    command_examples.get(&command_chapter.name)
                                {
                                    command_chapter.content += example_content;
                                }
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
            let missing_entries_text: String = command_contents
                .keys()
                .map(|c| format_index_entry(&c))
                .collect();

            let missing_summary_entries_text = format!("\nSome commands were missing from SUMMARY.md:\n\n{}\n\nTo fix this, add the above command(s) in SUMMARY.md, like so:\n\n{}\n",
                command_contents.into_keys().collect::<String>(), missing_entries_text);
            error_message.push_str(&missing_summary_entries_text);
        };

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

fn get_forc_command_from_file_name(file_name: OsString) -> String {
    file_name
        .into_string()
        .unwrap()
        .split('.')
        .next()
        .unwrap()
        .to_string()
        .replace('_', " ")
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

fn generate_doc_output(subcommand: &str) -> Result<String> {
    let mut result = String::new();

    let output = process::Command::new("forc")
        .args([subcommand, "--help"])
        .output()
        .expect("forc --help failed to run");

    if !output.status.success() {
        return Err(anyhow!("Failed to run forc {} --help", subcommand));
    }

    let s = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);

    for (index, line) in s.lines().enumerate() {
        let mut formatted_line = String::new();
        let line = line.trim();

        if index == 0 {
            formatted_line.push_str(&format_header_line(line));
        } else if index == 1 {
            formatted_line.push_str(line);
        } else {
            formatted_line.push_str(&format_line(line))
        }

        result.push_str(&formatted_line);

        if !formatted_line.ends_with('\n') {
            result.push('\n');
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_forc_command_from_file_name() {
        assert_eq!(
            "forc gm",
            get_forc_command_from_file_name(OsString::from("forc_gm.md")),
        );
    }
}
