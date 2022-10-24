use crate::error::{DirectoryError, DocumentError, LanguageServerError};
use dashmap::DashMap;
use forc_pkg::{manifest::Dependency, PackageManifestFile};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::Builder;
use tower_lsp::lsp_types::Url;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Directory {
    Manifest,
    Temp,
}

#[derive(Debug)]
pub struct SyncWorkspace {
    pub directories:        DashMap<Directory, PathBuf>,
    pub notify_join_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl SyncWorkspace {
    pub const LSP_TEMP_PREFIX: &'static str = "SWAY_LSP_TEMP_DIR";

    pub(crate) fn new() -> Self {
        Self {
            directories:        DashMap::new(),
            notify_join_handle: RwLock::new(None),
        }
    }

    /// Clean up the temp directory that was created once the
    /// server closes down.
    pub(crate) fn remove_temp_dir(&self) {
        if let Ok(dir) = self.temp_dir() {
            dir.parent().map(fs::remove_dir);
        }
    }

    pub(crate) fn create_temp_dir_from_workspace(
        &self,
        manifest_dir: &Path,
    ) -> Result<(), LanguageServerError> {
        let manifest = PackageManifestFile::from_dir(manifest_dir).map_err(|_| {
            DocumentError::ManifestFileNotFound {
                dir: manifest_dir.to_string_lossy().to_string(),
            }
        })?;

        // strip Forc.toml from the path to get the manifest directory
        let manifest_dir = manifest
            .path()
            .parent()
            .ok_or(DirectoryError::ManifestDirNotFound)?;

        // extract the project name from the path
        let project_name = manifest_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(DirectoryError::CantExtractProjectName {
                dir: manifest_dir.to_string_lossy().to_string(),
            })?;

        // Create a new temporary directory that we can clone the current workspace into.
        let temp_dir = Builder::new()
            .prefix(SyncWorkspace::LSP_TEMP_PREFIX)
            .tempdir()
            .map_err(|_| DirectoryError::TempDirFailed)?;

        let temp_path = temp_dir
            .into_path()
            .canonicalize()
            .map_err(|_| DirectoryError::CanonicalizeFailed)?
            .join(project_name);

        self.directories
            .insert(Directory::Manifest, manifest_dir.to_path_buf());
        self.directories.insert(Directory::Temp, temp_path);

        Ok(())
    }

    pub(crate) fn clone_manifest_dir_to_temp(&self) -> Result<(), DirectoryError> {
        copy_dir_contents(self.manifest_dir()?, self.temp_dir()?)
            .map_err(|_| DirectoryError::CopyContentsFailed)?;

        Ok(())
    }

    /// Check if the current path is part of the users workspace.
    /// Returns false if the path is from a dependancy
    pub(crate) fn is_path_in_workspace(&self, uri: &Url) -> bool {
        uri.as_ref().contains(SyncWorkspace::LSP_TEMP_PREFIX)
    }

    /// Convert the Url path from the client to point to the same file in our temp folder
    pub(crate) fn workspace_to_temp_url(&self, uri: &Url) -> Result<Url, DirectoryError> {
        self.convert_url(uri, self.temp_dir()?, self.manifest_dir()?)
    }

    /// Convert the Url path from the temp folder to point to the same file in the users workspace
    pub(crate) fn temp_to_workspace_url(&self, uri: &Url) -> Result<Url, DirectoryError> {
        self.convert_url(uri, self.manifest_dir()?, self.temp_dir()?)
    }

    /// If path is part of the users workspace, then convert URL from temp to workspace dir.
    /// Otherwise, pass through if it points to a dependency path
    pub(crate) fn to_workspace_url(&self, url: Url) -> Option<Url> {
        if self.is_path_in_workspace(&url) {
            Some(self.temp_to_workspace_url(&url).ok()?)
        } else {
            Some(url)
        }
    }

    pub(crate) fn temp_manifest_path(&self) -> Option<PathBuf> {
        self.temp_dir()
            .map(|dir| dir.join(sway_utils::constants::MANIFEST_FILE_NAME))
            .ok()
    }

    pub(crate) fn manifest_path(&self) -> Option<PathBuf> {
        self.manifest_dir()
            .map(|dir| dir.join(sway_utils::constants::MANIFEST_FILE_NAME))
            .ok()
    }

    /// Watch the manifest directory and check for any save events on Forc.toml
    pub(crate) fn watch_and_sync_manifest(&self) {
        let _ = self
            .manifest_path()
            .and_then(|manifest_path| PackageManifestFile::from_dir(&manifest_path).ok())
            .map(|manifest| {
                let manifest_dir = Arc::new(manifest.clone());
                if let Some(temp_manifest_path) = self.temp_manifest_path() {
                    edit_manifest_dependency_paths(&manifest, &temp_manifest_path);

                    let handle = tokio::spawn(async move {
                        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
                        // Setup debouncer. No specific tickrate, max debounce time 2 seconds
                        let mut debouncer =
                            new_debouncer(std::time::Duration::from_secs(1), None, move |event| {
                                if let Ok(e) = event {
                                    let _ = tx.blocking_send(e);
                                }
                            })
                            .unwrap();

                        debouncer
                            .watcher()
                            .watch(manifest_dir.as_ref().path(), RecursiveMode::NonRecursive)
                            .unwrap();

                        while let Some(_events) = rx.recv().await {
                            // Rescan the Forc.toml and convert
                            // relative paths to absolute. Save into our temp directory.
                            edit_manifest_dependency_paths(&manifest, &temp_manifest_path);
                        }
                    });

                    // Store the join handle so we can clean up the thread on shutdown
                    {
                        let mut join_handle = self.notify_join_handle.write();
                        *join_handle = Some(handle);
                    }
                }
            });
    }

    pub(crate) fn manifest_dir(&self) -> Result<PathBuf, DirectoryError> {
        self.directories
            .get(&Directory::Manifest)
            .map(|item| item.value().clone())
            .ok_or(DirectoryError::ManifestDirNotFound)
    }

    fn temp_dir(&self) -> Result<PathBuf, DirectoryError> {
        self.directories
            .get(&Directory::Temp)
            .map(|item| item.value().clone())
            .ok_or(DirectoryError::TempDirNotFound)
    }

    fn convert_url(&self, uri: &Url, from: PathBuf, to: PathBuf) -> Result<Url, DirectoryError> {
        let path = from.join(
            PathBuf::from(uri.path())
                .strip_prefix(to)
                .map_err(DirectoryError::StripPrefixError)?,
        );
        self.url_from_path(&path)
    }

    fn url_from_path(&self, path: &PathBuf) -> Result<Url, DirectoryError> {
        Url::from_file_path(&path).map_err(|_| DirectoryError::UrlFromPathFailed {
            path: path.to_string_lossy().to_string(),
        })
    }
}

/// Deserialize the manifest file and loop through the dependancies.
/// Check if the dependancy is specifying a 'path'.
/// If so, check if the path is relative and convert the relative path to an absolute path.
/// Edit the toml entry using toml_edit with the absolute path.
/// Save the manifest to temp_dir/Forc.toml.
pub(crate) fn edit_manifest_dependency_paths(
    manifest: &PackageManifestFile,
    temp_manifest_path: &Path,
) {
    // Key = name of the dependancy that has been specified will a relative path
    // Value = the absolute path that should be used to overwrite the relateive path
    let mut dependency_map: HashMap<String, PathBuf> = HashMap::new();

    if let Some(deps) = &manifest.dependencies {
        for (name, dep) in deps.iter() {
            if let Dependency::Detailed(details) = dep {
                if details.path.is_some() {
                    if let Some(abs_path) = manifest.dep_path(name) {
                        dependency_map.insert(name.clone(), abs_path);
                    }
                }
            }
        }
    }

    if dependency_map.capacity() != 0 {
        if let Ok(mut file) = File::open(manifest.path()) {
            let mut toml = String::new();
            let _ = file.read_to_string(&mut toml);
            if let Ok(mut manifest_toml) = toml.parse::<toml_edit::Document>() {
                for (name, abs_path) in dependency_map {
                    manifest_toml["dependencies"][&name]["path"] =
                        toml_edit::value(abs_path.display().to_string());
                }

                if let Ok(mut file) = File::create(temp_manifest_path) {
                    let _ = file.write_all(manifest_toml.to_string().as_bytes());
                }
            }
        }
    }
}

/// Copy the contents of the current workspace folder into the target directory
fn copy_dir_contents(
    src_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&target_dir)?;
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_contents(entry.path(), target_dir.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), target_dir.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
