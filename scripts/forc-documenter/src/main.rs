use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io;
use std::io::Write;
use std::process;
use std::str;

#[derive(Parser)]
#[clap(name = "forc-documenter", about = "Forc Documenter")]
struct Cli {
    /// the command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunCommand),
}

#[derive(Debug, Parser)]
struct RunCommand {
    #[clap(short = 'c', long = "command")]
    pub command_name: Option<String>,
}

#[derive(PartialEq)]
pub enum LineKind {
    SubHeader,
    Usage,
    Arg,
    Option,
    Text,
}

pub const SUBHEADERS: &[&str] = &["USAGE:", "ARGS:", "OPTIONS:", "SUBCOMMANDS:"];

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(_command) => {
            if _command.command_name.is_some() {
                generate(&_command.command_name.unwrap());
            } else {
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
                        let (command, description) =
                            line.trim().split_once(" ").unwrap_or(("", ""));
                        possible_commands.push(command.clone());
                    }
                    if line == "SUBCOMMANDS:" {
                        subcommand_is_parsed = true;
                    }
                }

                for command in possible_commands {
                    generate(command);
                }
            }
        }
    }

    Ok(())
}

fn generate(subcommand: &str) -> Result<()>{
    let mut result = String::new();

    let output = process::Command::new("forc")
        .args([subcommand, "--help"])
        .output()
        .expect("forc --help failed to run");

    if output.status.success() == false {
        return Err(anyhow!("Failed to run forc {} --help", subcommand));
    }

    let s = String::from_utf8_lossy(&output.stdout);

    for (index, line) in s.lines().enumerate() {
        let mut formatted_line = String::new();
        let line = line.trim();

        if index == 0 {
            formatted_line.push_str(&format_header_line(line));
        } else if index == 1 {
            formatted_line.push_str(&line);
        } else {
            formatted_line.push_str(&format_line(line))
        }

        result.push_str(&formatted_line);

        if !formatted_line.ends_with("\n") {
            result.push_str("\n");

        }
    }


    let mut file = File::create(subcommand.to_owned() + ".md").expect("Failed to create file");

    file.write_all(&result.as_bytes())
        .expect("Failed to write to file");
    
    Ok(())

}

fn format_line(line: &str) -> String {
    match get_line_kind(line) {
        LineKind::SubHeader => format_subheader_line(line),
        LineKind::Usage => format_usage_line(line),
        LineKind::Option => format_option_line(line),
        LineKind::Arg => format_arg_line(line),
        LineKind::Text => line.to_string(),
    }
}

fn get_line_kind(line: &str) -> LineKind {
    if SUBHEADERS.contains(&line) {
        LineKind::SubHeader
    } else if is_args_line(&line) {
        LineKind::Arg
    } else if is_options_line(&line) {
        LineKind::Option
    } else {
        LineKind::Text
    }
}

fn is_args_line(line: &str) -> bool {
    line.trim().starts_with('<')
}

fn is_options_line(line: &str) -> bool {
    line.trim().starts_with('-') && line.trim().chars().nth(1).unwrap() != ' '
}

fn format_header_line(header_line: &str) -> String {
    "\n# ".to_owned() + header_line + &"\n".to_owned()
}

fn format_subheader_line(subheader_line: &str) -> String {
    "\n## ".to_owned() + subheader_line + &"\n".to_owned()
}

fn format_usage_line(usage_line: &str) -> String {
    usage_line.to_string()
}

fn format_arg_line(arg_line: &str) -> String {
    let mut formatted_arg_line = String::new();

    for c in arg_line.chars() {
        if c == '>' {
            formatted_arg_line.push('_');
            formatted_arg_line.push(c);
        } else if c == '<' {
            formatted_arg_line.push(c);
            formatted_arg_line.push('_');
        } else {
            formatted_arg_line.push(c);
        }
    }
    if !formatted_arg_line.trim().ends_with('>') {
        let last_closing_bracket_idx = formatted_arg_line.rfind('>').unwrap();
        formatted_arg_line.replace_range(
            last_closing_bracket_idx + 1..last_closing_bracket_idx + 2,
            "\n\n",
        );
    }
    formatted_arg_line
}

