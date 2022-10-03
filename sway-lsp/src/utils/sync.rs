// #![allow(dead_code)]
use dashmap::DashMap;
use forc_pkg::{manifest::Dependency, ManifestFile};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::DebouncedEvent;
use std::sync::mpsc::{Receiver, Sender};

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
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
    pub directories: DashMap<Directory, PathBuf>,
    pub notify_join_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl SyncWorkspace {
    pub(crate) fn new() -> Self {
        Self {
            directories: DashMap::new(),
            notify_join_handle: RwLock::new(None),
        }
    }

    pub(crate) fn create_temp_dir_from_url(&self, uri: &Url) {
        // Convert the Uri to a PathBuf
        let manifest_dir = PathBuf::from(uri.path());
        if let Ok(manifest) = ManifestFile::from_dir(&manifest_dir) {
            // strip Forc.toml from the path
            let manifest_dir = manifest.path().parent().unwrap();
            // extract the project name from the path
            let project_name = manifest_dir.file_name().unwrap().to_str().unwrap();

            // Create a new temporary directory that we can clone the current workspace into.
            let p = Builder::new().tempdir().unwrap();
            let temp_dir = p.path().join(project_name);
            eprintln!("path: {:#?}", temp_dir);

            self.directories
                .insert(Directory::Manifest, manifest_dir.to_path_buf());
            self.directories.insert(Directory::Temp, temp_dir);
        }
    }

    pub(crate) fn clone_manifest_dir_to_temp(&self) {
        let manifest_dir = self
            .directories
            .get(&Directory::Manifest)
            .map(|item| item.value().clone())
            .unwrap();
        let temp_dir = self
            .directories
            .get(&Directory::Temp)
            .map(|item| item.value().clone())
            .unwrap();
        copy_dir_contents(manifest_dir, temp_dir);
    }

    // Convert the Url path from the client to point to the same file in our temp folder
    pub(crate) fn workspace_to_temp_url(&self, uri: &Url) -> Result<Url, ()> {
        let path = PathBuf::from(uri.path());
        let manifest_dir = self
            .directories
            .get(&Directory::Manifest)
            .map(|item| item.value().clone())
            .unwrap();
        let temp_dir = self
            .directories
            .get(&Directory::Temp)
            .map(|item| item.value().clone())
            .unwrap();
        let p = path.strip_prefix(manifest_dir).unwrap();
        Url::from_file_path(temp_dir.join(p))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn feature() {
    // 1. watch the manifest directory and check for any save events on Forc.toml
    // 2. deserialize the manifest file and loop through the dependancies
    // 3. check if the dependancy is specifying a 'path'
    // 4. if so, check if the path is relative
    // 5. convert the relative path to an absolute path
    // 6. edit the toml entry using toml_edit with the absolute path
    // 7. save the manifest to temp_dir/Forc.toml

    let current_open_file = Url::from_directory_path(Path::new("/Users/joshuabatty/Documents/rust/fuel/sway/test/src/e2e_vm_tests/test_programs/should_pass/language/doc_comments/src/main.sw")).unwrap();
    let directories: DashMap<Directory, PathBuf> = DashMap::new();
    let dirs = create_temp_dir_from_url(&current_open_file, &directories);

    let temp_manifest_path = directories
        .get(&Directory::Temp)
        .map(|item| item.value().clone())
        .unwrap()
        .join(sway_utils::constants::TEST_MANIFEST_FILE_NAME);

    let manifest_dir = PathBuf::from(current_open_file.path());
    if let Ok(manifest) = ManifestFile::from_dir(&manifest_dir) {
        let manifest_dir = Arc::new(manifest.clone());

        edit_manifest_dependency_paths(&manifest, &temp_manifest_path);

        let jh = tokio::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(10);
            // Setup debouncer. No specific tickrate, max debounce time 2 seconds
            let mut debouncer = new_debouncer(
                std::time::Duration::from_secs(1),
                None,
                move |event| match event {
                    Ok(e) => {
                        tx.blocking_send(e);
                    }
                    Err(_) => (),
                },
            )
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

        //jh.await;
        //jh.abort();
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(1250)).await;
    }
}

fn edit_manifest_dependency_paths(manifest: &ManifestFile, temp_manifest_path: &Path) {
    // Key = name of the dependancy that has been specified will a relative path
    // Value = the absolute path that should be used to overwrite the relateive path
    let mut dependency_map: HashMap<String, PathBuf> = HashMap::new();

    if let Some(deps) = &manifest.dependencies {
        for (name, dep) in deps.iter() {
            if let Dependency::Detailed(details) = dep {
                if let Some(_) = &details.path {
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
                    manifest_toml["dependencies"][&name] =
                        toml_edit::value(abs_path.display().to_string());
                }

                if let Ok(mut file) = File::create(temp_manifest_path) {
                    let _ = file.write_all(manifest_toml.to_string().as_bytes());
                }
            }
        }
    }
}

/// Copy the contents of the current workspace folder into the targer directory
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
