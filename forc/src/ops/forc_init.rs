use crate::cli::InitCommand;
use crate::utils::{defaults, program_type::ProgramType};
use anyhow::Context;
use forc_pkg::validation::validate_project_name;
use forc_types::{forc_result_bail, ForcResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use sway_utils::constants;
use tracing::{debug, info};

#[derive(Debug)]
enum InitType {
    Package(ProgramType),
    Workspace,
}

fn print_welcome_message() {
    let read_the_docs = format!(
        "Read the Docs:\n- {}\n- {}\n- {}\n- {}",
        "Sway Book: https://docs.fuel.network/docs/sway",
        "Forc Book: https://docs.fuel.network/docs/forc",
        "Rust SDK Book: https://docs.fuel.network/docs/fuels-rs",
        "TypeScript SDK: https://docs.fuel.network/docs/fuels-ts"
    );

    let join_the_community = format!(
        "Join the Community:\n- Follow us {}
- Ask questions on {}",
        "@SwayLang: https://twitter.com/SwayLang", "Discourse: https://forum.fuel.network/"
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

pub fn init(command: InitCommand) -> ForcResult<()> {
    let project_dir = match &command.path {
        Some(p) => PathBuf::from(p),
        None => {
            std::env::current_dir().context("Failed to get current directory for forc init.")?
        }
    };

    if !project_dir.is_dir() {
        forc_result_bail!(format!(
            "'{}' is not a valid directory.",
            project_dir.display()
        ),);
    }

    if project_dir.join(constants::MANIFEST_FILE_NAME).exists() {
        forc_result_bail!(
            "'{}' already includes a Forc.toml file.",
            project_dir.display()
        );
    }

    debug!(
        "\nUsing project directory at {}",
        project_dir.canonicalize()?.display()
    );

    let project_name = match command.name {
        Some(name) => name,
        None => project_dir
            .file_stem()
            .context("Failed to infer project name from directory name.")?
            .to_string_lossy()
            .into_owned(),
    };

    validate_project_name(&project_name)?;

    let init_type = match (
        command.contract,
        command.script,
        command.predicate,
        command.library,
        command.workspace,
    ) {
        (_, false, false, false, false) => InitType::Package(ProgramType::Contract),
        (false, true, false, false, false) => InitType::Package(ProgramType::Script),
        (false, false, true, false, false) => InitType::Package(ProgramType::Predicate),
        (false, false, false, true, false) => InitType::Package(ProgramType::Library),
        (false, false, false, false, true) => InitType::Workspace,
        _ => {
            forc_result_bail!(
                "Multiple types detected, please specify only one initialization type: \
        \n Possible Types:\n - contract\n - script\n - predicate\n - library\n - workspace"
            )
        }
    };

    // Make a new directory for the project
    let dir_to_create = match init_type {
        InitType::Package(_) => project_dir.join("src"),
        InitType::Workspace => project_dir.clone(),
    };
    fs::create_dir_all(dir_to_create)?;

    // Insert default manifest file
    match init_type {
        InitType::Workspace => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_workspace_manifest(),
        )?,
        InitType::Package(ProgramType::Library) => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            // Library names cannot have `-` in them because the Sway compiler does not allow that.
            // Even though this is technically not a problem in the toml file, we replace `-` with
            // `_` here as well so that the library name in the Sway file matches the one in
            // `Forc.toml`
            defaults::default_pkg_manifest(&project_name.replace('-', "_"), constants::LIB_ENTRY),
        )?,
        _ => fs::write(
            Path::new(&project_dir).join(constants::MANIFEST_FILE_NAME),
            defaults::default_pkg_manifest(&project_name, constants::MAIN_ENTRY),
        )?,
    }

    match init_type {
        InitType::Package(ProgramType::Contract) => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_contract(),
        )?,
        InitType::Package(ProgramType::Script) => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_script(),
        )?,
        InitType::Package(ProgramType::Library) => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::LIB_ENTRY),
            // Library names cannot have `-` in them because the Sway compiler does not allow that
            defaults::default_library(),
        )?,
        InitType::Package(ProgramType::Predicate) => fs::write(
            Path::new(&project_dir)
                .join("src")
                .join(constants::MAIN_ENTRY),
            defaults::default_predicate(),
        )?,
        _ => {}
    }

    // Ignore default `out` and `target` directories created by forc and cargo.
    let gitignore_path = Path::new(&project_dir).join(".gitignore");
    // Append to existing gitignore if it exists otherwise create a new one.
    let mut gitignore_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&gitignore_path)?;
    gitignore_file.write_all(defaults::default_gitignore().as_bytes())?;

    debug!(
        "\nCreated .gitignore at {}",
        gitignore_path.canonicalize()?.display()
    );

    debug!("\nSuccessfully created {init_type:?}: {project_name}",);

    print_welcome_message();

    Ok(())
}
