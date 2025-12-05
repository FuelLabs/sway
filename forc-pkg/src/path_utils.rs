//! Path-related utilities for forc package management.

use anyhow::{bail, Context, Result};
use std::{
    collections::hash_map,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use forc_util::user_forc_directory;

pub fn find_file_name<'sc>(manifest_dir: &Path, entry_path: &'sc Path) -> Result<&'sc Path> {
    let mut file_path = manifest_dir.to_path_buf();
    file_path.pop();
    let file_name = match entry_path.strip_prefix(file_path.clone()) {
        Ok(o) => o,
        Err(err) => bail!(err),
    };
    Ok(file_name)
}

/// Simple function to convert kebab-case to snake_case.
pub fn kebab_to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}

/// The location at which `forc` will checkout git repositories.
pub fn git_checkouts_directory() -> PathBuf {
    user_forc_directory().join("git").join("checkouts")
}

/// Given a path to a directory we wish to lock, produce a path for an associated lock file.
///
/// Note that the lock file itself is simply a placeholder for co-ordinating access. As a result,
/// we want to create the lock file if it doesn't exist, but we can never reliably remove it
/// without risking invalidation of an existing lock. As a result, we use a dedicated, hidden
/// directory with a lock file named after the checkout path.
///
/// Note: This has nothing to do with `Forc.lock` files, rather this is about fd locks for
/// coordinating access to particular paths (e.g. git checkout directories).
fn fd_lock_path<X: AsRef<Path>>(path: X) -> PathBuf {
    const LOCKS_DIR_NAME: &str = ".locks";
    const LOCK_EXT: &str = "forc-lock";
    let file_name = hash_path(path);
    user_forc_directory()
        .join(LOCKS_DIR_NAME)
        .join(file_name)
        .with_extension(LOCK_EXT)
}

/// Hash the path to produce a file-system friendly file name.
/// Append the file stem for improved readability.
fn hash_path<X: AsRef<Path>>(path: X) -> String {
    let path = path.as_ref();
    let mut hasher = hash_map::DefaultHasher::default();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    let file_name = match path.file_stem().and_then(|s| s.to_str()) {
        None => format!("{hash:X}"),
        Some(stem) => format!("{hash:X}-{stem}"),
    };
    file_name
}

/// Create an advisory lock over the given path.
///
/// See [fd_lock_path] for details.
pub fn path_lock<X: AsRef<Path>>(path: X) -> Result<fd_lock::RwLock<File>> {
    let lock_path = fd_lock_path(path);
    let lock_dir = lock_path
        .parent()
        .expect("lock path has no parent directory");
    std::fs::create_dir_all(lock_dir).context("failed to create forc advisory lock directory")?;
    let lock_file = File::create(&lock_path).context("failed to create advisory lock file")?;
    Ok(fd_lock::RwLock::new(lock_file))
}
