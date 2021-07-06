use crate::cli::BuildCommand;
use crate::ops::forc_build;
use crate::{
    cli::FormatCommand,
    utils::{
        constants::SWAY_EXTENSION,
        helpers::{find_manifest_dir, print_green, print_red},
    },
};
use formatter::get_formatted_data;
use prettydiff::{basic::DiffOp, diff_lines};
use std::{
    ffi::OsStr,
    fmt, fs, io,
    path::{Path, PathBuf},
};

pub fn format(command: FormatCommand) -> Result<(), FormatError> {
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

fn format_after_build(command: FormatCommand) -> Result<(), FormatError> {
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

                                    println!("{:?}\n", file);

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
                        Err(_) => {
                            // unreachable since we format it only after build is successful
                            unreachable!()
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

fn get_sway_files(path: PathBuf) -> Result<Vec<PathBuf>, FormatError> {
    let mut files = vec![];
    let mut dir_entries = vec![path];

    while let Some(entry) = dir_entries.pop() {
        for inner_entry in fs::read_dir(entry)? {
            if let Ok(entry) = inner_entry {
                let path = entry.path();
                if path.is_dir() {
                    dir_entries.push(path);
                } else {
                    if is_sway_file(&path) {
                        files.push(path)
                    }
                }
            }
        }
    }

    Ok(files)
}

fn format_sway_file(file: &PathBuf, formatted_content: &str) -> Result<(), FormatError> {
    fs::write(file, formatted_content)?;

    Ok(())
}

fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(SWAY_EXTENSION)) == res
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
