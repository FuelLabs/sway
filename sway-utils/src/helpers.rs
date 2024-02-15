use crate::constants;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

/// Returns a vector of paths to Sway files found within the given directory and its subdirectories.
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

/// Checks if the given file path points to a Sway file.
pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    file.is_file() && Some(OsStr::new(constants::SWAY_EXTENSION)) == res
}

/// Create an iterator over all prefixes in a slice, smallest first.
pub fn iter_prefixes<T>(slice: &[T]) -> impl DoubleEndedIterator<Item = &[T]> {
    (1..=slice.len()).map(move |len| &slice[..len])
}

/// Continually goes down in the file tree until a Forc manifest file is found.
pub fn find_nested_manifest_dir(starter_path: &Path) -> Option<PathBuf> {
    find_nested_dir_with_file(starter_path, constants::MANIFEST_FILE_NAME)
}

/// Continually goes down in the file tree until a specified file is found.
/// Starts the search from child directories of `starter_path`.
pub fn find_nested_dir_with_file(starter_path: &Path, file_name: &str) -> Option<PathBuf> {
    use walkdir::WalkDir;
    let starter_dir = if starter_path.is_dir() {
        starter_path
    } else {
        starter_path.parent()?
    };
    WalkDir::new(starter_path).into_iter().find_map(|e| {
        let entry = e.ok()?;
        if entry.path() != starter_dir.join(file_name)
            && entry.file_name().to_string_lossy() == file_name
        {
            let mut entry = entry.path().to_path_buf();
            entry.pop();
            Some(entry)
        } else {
            None
        }
    })
}

/// Continually goes up in the file tree until a specified file is found.
/// Starts the search from `starter_path`.
pub fn find_parent_dir_with_file<P: AsRef<Path>>(
    starter_path: P,
    file_name: &str,
) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(file_name);
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

/// Continually goes up in the file tree until a Forc manifest file is found.
pub fn find_parent_manifest_dir<P: AsRef<Path>>(starter_path: P) -> Option<PathBuf> {
    find_parent_dir_with_file(starter_path, constants::MANIFEST_FILE_NAME)
}

/// Continually goes up in the file tree until a Forc manifest file is found and given predicate
/// returns true.
pub fn find_parent_manifest_dir_with_check<T: AsRef<Path>, F>(
    starter_path: T,
    f: F,
) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    find_parent_manifest_dir(starter_path).and_then(|manifest_dir| {
        // If given check satisfies, return current directory, otherwise start searching from the parent.
        if f(&manifest_dir) {
            Some(manifest_dir)
        } else if let Some(parent_dir) = manifest_dir.parent() {
            find_parent_manifest_dir_with_check(parent_dir, f)
        } else {
            None
        }
    })
}
