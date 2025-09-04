use crate::{
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::document::{get_path_from_url, get_url_from_path, get_url_from_span},
};
use dashmap::DashMap;
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
use lsp_types::Url;
use std::{
    fs,
    path::{Path, PathBuf},
};
use sway_types::{SourceEngine, Span};
use sway_utils::{
    constants::{LOCK_FILE_NAME, MANIFEST_FILE_NAME},
    SWAY_EXTENSION,
};
use tempfile::Builder;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Directory {
    Manifest,
    Temp,
}

#[derive(Debug, Default)]
pub struct SyncWorkspace {
    pub directories: DashMap<Directory, PathBuf>,
}

impl SyncWorkspace {
    pub const LSP_TEMP_PREFIX: &'static str = "SWAY_LSP_TEMP_DIR";

    pub fn new() -> Self {
        Self::default()
    }

    /// Clean up the temp directory that was created once the server closes down.
    pub fn remove_temp_dir(&self) {
        if let Ok(dir) = self.temp_dir() {
            // The `temp_path` we store is `random_dir/project_name`.
            // So, we need to remove `random_dir` by getting the parent directory.
            if let Some(parent_dir) = dir.parent() {
                if parent_dir.file_name().is_some_and(|name| {
                    name.to_string_lossy()
                        .starts_with(SyncWorkspace::LSP_TEMP_PREFIX)
                }) {
                    if let Err(e) = fs::remove_dir_all(parent_dir) {
                        tracing::warn!("Failed to remove temp base dir {:?}: {}", parent_dir, e);
                    } else {
                        tracing::debug!("Successfully removed temp base dir: {:?}", parent_dir);
                    }
                }
            }
        }
    }

    pub fn create_temp_dir_from_workspace(
        &self,
        actual_workspace_root: &Path,
    ) -> Result<(), LanguageServerError> {
        let root_dir_name = actual_workspace_root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| DirectoryError::CantExtractProjectName {
                dir: actual_workspace_root.to_string_lossy().to_string(),
            })?;

        let temp_dir_guard = Builder::new()
            .prefix(SyncWorkspace::LSP_TEMP_PREFIX)
            .tempdir()
            .map_err(|_| DirectoryError::TempDirFailed)?;

        // Construct the path for our specific workspace clone *inside* the directory managed by temp_dir_guard.
        let temp_workspace_base = temp_dir_guard.path().join(root_dir_name);

        fs::create_dir_all(&temp_workspace_base).map_err(|io_err| {
            tracing::error!(
                "Failed to create subdirectory {:?} in temp: {}",
                temp_workspace_base,
                io_err
            );
            DirectoryError::TempDirFailed
        })?;

        let canonical_manifest_path = actual_workspace_root.canonicalize().map_err(|io_err| {
            tracing::warn!(
                "Failed to canonicalize manifest path {:?}: {}",
                actual_workspace_root,
                io_err
            );
            DirectoryError::CanonicalizeFailed
        })?;

        let canonical_temp_path = temp_workspace_base.canonicalize().map_err(|io_err| {
            tracing::warn!(
                "Failed to canonicalize temp path {:?}: {}",
                temp_workspace_base,
                io_err
            );
            DirectoryError::CanonicalizeFailed
        })?;

        self.directories
            .insert(Directory::Manifest, canonical_manifest_path);
        self.directories
            .insert(Directory::Temp, canonical_temp_path.clone());

        // Consume the guard to disable auto-cleanup.
        let _ = temp_dir_guard.keep();

        tracing::debug!(
            "SyncWorkspace: Manifest dir set to {:?}, Temp dir set to {:?}",
            actual_workspace_root,
            canonical_temp_path
        );

