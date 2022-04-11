fn is_option(token: &str) -> bool {
    token.starts_with('-')
}

fn is_arg(token: &str) -> bool {
    token.starts_with('<')
}

pub fn is_args_line(line: &str) -> bool {
    line.trim().starts_with('<')
}

pub fn is_options_line(line: &str) -> bool {
    line.trim().starts_with('-') && line.trim().chars().nth(1).unwrap() != ' '
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
}
