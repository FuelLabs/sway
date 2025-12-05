use crate::error::Result;
use crate::md_pre_process::flatten_markdown;
use crate::validate::validate_dir;
use flate2::write::GzEncoder;
use flate2::Compression;
use forc_diagnostic::println_warning;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Builder;
use tempfile::{tempdir, TempDir};
use walkdir::WalkDir;

const TARBALL_FILE_NAME: &str = "sway-project.tgz";

/// Creates a .tgz tarball from the current directory in a temporary location.
/// Returns the path to the created tarball.
pub fn create_tarball_from_current_dir(temp_tarball_dir: &TempDir) -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    validate_dir(&current_dir)?;

    // Copy project to a temporary directory, excluding `/out/`
    let temp_project_dir = tempdir()?;
    copy_project_excluding_out(temp_project_dir.path())?;

    // Process README.md if it exists
    process_readme(temp_project_dir.path())?;

    // Pack the temp directory into a tarball
    let tarball_path = temp_tarball_dir.path().join(TARBALL_FILE_NAME);
    let tar_gz = File::create(&tarball_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_dir_all(".", &temp_project_dir)?;
    tar.finish()?;

    // Return the tarball path
    Ok(tarball_path)
}

// Process README.md by flattening includes, if it exists
fn process_readme(temp_project_dir: &Path) -> Result<()> {
    let readme_path = temp_project_dir.join("README.md");
    if readme_path.exists() {
        match flatten_markdown(&readme_path) {
            Ok(flattened_content) => {
                fs::write(&readme_path, flattened_content)?;
            }
            Err(e) => {
                // Log warning but don't fail the publish
                println_warning(&format!("Failed to flatten README.md includes: {e}"));
            }
        }
    }
    Ok(())
}

/// Copies the current directory (excluding `/out/`) to a temporary directory.
fn copy_project_excluding_out(temp_project_dir: &Path) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    for entry in WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(&current_dir)?;

        // Skip the `/out` directory
        if relative_path.starts_with("out") {
            continue;
        }

        let new_path = temp_project_dir.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&new_path)?;
        } else {
            fs::copy(path, &new_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Error;
    use flate2::read::GzDecoder;
    use serial_test::serial;
    use std::{env, fs};
    use tar::Archive;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn test_create_tarball_success() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let tests_path = manifest_dir.join("tests").join("data");
        let src_dir = tests_path.join("success_with_no_deps");

        // Create another temporary directory for storing the tarball
        let temp_output_dir = tempdir().unwrap();

        // Run the function
        env::set_current_dir(&src_dir).unwrap();
        let result = create_tarball_from_current_dir(&temp_output_dir);
        assert!(result.is_ok());

        // Check that the tarball file was created
        let tarball_path = result.unwrap();
        assert!(tarball_path.exists());

        // Verify that the tarball contains Forc.toml
        let tar_file = fs::File::open(&tarball_path).unwrap();
        let tar = GzDecoder::new(tar_file);
        let mut archive = Archive::new(tar);

        let mut contains_forc_toml = false;
        for entry in archive.entries().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().unwrap().to_path_buf();
            if path.ends_with("Forc.toml") {
                contains_forc_toml = true;
                break;
            }
        }

        assert!(contains_forc_toml, "Tarball should contain Forc.toml");
    }

    #[test]
    #[serial]
    fn test_create_tarball_fails_without_forc_toml() {
        // Create a temporary directory that DOES NOT contain Forc.toml
        let temp_project_dir = tempdir().unwrap();

        // Create another temporary directory for storing the tarball
        let temp_output_dir = tempdir().unwrap();

        // Run the function, expecting an error
        env::set_current_dir(&temp_project_dir).unwrap();
        let result = create_tarball_from_current_dir(&temp_output_dir);

        assert!(matches!(result, Err(Error::ForcTomlNotFound)));
    }

    #[test]
    #[serial]
    fn test_create_tarball_excludes_out_dir() {
        let temp_project_dir = tempdir().unwrap();

        // Create necessary files
        fs::write(
            temp_project_dir.path().join("Forc.toml"),
            "[project]\nname = \"test_project\"\nversion = \"0.0.0\"\nentry = \"main.sw\"\nlicense=\"Apache-2.0\"",
        )
        .unwrap();
        fs::create_dir(temp_project_dir.path().join("src/")).unwrap();
        fs::write(temp_project_dir.path().join("src/main.sw"), "fn main() {}").unwrap();

        // Create an `out/debug/` directory with a dummy file
        let out_dir = temp_project_dir.path().join("out/debug/");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("compiled.bin"), "binary content").unwrap();

        // Create temp dir for tarball storage
        let temp_output_dir = tempdir().unwrap();

        // Change working directory to our fake project
        std::env::set_current_dir(temp_project_dir.path()).unwrap();

        // Run the function
        let result = create_tarball_from_current_dir(&temp_output_dir);
        assert!(result.is_ok());

        let tarball_path = result.unwrap();
        assert!(tarball_path.exists());

        // Verify that the tarball does NOT contain `out/`
        let tar_file = fs::File::open(&tarball_path).unwrap();
        let tar = GzDecoder::new(tar_file);
        let mut archive = Archive::new(tar);

        let mut contains_forc_toml = false;
        let mut contains_main_sw = false;
        let mut contains_out_dir = false;
        for entry in archive.entries().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().unwrap().to_path_buf();
            if path.starts_with("out") {
                contains_out_dir = true;
            } else if path.ends_with("Forc.toml") {
                contains_forc_toml = true;
            } else if path.ends_with("src/main.sw") {
                contains_main_sw = true;
            }
        }

        assert!(
            !contains_out_dir,
            "Tarball should not contain the `out/` directory"
        );
        assert!(contains_forc_toml, "Tarball should contain Forc.toml");
        assert!(contains_main_sw, "Tarball should contain main.sw");
    }
}
