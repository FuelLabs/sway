use crate::cli::NewCommand;
use crate::utils::{
    defaults,
    program_type::{ProgramType, ProgramType::*},
};
use anyhow::Result;
use forc_util::{print_light_blue, print_light_green, validate_name};
use std::fs;
use std::path::Path;
use sway_utils::constants;

fn print_welcome_message() {
    print_light_green("To compile, use `forc build`, and to run tests use `forc test`\n\n");

    print_light_blue("Read the Docs:\n");
    print_light_green("- Sway Book: ");
    print_light_blue("https://fuellabs.github.io/sway/latest\n");
    print_light_green("- Rust SDK Book: ");
    print_light_blue("https://fuellabs.github.io/fuels-rs/latest\n");
    print_light_green("- TypeScript SDK: ");
    print_light_blue("https://github.com/FuelLabs/fuels-ts\n\n");

    print_light_blue("Join the Community:\n");
    print_light_green("- Follow us @SwayLang: ");
    print_light_blue("https://twitter.com/SwayLang\n");
    print_light_green("- Ask questions in dev-chat on Discord: ");
    print_light_blue("https://discord.com/invite/xfpK4Pe\n\n");

    print_light_blue("Report Bugs:\n");
    print_light_green("- Sway Issues: ");
    print_light_blue("https://github.com/FuelLabs/sway/issues/new\n");
}

pub fn init(command: NewCommand) -> Result<()> {
    let project_name_or_path = command.project_name;
    validate_name(&project_name_or_path, "project name")?;

    let program_type = match (
        command.contract,
        command.script,
        command.predicate,
        command.library,
    ) {
        (_, false, false, false) => Contract,
        (false, true, false, false) => Script,
        (false, false, true, false) => Predicate,
        (false, false, false, true) => Library,
        _ => anyhow::bail!(
            "Multiple types detected, please specify only one program type: \
                \n Possible Types:\n - contract\n - script\n - predicate\n - library"
        ),
    };

    init_new_project(project_name_or_path, program_type)
}

pub(crate) fn init_new_project(
    project_name_or_path: String,
    program_type: ProgramType,
) -> Result<()> {
    let neat_name: String = project_name_or_path.split('/').last().unwrap().to_string();

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_name_or_path).join("src"))?;

    // Make directory for tests
    fs::create_dir_all(Path::new(&project_name_or_path).join("tests"))?;

    // Insert default manifest file
    match program_type {
        Library => fs::write(
            Path::new(&project_name_or_path).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&neat_name, constants::LIB_ENTRY),
        )?,
        _ => fs::write(
            Path::new(&project_name_or_path).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&neat_name, constants::MAIN_ENTRY),
        )?,
    }

    // Insert default test manifest file
    fs::write(
        Path::new(&project_name_or_path).join(constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&neat_name),
    )?;

    // Insert src based on program_type
    match program_type {
        Contract => fs::write(
            Path::new(&project_name_or_path)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_contract(),
        )?,
        Script => fs::write(
            Path::new(&project_name_or_path)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_script(),
        )?,
        Library => fs::write(
            Path::new(&project_name_or_path)
                .join("src")
                .join(constants::LIB_ENTRY),
            defaults::default_library(&project_name_or_path),
        )?,
        Predicate => fs::write(
            Path::new(&project_name_or_path)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_predicate(),
        )?,
    }

    // Insert default test function
    fs::write(
        Path::new(&project_name_or_path)
            .join("tests")
            .join("harness.rs"),
        defaults::default_test_program(&project_name_or_path),
    )?;

    // Ignore default `out` and `target` directories created by forc and cargo.
    fs::write(
        Path::new(&project_name_or_path).join(".gitignore"),
        defaults::default_gitignore(),
    )?;

    print_light_green(&format!(
        "\nSuccessfully created {program_type}: {neat_name}\n",
    ));

    print_welcome_message();

    Ok(())
}
