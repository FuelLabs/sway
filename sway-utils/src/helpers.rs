use crate::constants;
use std::ffi::OsStr;

use std::fs;
use std::path::{Path, PathBuf};

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

/// create an iterator over all prefixes in a slice, smallest first
///
/// ```
/// # use sway_utils::iter_prefixes;
/// let val = [1, 2, 3];
/// let mut it = iter_prefixes(&val);
/// assert_eq!(it.next(), Some([1].as_slice()));
/// assert_eq!(it.next(), Some([1, 2].as_slice()));
/// assert_eq!(it.next(), Some([1, 2, 3].as_slice()));
/// assert_eq!(it.next(), None);
///
/// ```
pub fn iter_prefixes<T>(slice: &[T]) -> impl Iterator<Item = &[T]> + DoubleEndedIterator {
    (1..=slice.len()).map(move |len| &slice[..len])
}
