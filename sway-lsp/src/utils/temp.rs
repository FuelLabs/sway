#![allow(dead_code)]

use forc_pkg::{self as pkg};
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use tower_lsp::lsp_types::*;

#[test]
fn feature() {
    let current_open_file = Url::from_directory_path(Path::new("/Users/joshuabatty/Documents/rust/fuel/sway/test/src/e2e_vm_tests/test_programs/should_pass/language/doc_comments/src/main.sw")).unwrap();
    let dirs = clone_project_to_tmp_dir(current_open_file).unwrap();

    print_project_files(dirs.temp_dir);
}

fn print_project_files(dir: impl AsRef<Path>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        eprintln!("{:?}", entry);
        let ty = entry.file_type().unwrap();
        if ty.is_dir() {
            print_project_files(entry.path());
        }
    }
}

/// Create a new temporary directory that we can clone the current workspace into.
pub(crate) fn create_project_dir(project_name: &str) -> PathBuf {
    let p = Builder::new().tempdir().unwrap();
    let p = p.path().join(project_name);
    p.to_path_buf()
}

/// Copy the contents of the current workspace folder into the targer directory
pub(crate) fn copy_dir_contents(
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

#[derive(Debug, Clone)]
pub struct Directories {
    pub manifest_dir: PathBuf,
    pub temp_dir: PathBuf,
}

pub(crate) fn clone_project_to_tmp_dir(uri: Url) -> Option<Directories> {
    // Convert the Uri to a PathBuf
    let manifest_dir = PathBuf::from(uri.path());
    if let Ok(manifest) = pkg::ManifestFile::from_dir(&manifest_dir) {
        // strip Forc.toml from the path
        let manifest_dir = manifest.path().parent().unwrap();
        // extract the project name from the path
        let project_name = manifest_dir.file_name().unwrap().to_str().unwrap();

        // create a new temp directory and join the project name to the path
        let temp_dir = create_project_dir(project_name);
        eprintln!("path: {:#?}", temp_dir);

        // recursively copy all files and dirs from the manifest_dir to the temp_dif
        copy_dir_contents(&manifest_dir, &temp_dir).unwrap();

        return Some(Directories {
            manifest_dir: manifest_dir.to_path_buf(),
            temp_dir: temp_dir.to_path_buf(),
        });
    }
    None
}
