use crate::cli::InitCommand;
use crate::utils::{defaults, program_type::ProgramType::*};
use anyhow::Result;
use forc_util::{println_green, validate_name};
use std::fs;
use std::path::{Path, PathBuf};
use sway_utils::constants;
use tracing::info;

fn print_welcome_message() {
    let read_the_docs = format!(
        "Read the Docs:\n- {}\n- {}\n- {}",
        "Sway Book: https://fuellabs.github.io/sway/latest",
        "Rust SDK Book: https://fuellabs.github.io/fuels-rs/latest",
        "TypeScript SDK: https://github.com/FuelLabs/fuels-ts"
    );

    let join_the_community = format!(
        "Join the Community:\n- Follow us {}
- Ask questions in dev-chat on {}",
        "@SwayLang: https://twitter.com/SwayLang", "Discord: https://discord.com/invite/xfpK4Pe"
    );

    let report_bugs = format!(
        "Report Bugs:\n- {}",
        "Sway Issues: https://github.com/FuelLabs/sway/issues/new"
    );

    let try_forc = "To compile, use `forc build`, and to run tests use `forc test`";

    info!(
        "\n{}\n\n----\n\n{}\n\n{}\n\n{}\n\n",
        try_forc, read_the_docs, join_the_community, report_bugs
    );
}

pub fn init(command: InitCommand) -> Result<()> {
    let project_dir = match &command.path {
        Some(p) => PathBuf::from(p),
        _ => std::env::current_dir().unwrap(),
    };

    let project_name = project_dir.to_str().unwrap().split('/').last().unwrap();

    validate_name(project_name, "project name")?;

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

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_dir).join("src"))?;

    // Insert default manifest file
    match program_type {
        Library => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(project_name, constants::LIB_ENTRY),
        )?,
        _ => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(project_name, constants::MAIN_ENTRY),
        )?,
    }

    match program_type {
        Contract => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_contract(),
        )?,
        Script => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_script(),
        )?,
        Library => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::LIB_ENTRY),
            defaults::default_library(project_name),
        )?,
        Predicate => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_predicate(),
        )?,
    }

    println_green(&format!(
        "Successfully created {program_type}: {project_name}",
    ));

    print_welcome_message();

    Ok(())
}