fn format_option_line(option_line: &str) -> String {
    let mut tokens_iter = option_line.trim().split(" ").into_iter();

    let mut result = String::new();
    let mut rest_of_line = String::new();

    while let Some(token) = tokens_iter.next() {
        if is_option(token) {
            result.push_str(&format_option(token));
        } else if is_arg(token) {
            result.push_str(&format_arg(token));
        } else if token == "" {
            rest_of_line = tokens_iter
                .fold(String::new(), |mut a, b| {
                    a.reserve(b.len() + 1);
                    a.push_str(b);
                    a.push_str(" ");
                    a
                })
                .trim()
                .to_owned();
            break;
        }
    }
    result.push_str("\n\n");
    result.push_str(&rest_of_line);
    result.push_str("\n");

    println!("result: {}", result);
    result
}

fn is_option(token: &str) -> bool {
    token.starts_with("-")
}

fn is_arg(token: &str) -> bool {
    token.starts_with("<")
}

fn format_option(option: &str) -> String {
    match option.ends_with(",") {
        true => {
            let mut s = option.to_string();
            s.pop();
            "`".to_owned() + &s + &"`, ".to_owned()
        }
        false => "`".to_owned() + option + &"` ".to_owned(),
    }
}

fn format_arg(arg: &str) -> String {
    let mut result = String::new();
    let mut inner = arg.to_string();

    inner.pop();
    inner.remove(0);

    result.push('<');
    result.push('_');
    result.push_str(&inner);
    result.push('_');
    result.push('>');

    result
}

#[cfg(test)]
mod tests {
    use crate::is_options_line;

    use super::{format_arg_line, format_header_line, format_option_line, format_subheader_line};

    #[test]
    fn test_format_header_line() {
        let example_header = "forc-fmt";
        let expected_header = "\n# forc-fmt\n";

        assert_eq!(expected_header, format_header_line(example_header));
    }

    #[test]
    fn test_format_subheader_line() {
        let example_subheader = "USAGE:";
        let expected_subheader = "\n## USAGE:\n";

        assert_eq!(expected_subheader, format_subheader_line(example_subheader));
    }

    #[test]
    fn test_format_arg_line() {
        let example_arg_line_1 = "<PROJECT_NAME> Some description";
        let example_arg_line_2 = "<arg1> <arg2> Some description";
        let expected_arg_line_1 = "<_PROJECT_NAME_>\n\nSome description";
        let expected_arg_line_2 = "<_arg1_> <_arg2_>\n\nSome description";

        assert_eq!(expected_arg_line_1, format_arg_line(example_arg_line_1));
        assert_eq!(expected_arg_line_2, format_arg_line(example_arg_line_2));
    }

    #[test]
    fn test_format_option_line() {
        let example_option_line_1 = 
        "-c, --check    Run in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1
        and prints a diff if formatting is required";
        let example_option_line_2 =
            "-o <JSON_OUTFILE> If set, outputs a json file representing the output json abi";
        let expected_option_line_1= "`-c`, `--check`\n\nRun in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff if formatting is required\n";
        let expected_option_line_2 = "`-o` <_JSON_OUTFILE_>\n\n
        If set, outputs a json file representing the output json abi";

        assert_eq!(
            expected_option_line_1,
            format_option_line(example_option_line_1)
        );
        assert_eq!(
            expected_option_line_2,
            format_option_line(example_option_line_2)
        );
    }

    #[test]
    fn test_is_options_line() {
        let example_option_line_1= "    -s, --silent             Silent mode. Don't output any warnings or errors to the command line";
        let example_option_line_2 = "    -o <JSON_OUTFILE>        If set, outputs a json file representing the output json abi";
        let example_option_line_3 = " - counter";

        assert!(is_options_line(example_option_line_1));
        assert!(is_options_line(example_option_line_2));
        assert!(!is_options_line(example_option_line_3));
    }
}
