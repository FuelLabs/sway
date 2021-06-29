use crate::{
    cli::FormatCommand,
    utils::{constants::SWAY_EXTENSION, helpers::find_manifest_dir},
};
use formatter::get_formatted_data;
use prettydiff::diff_lines;
use std::{
    ffi::OsStr,
    fmt, fs, io,
    path::{Path, PathBuf},
};

pub fn format(command: FormatCommand) -> Result<(), FormatError> {
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
                                    println!("{}", changeset);

                                    if !contains_edits {
                                        contains_edits = true;
                                    }
                                }
                            } else {
                                format_sway_file(&file, &formatted_content)?;
                            }
                        }
                        Err(errors) => {
                            eprintln!("{}", errors.join("\n"));
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

impl From<io::Error> for FormatError {
    fn from(e: io::Error) -> Self {
        FormatError {
            message: e.to_string(),
        }
    }
}
