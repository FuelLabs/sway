use crate::utils::defaults;
use anyhow::Result;
use std::fs;
use std::path::Path;
use sway_utils::constants;

pub(crate) fn init_new_project(project_name: String) -> Result<()> {
    let neat_name: String = project_name.split('/').last().unwrap().to_string();

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_name).join("src"))?;

    // Make directory for tests
    fs::create_dir_all(Path::new(&project_name).join("tests"))?;

    // Insert default manifest file
    fs::write(
        Path::new(&project_name).join(constants::MANIFEST_FILE_NAME),
        defaults::default_manifest(&neat_name),
    )?;

    // Insert default test manifest file
    fs::write(
        Path::new(&project_name).join(constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&neat_name),
    )?;

    // Insert default main function
    fs::write(
        Path::new(&project_name).join("src").join("main.sw"),
        defaults::default_program(),
    )?;

    // Insert default test function
    fs::write(
        Path::new(&project_name).join("tests").join("harness.rs"),
        defaults::default_test_program(),
    )?;

    // Ignore default `out` and `target` directories created by forc and cargo.
    fs::write(
        Path::new(&project_name).join(".gitignore"),
        defaults::default_gitignore(),
    )?;

    Ok(())
}
