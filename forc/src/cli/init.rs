use std::fs;

pub(crate) fn init_new_project(project_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // make an new directory for the project
    fs::create_dir_all(format!("{}/src", project_name))?;

    // insert default manifest file
    fs::write(
        format!("{}/Forc.toml", project_name),
        crate::defaults::default_manifest(),
    )?;

    // insert default main function
    fs::write(
        format!("{}/src/main.fm", project_name),
        crate::defaults::default_program(),
    )?;

    Ok(())
}
