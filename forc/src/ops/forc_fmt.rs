use crate::{
    cli::FormatCommand,
    utils::{constants::SWAY_EXTENSION, helpers::find_manifest_dir},
};
use formatter::get_formatted_data;
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
            let mut files_to_be_formatted = vec![];

            for file in files {
                if let Ok(file_content) = fs::read_to_string(&file) {
                    match core_lang::parse(&file_content) {
                        core_lang::CompileResult::Ok {
                            value: _,
                            warnings: _,
                            errors: _,
                        } => {
                            if command.check {
                                if file_should_change(&file_content) {
                                    files_to_be_formatted.push(format!("{:?}", file))
                                }
                            } else {
                                format_sway_file(&file_content, &file)?;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if command.check {
                if files_to_be_formatted.is_empty() {
                    // All files are formatted, exit cleanly
                    std::process::exit(0);
                } else {
                    for file in files_to_be_formatted {
                        eprintln!("{}", file);
                    }
                    // One or more files are not formatted, exit with error
                    std::process::exit(1);
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

fn file_should_change(file_content: &str) -> bool {
    // todo: get tab_size from Manifest file
    let (_, formatted_content) = get_formatted_data(file_content, 4);
    let should_change = file_content != &formatted_content;
    should_change
}

fn format_sway_file(file_content: &str, file: &PathBuf) -> Result<(), FormatError> {
    // todo: get tab_size from Manifest file
    let (_, formatted_content) = get_formatted_data(file_content, 4);
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