        Ok(())
    }

    pub fn clone_manifest_dir_to_temp(&self) -> Result<(), DirectoryError> {
        copy_dir_contents(self.manifest_dir()?, self.temp_dir()?)
            .map_err(|_| DirectoryError::CopyContentsFailed)?;

        Ok(())
    }

    /// Convert the Url path from the client to point to the same file in our temp folder
    pub fn workspace_to_temp_url(&self, uri: &Url) -> Result<Url, DirectoryError> {
        convert_url(uri, &self.temp_dir()?, &self.manifest_dir()?)
    }

    /// Convert the [Url] path from the temp folder to point to the same file in the users workspace.
    pub(crate) fn temp_to_workspace_url(&self, uri: &Url) -> Result<Url, DirectoryError> {
        convert_url(uri, &self.manifest_dir()?, &self.temp_dir()?)
    }

    /// If it is a path to a temp directory, convert the path in the [Span] to the same file in the user's
    /// workspace. Otherwise, return the span as-is.
    pub(crate) fn temp_to_workspace_span(
        &self,
        source_engine: &SourceEngine,
        span: &Span,
    ) -> Result<Span, DirectoryError> {
        let url = get_url_from_span(source_engine, span)?;
        if is_path_in_temp_workspace(&url) {
            let converted_url = convert_url(&url, &self.manifest_dir()?, &self.temp_dir()?)?;
            let converted_path = get_path_from_url(&converted_url)?;
            let source_id = source_engine.get_source_id(&converted_path);
            let converted_span = Span::new(
                span.src().clone(),
                span.start(),
                span.end(),
                Some(source_id),
            );
            match converted_span {
                Some(span) => Ok(span),
                None => Err(DirectoryError::SpanFromPathFailed {
                    path: converted_path.to_string_lossy().to_string(),
                }),
            }
        } else {
            Ok(span.clone())
        }
    }

    /// If path is part of the users workspace, then convert URL from temp to workspace dir.
    /// Otherwise, pass through if it points to a dependency path
    pub(crate) fn to_workspace_url(&self, url: Url) -> Option<Url> {
        if is_path_in_temp_workspace(&url) {
            Some(self.temp_to_workspace_url(&url).ok()?)
        } else {
            Some(url)
        }
    }

    /// Returns the path to the Forc.toml of the workspace in the temp directory.
    #[allow(dead_code)]
    pub(crate) fn temp_manifest_path(&self) -> Option<PathBuf> {
        self.temp_dir()
            .map(|dir| dir.join(sway_utils::constants::MANIFEST_FILE_NAME))
            .ok()
    }

    /// Returns the path to the Forc.toml of the workspace.
    pub fn workspace_manifest_path(&self) -> Option<PathBuf> {
        self.manifest_dir()
            .map(|dir| dir.join(sway_utils::constants::MANIFEST_FILE_NAME))
            .ok()
    }

    /// Returns the path to the Forc.toml of the workspace member containing the given TEMP URI.
    /// This function assumes the input URI points to a file within the temporary cloned workspace.
    pub(crate) fn member_manifest_path(&self, temp_uri: &Url) -> Option<PathBuf> {
        let file_path_in_temp_member = get_path_from_url(temp_uri).ok()?;
        let temp_workspace_root_dir = self.temp_dir().ok()?;
        let manifest_file = ManifestFile::from_dir(&temp_workspace_root_dir).ok()?;
        match manifest_file {
            ManifestFile::Package(pkg_manifest) => file_path_in_temp_member
                .starts_with(pkg_manifest.dir())
                .then(|| pkg_manifest.path().to_path_buf()),
            ManifestFile::Workspace(ws_manifest) => ws_manifest
                .member_pkg_manifests()
                .ok()?
                .filter_map(Result::ok)
                .find(|member_pkg| file_path_in_temp_member.starts_with(member_pkg.dir()))
                .map(|member_pkg| member_pkg.path().to_path_buf()),
        }
    }

    pub fn member_path(&self, temp_uri: &Url) -> Option<PathBuf> {
        let p = self.member_manifest_path(temp_uri)?;
        let dir = p.parent()?;
        Some(dir.to_path_buf())
    }

    /// Read the Forc.toml and convert relative paths to absolute. Save into our temp directory.
    pub fn sync_manifest(&self) -> Result<(), LanguageServerError> {
        let actual_manifest_dir = self.manifest_dir()?;
        let temp_manifest_dir = self.temp_dir()?;

        // Load the manifest from the *actual* workspace root to determine if it's a package or workspace
        match ManifestFile::from_dir(&actual_manifest_dir) {
            Ok(ManifestFile::Package(pkg_manifest_file)) => {
                let actual_pkg_manifest_path = pkg_manifest_file.path();
                let temp_pkg_manifest_path = temp_manifest_dir.join(
                    actual_pkg_manifest_path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new(MANIFEST_FILE_NAME)),
                );
                tracing::debug!(
                    "Syncing single package manifest: {:?} to temp: {:?}",
                    actual_pkg_manifest_path,
                    temp_pkg_manifest_path
                );
                edit_manifest_dependency_paths(
                    pkg_manifest_file.dir(),
                    actual_pkg_manifest_path,
                    &temp_pkg_manifest_path,
                )?;
            }
            Ok(ManifestFile::Workspace(ws_manifest_file)) => {
                // Workspace: iterate through members and sync each member's manifest
                tracing::debug!("Syncing workspace members in: {:?}", actual_manifest_dir);
                match ws_manifest_file.member_pkg_manifests() {
                    Ok(member_manifests_iter) => {
                        for member_result in member_manifests_iter {
                            match member_result {
                                Ok(actual_member_pkg_manifest) => {
                                    let actual_member_manifest_path =
                                        actual_member_pkg_manifest.path();
                                    if let Ok(relative_member_path) = actual_member_manifest_path
                                        .strip_prefix(&actual_manifest_dir)
                                    {
                                        let temp_member_manifest_path =
                                            temp_manifest_dir.join(relative_member_path);

                                        tracing::debug!(
                                            "Syncing workspace member manifest: {:?} to temp: {:?}",
                                            actual_member_manifest_path,
                                            temp_member_manifest_path
                                        );
                                        edit_manifest_dependency_paths(
                                            actual_member_pkg_manifest.dir(),
                                            actual_member_manifest_path,
                                            &temp_member_manifest_path,
                                        )?;
                                    } else {
                                        tracing::error!(
                                            "Could not determine relative path for member: {:?}",
                                            actual_member_manifest_path
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to load workspace member manifest: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to get member manifests for workspace {:?}: {}",
                            actual_manifest_dir,
                            e
                        );
                    }
                }

                // Sync the root workspace Forc.toml itself
                let actual_root_workspace_toml_path = ws_manifest_file.path();
                let temp_root_workspace_toml_path = temp_manifest_dir.join(
                    actual_root_workspace_toml_path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new(MANIFEST_FILE_NAME)),
                );
                tracing::debug!(
                    "Syncing root workspace manifest for patches: {:?} to temp: {:?}",
                    actual_root_workspace_toml_path,
                    temp_root_workspace_toml_path
                );
                edit_manifest_dependency_paths(
                    ws_manifest_file.dir(),
                    actual_root_workspace_toml_path,
                    &temp_root_workspace_toml_path,
                )?;
            }
            Err(e) => {
                tracing::error!(
                    "Failed to load manifest from actual directory {:?}: {}. Cannot sync manifest.",
                    actual_manifest_dir,
                    e
                );
            }
        }
        Ok(())
    }

    /// Return the path to the projects manifest directory.
    pub(crate) fn manifest_dir(&self) -> Result<PathBuf, DirectoryError> {
        self.directories
            .try_get(&Directory::Manifest)
            .try_unwrap()
            .map(|item| item.value().clone())
            .ok_or(DirectoryError::ManifestDirNotFound)
    }

    /// Return the path to the temporary directory that was created for the current session.
    pub(crate) fn temp_dir(&self) -> Result<PathBuf, DirectoryError> {
        self.directories
            .try_get(&Directory::Temp)
            .try_unwrap()
            .map(|item| item.value().clone())
            .ok_or(DirectoryError::TempDirNotFound)
    }
}

/// Check if the current path is part of the users workspace.
/// Returns false if the path is from a dependency
pub(crate) fn is_path_in_temp_workspace(uri: &Url) -> bool {
    uri.as_ref().contains(SyncWorkspace::LSP_TEMP_PREFIX)
}

fn convert_url(uri: &Url, from: &Path, to: &PathBuf) -> Result<Url, DirectoryError> {
    let path = from.join(
        PathBuf::from(uri.path())
            .strip_prefix(to)
            .map_err(DirectoryError::StripPrefixError)?,
    );
    get_url_from_path(&path)
}

/// Deserialize the manifest file and loop through the dependencies.
/// Check if the dependency is specifying a 'path'.
/// If so, check if the path is relative and convert the relative path to an absolute path.
/// Edit the toml entry using toml_edit with the absolute path.
/// Save the manifest to temp_dir/Forc.toml.
pub(crate) fn edit_manifest_dependency_paths(
    manifset_dir: &Path,
    manifest_path: &Path,
    temp_manifest_path: &Path,
) -> Result<(), LanguageServerError> {
    // Read and parse the original manifest
    let manifest_content =
        std::fs::read_to_string(manifest_path).map_err(|err| DocumentError::IOError {
            path: manifest_path.to_string_lossy().to_string(),
            error: err.to_string(),
        })?;

    let mut doc = manifest_content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|err| DocumentError::IOError {
            path: manifest_path.to_string_lossy().to_string(),
            error: format!("Failed to parse TOML: {err}"),
        })?;

    let manifest =
        ManifestFile::from_file(manifest_path).map_err(|err| DocumentError::IOError {
            path: manifest_path.to_string_lossy().to_string(),
            error: err.to_string(),
        })?;

    if let ManifestFile::Package(package) = manifest {
        // Process dependencies if they exist
        if let Some(deps) = &package.dependencies {
            if let Some(deps_table) = doc.get_mut("dependencies").and_then(|v| v.as_table_mut()) {
                process_dependencies(manifset_dir, deps, deps_table)?;
            }
        }
    }

    // Write the updated manifest to the temp file
    std::fs::write(temp_manifest_path, doc.to_string()).map_err(|err| {
        DocumentError::UnableToWriteFile {
            path: temp_manifest_path.to_string_lossy().to_string(),
            err: err.to_string(),
        }
    })?;

    Ok(())
}

