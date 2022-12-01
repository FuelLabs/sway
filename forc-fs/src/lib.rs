use file_lock::{FileLock, FileOptions};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Filesystem {
    root: PathBuf,
}

impl AsRef<Path> for Filesystem {
    fn as_ref(&self) -> &Path {
        self.root.as_path()
    }
}

impl Filesystem {
    /// Creates a new filesystem to be rooted at the given path.
    pub fn new(path: PathBuf) -> Filesystem {
        Filesystem { root: path }
    }

    /// Like `Path::join`, creates a new filesystem rooted at this filesystem
    /// joined with the given path.
    pub fn join<T: AsRef<Path>>(&self, other: T) -> Filesystem {
        Filesystem::new(self.root.join(other))
    }

    pub fn display(&self) -> std::path::Display<'_> {
        self.root.display()
    }

    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// Opens exclusive access to a file, returning the locked version of a
    /// file.
    ///
    /// This function will create a file at `path` if it doesn't already exist
    /// (including intermediate directories), and then it will acquire an
    /// exclusive lock on `path`.
    ///
    /// The returned file can be accessed to look at the path and also has
    /// read/write access to the underlying file.
    pub fn open_rw<P>(&self, path: P) -> std::io::Result<FileLock>
    where
        P: AsRef<Path>,
    {
        let is_blocking = true;
        let options = FileOptions::new().read(true).write(true).create(true);
        FileLock::lock(path, is_blocking, options)
    }
}
