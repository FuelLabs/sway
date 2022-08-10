use crate::cli::InitCommand;
use crate::utils::{defaults, program_type::ProgramType::*};
use anyhow::{Context, Result};
use forc_util::validate_name;
use std::fs;
use std::io::Write;
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
        None => {
            std::env::current_dir().context("Failed to get current directory for forc init.")?
        }
    };

    if !project_dir.is_dir() {
        anyhow::bail!("'{}' is not a valid directory.", project_dir.display());
    }

    if project_dir.join(constants::MANIFEST_FILE_NAME).exists() {
        anyhow::bail!(
            "'{}' already includes a Forc.toml file.",
            project_dir.display()
        );
    }

    if command.verbose {
        info!(
            "\nUsing project directory at {}",
            project_dir.canonicalize()?.display()
        );
    }

    let project_name = match command.name {
        Some(name) => name,
        None => project_dir
            .file_stem()
            .context("Failed to infer project name from directory name.")?
            .to_string_lossy()
            .into_owned(),
    };

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

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_dir).join("src"))?;

    // Make directory for tests
    fs::create_dir_all(Path::new(&project_dir).join("tests"))?;

    // Insert default manifest file
    match program_type {
        Library => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&project_name, constants::LIB_ENTRY),
        )?,
        _ => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_manifest(&project_name, constants::MAIN_ENTRY),
        )?,
    }

    // Insert default test manifest file
    fs::write(
        Path::new(&project_dir).join(constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&project_name),
    )?;

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
            defaults::default_library(&project_name),
        )?,
        Predicate => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_predicate(),
        )?,
    }

    // Insert default test function
    let harness_path = Path::new(&project_dir).join("tests").join("harness.rs");
    fs::write(&harness_path, defaults::default_test_program(&project_name))?;

    if command.verbose {
        info!(
            "\nCreated test harness at {}",
            harness_path.canonicalize()?.display()
        );
    }

    // Ignore default `out` and `target` directories created by forc and cargo.
    let gitignore_path = Path::new(&project_dir).join(".gitignore");
    // Append to existing gitignore if it exists otherwise create a new one.
    let mut gitignore_file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&gitignore_path)?;
    gitignore_file.write_all(defaults::default_gitignore().as_bytes())?;

    if command.verbose {
        info!(
            "\nCreated .gitignore at {}",
            gitignore_path.canonicalize()?.display()
        );
    }

    if command.verbose {
        info!("\nSuccessfully created {program_type}: {project_name}",);
    }

    print_welcome_message();

    Ok(())
}