/// Process dependencies and convert relative paths to absolute
fn process_dependencies(
    manifest_dir: &Path,
    deps: &std::collections::BTreeMap<String, forc_pkg::manifest::Dependency>,
    deps_table: &mut toml_edit::Table,
) -> Result<(), LanguageServerError> {
    for (name, dependency) in deps {
        if let forc_pkg::manifest::Dependency::Detailed(details) = dependency {
            if let Some(rel_path) = &details.path {
                // Convert relative path to absolute
                let abs_path = manifest_dir
                    .join(rel_path)
                    .canonicalize()
                    .map_err(|_| DirectoryError::CanonicalizeFailed)?
                    .to_string_lossy()
                    .to_string();

                // Update the path in the TOML document
                if let Some(dep_item) = deps_table.get_mut(name) {
                    let path_value = toml_edit::Value::from(abs_path);
                    if let Some(table) = dep_item.as_inline_table_mut() {
                        table.insert("path", path_value);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Copies only the specified files from the source directory to the target directory.
/// This function targets files ending with `.sw`, and the specific files `Forc.toml` and `Forc.lock`.
/// It returns `Ok(true)` if any relevant files were copied over, and `Ok(false)` if no such files were found.
fn copy_dir_contents(
    src_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> std::io::Result<bool> {
    let mut has_relevant_files = false;
    for entry in fs::read_dir(&src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ty = entry.file_type()?;
        if ty.is_dir() {
            // Recursively check the directory; if it has relevant files, create the target directory
            if copy_dir_contents(&path, target_dir.as_ref().join(entry.file_name()))? {
                has_relevant_files = true;
            }
        } else if let Some(file_name_os) = path.file_name() {
            if let Some(file_name) = file_name_os.to_str() {
                if file_name.ends_with(&format!(".{SWAY_EXTENSION}"))
                    || file_name == MANIFEST_FILE_NAME
                    || file_name == LOCK_FILE_NAME
                {
                    if !has_relevant_files {
                        fs::create_dir_all(&target_dir)?;
                        has_relevant_files = true;
                    }
                    fs::copy(&path, target_dir.as_ref().join(file_name))?;
                }
            }
        }
    }
    Ok(has_relevant_files)
}
