use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, read_to_string, remove_dir_all, File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::str;

mod checkers;
mod constants;
mod helpers;

use crate::checkers::{check_index_diffs, check_summary_diffs};
use crate::helpers::{
    format_command_doc_name, format_header_line, format_index_entry_name,
    format_index_entry_string, format_index_line_for_summary, format_line,
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
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.ends_with("sway") {
            return curr_path;
        }
        curr_path = curr_path.parent().unwrap().to_path_buf()
    }
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

fn write_new_summary_contents(
    existing_summary_contents: String,
    new_index_contents: String,
) -> String {
    let mut new_summary_contents = String::new();
    for line in existing_summary_contents.lines() {
        if line.contains("[Commands](./forc/commands/index.md)") {
            new_summary_contents.push_str(line);
            new_summary_contents.push('\n');
            for index_line in new_index_contents.lines().skip(2) {
                let summary_index_line = format_index_line_for_summary(index_line);
                new_summary_contents.push_str(&("    ".to_owned() + &summary_index_line));
                new_summary_contents.push('\n');
            }
        } else if line.contains("/forc/commands/") {
            continue;
        } else {
            new_summary_contents.push_str(line);
            new_summary_contents.push('\n');
        }
    }
    new_summary_contents
}

fn write_docs(command: WriteDocsCommand) -> Result<()> {
    let WriteDocsCommand { dry_run } = command;

    let forc_commands_docs_path = get_sway_path().join("docs/src/forc/commands");
    let summary_file_path = get_sway_path().join("docs/src/SUMMARY.md");
    let index_file_path = forc_commands_docs_path.join("index.md");

    if !dry_run {
        remove_dir_all(&forc_commands_docs_path).expect("Failed to clean commands directory");
        create_forc_commands_docs_dir(&forc_commands_docs_path)
            .expect("Failed to prepare forc commands docs directory");
    }
    let mut index_file = OpenOptions::new()
        .create(!dry_run)
        .read(true)
        .write(!dry_run)
        .open(index_file_path)
        .expect("Problem reading, opening or creating forc/commands/index.md");

    let mut summary_file =
        File::open(&summary_file_path).expect("Problem reading, opening or creating SUMMARY.md");

    let mut existing_summary_contents = String::new();
    summary_file.read_to_string(&mut existing_summary_contents)?;

    let output = process::Command::new("forc")
        .arg("--version")
        .output()
        .expect("Failed running forc --version");
    let version = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);
    let version_message = "Running forc --help using ".to_owned() + &version;
    println!("{}", version_message);

    let output = process::Command::new("forc")
        .arg("--help")
        .output()
        .expect("Failed to run help command");

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

    let mut new_index_contents = String::new();
    new_index_contents.push_str(constants::INDEX_HEADER);

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
        let index_entry_string = format_index_entry_string(&document_name, &index_entry_name);

        let forc_command_file_path = forc_commands_docs_path.join(document_name);
        new_index_contents.push_str(&index_entry_string);

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
            let mut command_file =
                File::create(&forc_command_file_path).expect("Failed to create documentation");
            command_file
                .write_all(result.as_bytes())
                .expect("Failed to write to file");
        }
    }

    let new_summary_contents = write_new_summary_contents(
        existing_summary_contents.clone(),
        new_index_contents.clone(),
    );
    if dry_run {
        check_index_diffs(index_file, new_index_contents)?;
        check_summary_diffs(existing_summary_contents, new_summary_contents)?;
    } else {
        println!("Updating forc commands in forc/commands/index.md...");
        index_file
            .write_all(new_index_contents.as_bytes())
            .expect("Failed to write to forc/commands/index.md");

        let mut new_summary_file = File::create(&summary_file_path)?;
        println!("Updating forc commands in SUMMARY.md...");
        new_summary_file
            .write_all(new_summary_contents.as_bytes())
            .expect("Failed to write to SUMMARY.md");
    }

    println!("Done.");
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
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::WriteDocs(command) => write_docs(command)?,
    }
    Ok(())
}
