use crate::cli::{BuildCommand, FormatCommand};
use crate::ops::forc_build;
use crate::utils::helpers::{println_green, println_red};
use prettydiff::{basic::DiffOp, diff_lines};
use std::{fmt, fs, io, path::Path, sync::Arc};
use sway_fmt::get_formatted_data;
use sway_utils::{constants, find_manifest_dir, get_sway_files};
use taplo::formatter as taplo_fmt;

pub fn format(command: FormatCommand) -> Result<(), FormatError> {
    let build_command = BuildCommand {
        path: None,
        print_finalized_asm: false,
        print_intermediate_asm: false,
        binary_outfile: None,
        offline_mode: false,
        silent_mode: false,
    };

    match forc_build::build(build_command) {
        // build is successful, continue to formatting
        Ok(_) => format_after_build(command),

        // forc_build will print all the errors/warnings
        Err(err) => Err(err.into()),
    }
}

fn format_after_build(command: FormatCommand) -> Result<(), FormatError> {
    let curr_dir = std::env::current_dir()?;

    match find_manifest_dir(&curr_dir) {
        Some(path) => {
            let mut manifest_file = path.clone();
            manifest_file.push(constants::MANIFEST_FILE_NAME);
            let files = get_sway_files(path);
            let mut contains_edits = false;

            for file in files {
                if let Ok(file_content) = fs::read_to_string(&file) {
                    // todo: get tab_size from Manifest file
                    let file_content: Arc<str> = Arc::from(file_content);
                    match get_formatted_data(file_content.clone(), 4) {
                        Ok((_, formatted_content)) => {
                            if command.check {
                                if *file_content != *formatted_content {
                                    if !contains_edits {
                                        contains_edits = true;
                                    }
                                    println!("\n{:?}\n", file);
                                    display_file_diff(&file_content, &formatted_content)?;
                                }
                            } else {
                                format_file(&file, &formatted_content)?;
                            }
                        }
                        Err(err) => {
                            // there could still be Sway files that are not part of the build
                            eprintln!("\nThis file: {:?} is not part of the build", file);
                            eprintln!("{}", err.join("\n"));
                        }
                    }
                }
            }

            // format manifest using taplo formatter
            if let Ok(file_content) = fs::read_to_string(&manifest_file) {
                let formatted_content = taplo_fmt::format(
                    &file_content,
                    taplo_fmt::Options {
                        ..Default::default()
                    },
                );
                if command.check {
                    if formatted_content != file_content {
                        if !contains_edits {
                            contains_edits = true;
                        }
                        eprintln!("\nManifest Forc.toml improperly formatted");
                        display_file_diff(&file_content, &formatted_content)?;
                    } else {
                        format_file(&manifest_file, &formatted_content)?;
                    }
                }
            }

            if command.check {
                if contains_edits {
                    // One or more files are not formatted, exit with error
                    Err("Files contain formatting violations.".into())
                } else {
                    // All files are formatted, exit cleanly
                    Ok(())
                }
            } else {
                Ok(())
            }
        }
        _ => Err("Manifest file does not exist".into()),
    }
}

fn display_file_diff(file_content: &str, formatted_content: &str) -> Result<(), FormatError> {
    let changeset = diff_lines(file_content, formatted_content);
    let mut count_of_updates = 0;
    for diff in changeset.diff() {
        // max 100 updates
        if count_of_updates >= 100 {
            break;
        }
        match diff {
            DiffOp::Equal(old) => {
                for o in old {
                    println!("{}", o)
                }
            }
            DiffOp::Insert(new) => {
                count_of_updates += 1;
                for n in new {
                    println_green(&format!("+{}", n))?;
                }
            }
            DiffOp::Remove(old) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{}", o))?;
                }
            }
            DiffOp::Replace(old, new) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{}", o))?;
                }
                for n in new {
                    println_green(&format!("+{}", n))?;
                }
            }
        }
    }
    Result::Ok(())
}

fn format_file(file: &Path, formatted_content: &str) -> Result<(), FormatError> {
    fs::write(file, formatted_content)?;

    Ok(())
}

pub struct FormatError {
    pub message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<&str> for FormatError {
    fn from(s: &str) -> Self {
        FormatError {
            message: s.to_string(),
        }
    }
}

impl From<String> for FormatError {
    fn from(s: String) -> Self {
        FormatError { message: s }
    }
}

impl From<io::Error> for FormatError {
    fn from(e: io::Error) -> Self {
        FormatError {
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::taplo_fmt;
    use std::default::Default;

    #[test]
    fn test_forc_indentation() {
        let correct_forc_manifest = r#"
[project]
author = "Fuel Labs <contact@fuel.sh>"
license = "Apache-2.0"
name = "Fuel example project"


[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core" }
std = { git = "http://github.com/FuelLabs/sway-lib-std" }
"#;
        let formatted_content = taplo_fmt::format(
            correct_forc_manifest,
            taplo_fmt::Options {
                ..Default::default()
            },
        );
        assert_eq!(formatted_content, correct_forc_manifest);
        let indented_forc_manifest = r#"
        [project]
    author = "Fuel Labs <contact@fuel.sh>"
                    license = "Apache-2.0"
    name = "Fuel example project"


    [dependencies]
        core = { git = "http://github.com/FuelLabs/sway-lib-core" }
                    std = { git = "http://github.com/FuelLabs/sway-lib-std" }
"#;
        let formatted_content = taplo_fmt::format(
            indented_forc_manifest,
            taplo_fmt::Options {
                ..Default::default()
            },
        );
        assert_eq!(formatted_content, correct_forc_manifest);
        let whitespace_forc_manifest = r#"
[project]
 author="Fuel Labs <contact@fuel.sh>"
license   =                                   "Apache-2.0"
name = "Fuel example project"


[dependencies]
core = {git="http://github.com/FuelLabs/sway-lib-core"}
std         =     {   git     =  "http://github.com/FuelLabs/sway-lib-std"             }
"#;
        let formatted_content = taplo_fmt::format(
            whitespace_forc_manifest,
            taplo_fmt::Options {
                ..Default::default()
            },
        );
        assert_eq!(formatted_content, correct_forc_manifest);
    }
}
