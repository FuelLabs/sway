use crate::utils::{constants, defaults};
use std::fs;

pub(crate) fn init_new_project(project_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Make a new directory for the project
    fs::create_dir_all(format!("{}/src", project_name))?;

    // Make directory for tests
    fs::create_dir_all(format!("{}/tests", project_name))?;

    // Insert default manifest file
    fs::write(
        format!("{}/{}", project_name, constants::MANIFEST_FILE_NAME),
        defaults::default_manifest(&project_name),
    )?;

    // Insert default test manifest file
    fs::write(
        format!(
            "{}/tests/{}",
            project_name,
            constants::TEST_MANIFEST_FILE_NAME
        ),
        defaults::default_tests_manifest(&project_name),
    )?;

    // Insert default main function
    fs::write(
        format!("{}/src/main.sw", project_name),
        defaults::default_program(),
    )?;

    // Insert default test function
    fs::write(
        format!("{}/tests/harness.rs", project_name),
        defaults::default_test_program(),
    )?;

    Ok(())
}
