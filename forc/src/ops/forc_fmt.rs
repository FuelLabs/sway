use crate::{cli::FormatCommand, utils::constants::SWAY_EXTENSION};
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
            format_sway_files(files)
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
                if entry.path().is_dir() {
                    dir_entries.push(entry.path());
                } else {
                    if is_sway_file(&entry.path()) {
                        files.push(entry.path())
                    }
                }
            }
        }
    }

    Ok(files)
}

fn format_sway_files(files: Vec<PathBuf>) -> Result<(), FormatError> {
    for file in files {
        if let Ok(file_content) = fs::read_to_string(&file) {
            // todo: get tab_size from Manifest file
            let (_, formatted_content) = get_formatted_data(&file_content, 4);
            fs::write(file, formatted_content)?;
        }
    }

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
// Continually go up in the file tree until a manifest (Forc.toml) is found.
fn find_manifest_dir(starter_path: &PathBuf) -> Option<PathBuf> {
    let mut path = fs::canonicalize(starter_path.clone()).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(crate::utils::constants::MANIFEST_FILE_NAME);
        if path.exists() {
            path.pop();
            return Some(path);
        } else {
            path.pop();
            path.pop();
        }
    }
    None
}
