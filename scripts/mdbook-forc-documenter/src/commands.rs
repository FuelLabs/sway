use crate::formatter::{format_header_line, format_line};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::ffi::OsString;
use std::process;

pub fn possible_forc_commands() -> Vec<String> {
    let mut possible_commands = Vec::new();
    let output = process::Command::new("forc")
        .arg("--help")
        .output()
        .expect("Failed running forc --help");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines = output_str.lines();

    let mut has_parsed_subcommand_header = false;

    for line in lines {
        if has_parsed_subcommand_header {
            let (command, _) = line.trim().split_once(' ').unwrap_or(("", ""));
            possible_commands.push(command.to_string());
        }
        if line == "SUBCOMMANDS:" {
            has_parsed_subcommand_header = true;
        }
    }

    possible_commands
}

pub fn get_contents_from_commands(commands: &[String]) -> HashMap<String, String> {
    let mut contents: HashMap<String, String> = HashMap::new();

    for command in commands {
        let result = match generate_documentation(command) {
            Ok(output) => output,
            Err(_) => continue,
        };
        contents.insert("forc ".to_owned() + command, result);
    }

    contents
}

fn generate_documentation(subcommand: &str) -> Result<String> {
    let mut result = String::new();
    let mut has_parsed_subcommand_header = false;

    let output = process::Command::new("forc")
        .args([subcommand, "--help"])
        .output()
        .expect("Failed running forc --help");

    if !output.status.success() {
        return Err(anyhow!("Failed to run forc {} --help", subcommand));
    }

    let s = String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr);

    for (index, line) in s.lines().enumerate() {
        let mut formatted_line = String::new();
        let line = line.trim();

        if line == "SUBCOMMANDS:" {
            has_parsed_subcommand_header = true;
        }

        if index == 0 {
            formatted_line.push_str(&format_header_line(line));
        } else if index == 1 {
            formatted_line.push_str(line);
        } else {
            formatted_line.push_str(&format_line(line, has_parsed_subcommand_header))
        }

        result.push_str(&formatted_line);

        if !formatted_line.ends_with('\n') {
            result.push('\n');
        }
    }
    result = result.trim().to_string();
    Ok(result)
}

pub fn get_forc_command_from_file_name(file_name: OsString) -> String {
    file_name
        .into_string()
        .unwrap()
        .split('.')
        .next()
        .unwrap()
        .to_string()
        .replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_forc_command_from_file_name() {
        assert_eq!(
            "forc explore",
            get_forc_command_from_file_name(OsString::from("forc_explore.md")),
        );
    }
}
