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
    pub fn lsp<X: AsRef<Path>>(filename: X) -> PidFileLocking {
        Self::new(filename, ".lsp-locks", "lock")
    }

    /// Checks if the given pid is active
    #[cfg(not(target = "windows"))]
    fn is_pid_active(pid: usize) -> bool {
        // Not using sysinfo here because it has compatibility issues with fuel.nix
        // https://github.com/FuelLabs/fuel.nix/issues/64
        use std::process::Command;
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .expect("Failed to execute ps command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.contains(&format!("{} ", pid))
    }

    #[cfg(target = "windows")]
    fn is_pid_active(pid: usize) -> bool {
        // Not using sysinfo here because it has compatibility issues with fuel.nix
        // https://github.com/FuelLabs/fuel.nix/issues/64
        use std::process::Command;
        let output = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output()
            .expect("Failed to execute tasklist command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        // Check if the output contains the PID, indicating the process is active
        output_str.contains(&format!("{}", pid))
    }

    /// Removes the lock file if it is not locked or the process that locked it is no longer active
    pub fn release(&self) -> io::Result<()> {
        if self.is_locked() {
            Err(io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Cannot remove a dirty lock file, it is locked by another process (PID: {:#?})",
                    self.get_locker_pid()
                ),
            ))
        } else {
            self.remove_file()?;
            Ok(())
        }
    }

    fn remove_file(&self) -> io::Result<()> {
        match remove_file(&self.0) {
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(e);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Returns the PID of the owner of the current lock. If the PID is not longer active the lock
    /// file will be removed
    pub fn get_locker_pid(&self) -> Option<usize> {
        let fs = File::open(&self.0);
        if let Ok(mut file) = fs {
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok();
            drop(file);
            if let Ok(pid) = contents.trim().parse::<usize>() {
                return if Self::is_pid_active(pid) {
                    Some(pid)
                } else {
                    let _ = self.remove_file();
                    None
                };
            }
        }
        None
    }

    /// Checks if the current path is owned by any other process. This will return false if there is
    /// no lock file or the current process is the owner of the lock file
    pub fn is_locked(&self) -> bool {
        self.get_locker_pid()
            .map(|pid| pid != (std::process::id() as usize))
            .unwrap_or_default()
    }

    /// Locks the given filepath if it is not already locked
    pub fn lock(&self) -> io::Result<()> {
        self.release()?;
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
        os::unix::fs::MetadataExt,
    };

    #[test]
    fn test_fs_locking_same_process() {
        let x = PidFileLocking::lsp("test");
        assert!(!x.is_locked()); // checks the non-existence of the lock (therefore it is not locked)
        assert!(x.lock().is_ok());
        // The current process is locking "test"
        let x = PidFileLocking::lsp("test");
        assert!(!x.is_locked());
    }

    #[test]
    fn test_legacy() {
        // tests against an empty file (as legacy were creating this files)
        let x = PidFileLocking::lsp("legacy");
        assert!(x.lock().is_ok());
        // lock file exists,
        assert!(metadata(&x.0).is_ok());

        // simulate a stale lock file from legacy (which should be empty)
        let _ = File::create(&x.0).unwrap();
        assert_eq!(metadata(&x.0).unwrap().size(), 0);

        let x = PidFileLocking::lsp("legacy");
        assert!(!x.is_locked());
    }

    #[test]
    fn test_remove() {
        let x = PidFileLocking::lsp("lock");
        assert!(x.lock().is_ok());
        assert!(x.release().is_ok());
        assert!(x.release().is_ok());
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
        assert!(!x.is_locked());
        let e = metadata(&x.0).unwrap_err().kind();
        assert_eq!(e, ErrorKind::NotFound);
    }
}
