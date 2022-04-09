use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, read_to_string, remove_dir_all, File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::str;

mod constants;
mod helpers;

use crate::helpers::{
    format_command_doc_name, format_header_line, format_index_entry_name,
    format_index_entry_string, format_line,
};

#[derive(Parser)]
#[clap(name = "forc-documenter", about = "Forc Documenter")]
struct Cli {
    /// the command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    WriteDocs(WriteDocsCommand),
}

#[derive(Debug, Parser)]
struct WriteDocsCommand {
    #[clap(long)]
    pub dry_run: bool,
}

fn get_sway_path() -> PathBuf {
    let curr_dir = std::env::current_dir().unwrap();

    if curr_dir.ends_with("sway") {
        return curr_dir;
    }

    let sway_dir = curr_dir
        .parent()
        .unwrap()
        .parent()
        .expect("Unable to navigate to project root");
    sway_dir.to_path_buf()
}

fn create_forc_commands_docs_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        create_dir_all(&path)?;
    }

    Ok(())
}

fn get_example_for_command(command: &str) -> &str {
    match command {
        "init" => constants::FORC_INIT_EXAMPLE,
        "build" => constants::FORC_BUILD_EXAMPLE,
        "test" => constants::FORC_TEST_EXAMPLE,
        "deploy" => constants::FORC_DEPLOY_EXAMPLE,
        "parse-bytecode" => constants::FORC_PARSE_BYTECODE_EXAMPLE,
        _ => "",
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::WriteDocs(_command) => {
            let WriteDocsCommand { dry_run } = _command;

            let forc_commands_docs_path = get_sway_path().join("docs/src/forc/commands");
            let index_file_path = forc_commands_docs_path.join("index.md");

            if !dry_run {
                remove_dir_all(&forc_commands_docs_path)
                    .expect("Failed to clean commands directory");
                create_forc_commands_docs_dir(&forc_commands_docs_path)
                    .expect("Failed to prepare forc commands docs directory");
            }
            let mut index_file = OpenOptions::new()
                .create(!dry_run)
                .read(true)
                .write(!dry_run)
                .open(index_file_path)
                .expect("Problem reading, opening or creating forc/commands/index.md");

            let output = process::Command::new("forc")
                .arg("--help")
                .output()
                .expect("Failed to run help command");

            let s = String::from_utf8_lossy(&output.stdout);
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

            let mut index_contents = String::new();

            index_contents.push_str(constants::INDEX_HEADER);

            for command in possible_commands.iter() {
                let mut result = match generate_doc_output(command) {
                    Ok(output) => output,
                    Err(_) => continue,
                };

                let example = get_example_for_command(command);
                if !example.is_empty() {
                    result.push_str(constants::EXAMPLES_HEADER);
                    result.push_str(example);
                }
                result = result.trim().to_string();

                let document_name = format_command_doc_name(command);
                let index_entry_name = format_index_entry_name(command);
                let index_entry_string =
                    format_index_entry_string(&document_name, &index_entry_name);

                let forc_command_file_path = forc_commands_docs_path.join(document_name);
                index_contents.push_str(&index_entry_string);

                if dry_run {
                    let existing_contents = read_to_string(&forc_command_file_path);
                    match existing_contents {
                        Ok(existing_contents) => {
                            if existing_contents == result {
                                println!("forc {}: documentation ok.", &command);
                            } else {
                                return Err(anyhow!(
                                    "Documentation inconsistent for forc {} - {}",
                                    &command,
                                    constants::RUN_WRITE_DOCS_MESSAGE
                                ));
                            }
                        }
                        Err(_) => {
                            return Err(anyhow!(
                                "Documentation does not exist for forc {} - {}",
                                &command,
                                constants::RUN_WRITE_DOCS_MESSAGE
                            ));
                        }
                    }
                } else {
                    println!("Generating docs for command: forc {}...", &command);
                    let mut command_file = File::create(&forc_command_file_path)
                        .expect("Failed to create documentation");
                    command_file
                        .write_all(result.as_bytes())
                        .expect("Failed to write to file");
                }
            }

            if dry_run {
                let mut existing_index_contents = String::new();
                index_file.read_to_string(&mut existing_index_contents)?;

                if index_contents == existing_index_contents {
                    println!("index.md ok.");
                } else {
                    return Err(anyhow!(
                        "index.md inconsistent - {}",
                        constants::RUN_WRITE_DOCS_MESSAGE
                    ));
                }
            } else {
                index_file
                    .write_all(index_contents.as_bytes())
                    .expect("Failed to write to forc/commands/index.md");
            }

            println!("Done.");
        }
    }
    Ok(())
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

    let s = String::from_utf8_lossy(&output.stdout);

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
