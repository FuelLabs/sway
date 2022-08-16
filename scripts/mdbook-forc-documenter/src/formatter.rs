#[derive(PartialEq, Eq)]
pub enum LineKind {
    SubHeader,
    Arg,
    Option,
    Subcommand,
    Text,
}

fn get_line_kind(line: &str, has_parsed_subcommand_header: bool) -> LineKind {
    if SUBHEADERS.contains(&line) {
        LineKind::SubHeader
    } else if is_args_line(line) {
        LineKind::Arg
    } else if is_options_line(line) {
        LineKind::Option
    } else if has_parsed_subcommand_header {
        LineKind::Subcommand
    } else {
        LineKind::Text
    }
}

pub const SUBHEADERS: &[&str] = &["USAGE:", "ARGS:", "OPTIONS:", "SUBCOMMANDS:"];

pub fn is_args_line(line: &str) -> bool {
    line.trim().starts_with('<')
}

pub fn is_options_line(line: &str) -> bool {
    line.trim().starts_with('-') && line.trim().chars().nth(1).unwrap() != ' '
}

pub fn is_option(token: &str) -> bool {
    token.starts_with('-')
}

pub fn is_arg(token: &str) -> bool {
    token.starts_with('<')
}

pub fn format_header_line(header_line: &str) -> String {
    "\n# ".to_owned() + header_line.split_whitespace().next().unwrap() + "\n"
}

pub fn format_line(line: &str, has_parsed_subcommand_header: bool) -> String {
    match get_line_kind(line, has_parsed_subcommand_header) {
        LineKind::SubHeader => format_subheader_line(line),
        LineKind::Option => format_option_line(line),
        LineKind::Arg => format_arg_line(line),
        LineKind::Subcommand => format_subcommand_line(line),
        LineKind::Text => line.to_string(),
    }
}

fn format_subheader_line(subheader_line: &str) -> String {
    "\n## ".to_owned() + subheader_line + "\n"
}

fn format_subcommand_line(line: &str) -> String {
    let mut line_iter = line.trim().splitn(2, ' ');
    let name = "`".to_owned() + line_iter.next().unwrap() + "`\n\n";
    let text = line_iter.collect::<String>().trim_start().to_owned() + "\n\n";
    name + &text
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
    "\n".to_owned() + &formatted_arg_line
}

fn format_option_line(option_line: &str) -> String {
    let mut tokens_iter = option_line.trim().split(' ');

    let mut result = String::new();
    let mut rest_of_line = String::new();

    while let Some(token) = tokens_iter.next() {
        if is_option(token) {
            result.push_str(&format_option(token));
        } else if is_arg(token) {
            result.push_str(&format_arg(token));
        } else {
            rest_of_line.push_str(token);
            rest_of_line.push(' ');
            rest_of_line = tokens_iter
                .fold(rest_of_line, |mut a, b| {
                    a.reserve(b.len() + 1);
                    a.push_str(b);
                    a.push(' ');
                    a
                })
                .trim()
                .to_owned();
            break;
        }
    }
    result.push_str("\n\n");
    result.push_str(&rest_of_line);
    result.push('\n');

    "\n".to_owned() + &result
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

fn format_option(option: &str) -> String {
    match option.ends_with(',') {
        true => {
            let mut s = option.to_string();
            s.pop();
            "`".to_owned() + &s + "`, "
        }
        false => "`".to_owned() + option + "` ",
    }
}

/// Index entries should be in the form of:
/// - [forc addr2line](./forc_addr2line.md)\n
pub fn format_index_entry(forc_command_str: &str) -> String {
    let command_name = forc_command_str;
    let command_link = forc_command_str.replace(' ', "_");
    format!("- [{}](./{}.md)\n", command_name, command_link)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_options_line() {
        let example_option_line_1= "    -s, --silent             Silent mode. Don't output any warnings or errors to the command line";
        let example_option_line_2 = "    -o <JSON_OUTFILE>        If set, outputs a json file representing the output json abi";
        let example_option_line_3 = " - counter";

        assert!(is_options_line(example_option_line_1));
        assert!(is_options_line(example_option_line_2));
        assert!(!is_options_line(example_option_line_3));
    }

    #[test]
    fn test_format_index_entry() {
        let forc_command = "forc build";

        assert_eq!(
            format_index_entry(forc_command),
            "- [forc build](./forc_build.md)\n"
        );
    }

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
        let expected_arg_line_1 = "\n<_PROJECT_NAME_>\n\nSome description";
        let expected_arg_line_2 = "\n<_arg1_> <_arg2_>\n\nSome description";

        assert_eq!(expected_arg_line_1, format_arg_line(example_arg_line_1));
        assert_eq!(expected_arg_line_2, format_arg_line(example_arg_line_2));
    }

    #[test]
    fn test_format_option_line() {
        let example_option_line_1 = "-c, --check    Run in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff if formatting is required";
        let example_option_line_2 =
            "-o <JSON_OUTFILE> If set, outputs a json file representing the output json abi";
        let expected_option_line_1= "\n`-c`, `--check` \n\nRun in 'check' mode. Exits with 0 if input is formatted correctly. Exits with 1 and prints a diff if formatting is required\n";
        let expected_option_line_2 = "\n`-o` <_JSON_OUTFILE_>\n\nIf set, outputs a json file representing the output json abi\n";

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
    fn test_format_subcommand_line() {
        let example_subcommand =
            "   clean     Cleans up any existing state associated with the fuel block explorer";
        let expected_subcommand =
            "`clean`\n\nCleans up any existing state associated with the fuel block explorer\n\n";

        assert_eq!(
            expected_subcommand,
            format_subcommand_line(example_subcommand)
        );
    }
}
