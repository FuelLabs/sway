use crate::cli::InitCommand;
use crate::utils::{
    defaults,
    program_type::{ProgramType, ProgramType::*},
};
use anyhow::Result;
use forc_util::{println_green, validate_name};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use sway_utils::constants;
use tracing::info;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FileType {
    File,
    Dir,
}

// Dead fields required for deserialization.
#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Links {
    git: String,
    html: String,
    #[serde(rename = "self")]
    cur: String,
}

// Dead fields required for deserialization.
#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct ContentResponse {
    #[serde(rename = "_links")]
    links: Links,
    download_url: Option<String>,
    git_url: String,
    html_url: String,
    name: String,
    path: String,
    sha: String,
    size: u64,
    #[serde(rename = "type")]
    file_type: FileType,
    url: String,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct GithubRepoResponse {
    sha: String,
    url: String,
    // We only care about the tree here
    tree: Vec<GithubTree>,
    truncated: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct GithubTree {
    mode: String,
    // We only care about the "path" which are files / directory names
    path: String,
    sha: String,
    size: Option<usize>,
    #[serde(rename = "type")]
    data_type: String,
    url: String,
}

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
    let project_name = command.project_name;
    validate_name(&project_name, "project name")?;

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

    init_new_project(project_name, program_type)
}

pub(crate) fn init_new_project(project_name: String, program_type: ProgramType) -> Result<()> {
    let neat_name: String = project_name.split('/').last().unwrap().to_string();

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_name).join("src"))?;

    // Make directory for tests
    fs::create_dir_all(Path::new(&project_name).join("tests"))?;

    // Insert default manifest file
    match program_type {
        Library => fs::write(
            Path::new(&project_name).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&neat_name, constants::LIB_ENTRY),
        )?,
        _ => fs::write(
            Path::new(&project_name).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&neat_name, constants::MAIN_ENTRY),
        )?,
    }

    // Insert default test manifest file
    fs::write(
        Path::new(&project_name).join(constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&neat_name),
    )?;

    // Insert src based on program_type
    match program_type {
        Contract => fs::write(
            Path::new(&project_name)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_contract(),
        )?,
        Script => fs::write(
            Path::new(&project_name)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_script(),
        )?,
        Library => fs::write(
            Path::new(&project_name)
                .join("src")
                .join(constants::LIB_ENTRY),
            defaults::default_library(&project_name),
        )?,
        Predicate => fs::write(
            Path::new(&project_name)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_predicate(),
        )?,
    }

    // Insert default test function
    fs::write(
        Path::new(&project_name).join("tests").join("harness.rs"),
        defaults::default_test_program(&project_name),
    )?;

    // Ignore default `out` and `target` directories created by forc and cargo.
    fs::write(
        Path::new(&project_name).join(".gitignore"),
        defaults::default_gitignore(),
    )?;

    println_green(&format!(
        "Successfully created {program_type}: {project_name}",
    ));

    print_welcome_message();

    Ok(())
}
