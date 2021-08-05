use crate::cli::BuildCommand;
use crate::ops::forc_build;
use crate::utils::cli_error::CliError;
use crate::utils::helpers::get_sway_files;
use crate::{
    cli::FormatCommand,
    utils::helpers::{find_manifest_dir, print_green, print_red},
};
use formatter::get_formatted_data;
use prettydiff::{basic::DiffOp, diff_lines};
use std::{fs, path::PathBuf};

pub fn format(command: FormatCommand) -> Result<(), CliError> {
    let build_command = BuildCommand {
        path: None,
        print_asm: false,
        binary_outfile: None,
        offline_mode: false,
    };

    match forc_build::build(build_command) {
        // build is successful, continue to formatting
        Ok(_) => format_after_build(command),

        // forc_build will print all the errors/warnings
        Err(err) => Err(err.into()),
    }
}

fn format_after_build(command: FormatCommand) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir()?;

    match find_manifest_dir(&curr_dir) {
        Some(path) => {
            let files = get_sway_files(path)?;
            let mut contains_edits = false;

            for file in files {
                if let Ok(file_content) = fs::read_to_string(&file) {
                    // todo: get tab_size from Manifest file
                    match get_formatted_data(&file_content, 4) {
                        Ok((_, formatted_content)) => {
                            if command.check {
                                if file_content != formatted_content {
                                    let changeset = diff_lines(&file_content, &formatted_content);

                                    println!("\n{:?}\n", file);

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
                                                    print_green(&format!("+{}", n))?;
                                                }
                                            }
                                            DiffOp::Remove(old) => {
                                                count_of_updates += 1;
                                                for o in old {
                                                    print_red(&format!("-{}", o))?;
                                                }
                                            }
                                            DiffOp::Replace(old, new) => {
                                                count_of_updates += 1;
                                                for o in old {
                                                    print_red(&format!("-{}", o))?;
                                                }
                                                for n in new {
                                                    print_green(&format!("+{}", n))?;
                                                }
                                            }
                                        }
                                    }

                                    if !contains_edits {
                                        contains_edits = true;
                                    }
                                }
                            } else {
                                format_sway_file(&file, &formatted_content)?;
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

            if command.check {
                if contains_edits {
                    // One or more files are not formatted, exit with error
                    std::process::exit(1);
                } else {
                    // All files are formatted, exit cleanly
                    std::process::exit(0);
                }
            }

            Ok(())
        }
        _ => Err("Manifest file does not exist".into()),
    }
}

fn format_sway_file(file: &PathBuf, formatted_content: &str) -> Result<(), CliError> {
    fs::write(file, formatted_content)?;

    Ok(())
}
