use crate::{hash_path, user_forc_directory};
use std::{
    fs::{create_dir_all, remove_file, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

/// Very simple AdvisoryPathMutex class
///
/// The goal of this struct is to signal other processes that a path is being used by another
/// process exclusively.
///
/// This struct will self-heal if the process that locked the file is no longer running.
pub struct PidFileLocking(PathBuf);

impl PidFileLocking {
    pub fn new<X: AsRef<Path>, Y: AsRef<Path>>(
        filename: X,
        dir: Y,
        extension: &str,
    ) -> PidFileLocking {
        let file_name = hash_path(filename);
        Self(
            user_forc_directory()
                .join(dir)
                .join(file_name)
                .with_extension(extension),
        )
    }

    /// Create a new PidFileLocking instance that is shared between the LSP and any other process
    /// that may want to update the file and needs to wait for the LSP to finish (like forc-fmt)
    pub fn lsp<X: AsRef<Path>>(path: X) -> PidFileLocking {
        Self::new(path, ".lsp-locks", "dirty")
    }

    /// Checks if the given pid is active
    fn is_pid_active(pid: usize) -> bool {
        use sysinfo::{Pid, System};
        if pid == std::process::id() as usize {
            return false;
        }
        System::new_all().process(Pid::from(pid)).is_some()
    }

    /// Removes the lock file if it is not locked or the process that locked it is no longer active
    pub fn remove(&self) -> io::Result<()> {
        if self.is_locked()? {
            Err(io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot remove a dirty lock file, it is locked by another process",
            ))
        } else {
            self.remove_file()
        }
    }

    /// A thin wrapper on top of std::fs::remove_file that does not error if the file does not exist
    fn remove_file(&self) -> io::Result<()> {
        match remove_file(&self.0) {
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(error)
                }
            }
            _ => Ok(()),
        }
    }

    /// Checks if the given filepath is locked by any process
    pub fn is_locked(&self) -> io::Result<bool> {
        let fs = File::open(&self.0);
        match fs {
            Ok(mut file) => {
                let mut pid = String::new();
                file.read_to_string(&mut pid)?;
                let is_locked = pid
                    .trim()
                    .parse::<usize>()
                    .map(Self::is_pid_active)
                    .unwrap_or_default();
                drop(file);
                if !is_locked {
                    self.remove_file()?;
                }
                Ok(is_locked)
            }
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    Ok(false)
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Locks the given filepath if it is not already locked
    pub fn lock(&self) -> io::Result<()> {
        self.remove()?;
        if let Some(dir) = self.0.parent() {
            // Ensure the directory exists
            create_dir_all(dir)?;
        }

        let mut fs = File::create(&self.0)?;
        fs.write_all(std::process::id().to_string().as_bytes())?;
        fs.sync_all()?;
        fs.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::PidFileLocking;
    use std::{
        fs::{metadata, File},
        io::{ErrorKind, Write},
    };

    #[test]
    fn test_fs_locking_same_process() {
        let x = PidFileLocking::lsp("test");
        assert!(x.lock().is_ok());
        // The current process is locking "test"
        let x = PidFileLocking::lsp("test");
        assert!(!x.is_locked().unwrap());
    }

    #[test]
    fn test_fs_locking_stale() {
        let x = PidFileLocking::lsp("stale");
        assert!(x.lock().is_ok());

        // lock file exists,
        assert!(metadata(&x.0).is_ok());

        // simulate a stale lock file
        let mut x = File::create(&x.0).unwrap();
        x.write_all(b"191919191919").unwrap();
        x.flush().unwrap();
        drop(x);

        // PID=191919191919 does not exists, hopefully, and this should remove the lock file
        let x = PidFileLocking::lsp("stale");
        assert!(!x.is_locked().unwrap());
        let e = metadata(&x.0).unwrap_err().kind();
        assert_eq!(e, ErrorKind::NotFound);
    }
}
