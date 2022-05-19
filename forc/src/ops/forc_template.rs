use crate::cli::TemplateCommand;
use crate::ops::forc_init::{edit_cargo_toml, edit_forc_toml};
use crate::utils::{defaults, SWAY_GIT_TAG};
use anyhow::{anyhow, Result};
use forc_pkg::{
    fetch_git, fetch_id, find_dir_within, git_commit_path, pin_git, Manifest, SourceGit,
};
use fs_extra::dir::{copy, CopyOptions};
use std::path::PathBuf;
use std::{env, fs};
use sway_utils::constants;
use url::Url;

pub fn init(command: TemplateCommand) -> Result<()> {
    let template_name = match command.template_name.clone() {
        Some(temp_name) => temp_name,
        None => "DEFAULT_TEMP_NAME".to_string(),
    };

    let source = SourceGit {
        repo: Url::parse(&command.url)?,
        reference: forc_pkg::GitReference::DefaultBranch,
    };

    let current_dir = &env::current_dir().expect("cant get current dir");

    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(current_dir, fetch_ts);

    println!("Resolving the HEAD of {}", source.repo);
    let git_source = pin_git(fetch_id, &template_name, source)?;

    let repo_path = git_commit_path(
        &template_name,
        &git_source.source.repo,
        &git_source.commit_hash,
    );
    if !repo_path.exists() {
        println!("  Fetching {}", git_source.to_string());
        fetch_git(fetch_id, &template_name, &git_source)?;
    }

    let from_path = match command.template_name {
        Some(_) => find_dir_within(&repo_path, &template_name, SWAY_GIT_TAG).ok_or_else(|| {
            anyhow!(
                "failed to find a template `{}` in {}",
                template_name,
                command.url
            )
        })?,
        None => {
            let manifest_path = repo_path.join(constants::MANIFEST_FILE_NAME);
            if Manifest::from_file(&manifest_path, SWAY_GIT_TAG).is_err() {
                anyhow::bail!("failed to find a template in {}", command.url);
            }
            repo_path
        }
    };

    // Create the target dir
    let target_dir = current_dir.join(&command.project_name);

    // Copy contents from template to target dir
    copy_template_to_target(&from_path, &target_dir)?;

    // Edit forc.toml
    edit_forc_toml(&target_dir, &command.project_name, &whoami::realname())?;
    if target_dir.join("test").exists() {
        edit_cargo_toml(&target_dir, &command.project_name, &whoami::realname())?;
    } else {
        // Create the tests directory, harness.rs and Cargo.toml file
        fs::create_dir_all(target_dir.join("tests"))?;

        fs::write(
            target_dir.join("tests").join("harness.rs"),
            defaults::default_test_program(&command.project_name),
        )?;

        fs::write(
            target_dir.join("Cargo.toml"),
            defaults::default_tests_manifest(&command.project_name),
        )?;
    }
    Ok(())
}

fn copy_template_to_target(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(from, to, &copy_options)?;
    Ok(())
}
