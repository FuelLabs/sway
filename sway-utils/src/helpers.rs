use crate::constants;
use std::ffi::OsStr;

use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn get_sway_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut dir_entries = vec![path];

    while let Some(next_dir) = dir_entries.pop() {
        if let Ok(read_dir) = fs::read_dir(next_dir) {
            for entry in read_dir.filter_map(|res| res.ok()) {
                let path = entry.path();

                if path.is_dir() {
                    dir_entries.push(path);
                } else if is_sway_file(&path) {
                    files.push(path)
                }
            }
        }
    }

    files
}
pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(constants::SWAY_EXTENSION)) == res
}
