use anyhow::{bail, Result};
use std::fs::File;
use std::io::Read;
use tracing::{info, error};

pub fn is_option(token: &str) -> bool {
    token.starts_with('-')
}

pub fn is_arg(token: &str) -> bool {
    token.starts_with('<')
}

pub fn is_args_line(line: &str) -> bool {
    line.trim().starts_with('<')
}

pub fn is_options_line(line: &str) -> bool {
    line.trim().starts_with('-') && line.trim().chars().nth(1).unwrap() != ' '
}

pub fn check_summary_diffs(
    existing_summary_contents: &str,
    new_summary_contents: &str,
) -> Result<()> {
    if existing_summary_contents == new_summary_contents {
        info!("[✓] SUMMARY.md ok.");
    } else {
        error!("[x] SUMMARY.md inconsistent!");
        bail!("SUMMARY.md is inconsistent.");
    }

    Ok(())
}

pub fn check_index_diffs(mut index_file: File, new_index_contents: &str) -> Result<()> {
    let mut existing_index_contents = String::new();
    index_file.read_to_string(&mut existing_index_contents)?;
    if existing_index_contents == new_index_contents {
        info!("[✓] index.md ok.");
    } else {
        error!("[x] index.md inconsistent!");
        bail!("index.md is inconsistent.");
    }

    Ok(())
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
