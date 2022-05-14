use crate::formatter::{format_header_line, format_index_entry, format_line};
use anyhow::{anyhow, Result};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
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

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if chapter.name == "Commands" {
                    // add commands stuff here
                    let mut command_index_content = String::new();

                    for sub_item in chapter.sub_items.iter_mut() {
                        if let BookItem::Chapter(ref mut command_chapter) = sub_item {
                            let forc_subcommand = command_chapter.name.split(' ').nth(1).unwrap();
                            if possible_commands.iter().any(|&i| i == forc_subcommand) {
                                let mut result = match generate_doc_output(forc_subcommand) {
                                    Ok(output) => output,
                                    Err(_) => continue,
                                };

                                result = result.trim().to_string();
                                command_index_content
                                    .push_str(&format_index_entry(&command_chapter.name));
                                command_chapter.content = result;
                            }
                        }
                    }

                    chapter.content.push_str(&command_index_content);
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
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
