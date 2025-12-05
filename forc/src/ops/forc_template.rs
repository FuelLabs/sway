use crate::cli::TemplateCommand;
use anyhow::{anyhow, Context, Result};
use forc_pkg::{
    manifest::{self, PackageManifest},
    source::{self, git::Url},
};
use forc_diagnostic::println_action_green;
use forc_pkg::validation::validate_project_name;
use fs_extra::dir::{copy, CopyOptions};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, str::FromStr};
use sway_utils::constants;

pub fn init(command: TemplateCommand) -> Result<()> {
    validate_project_name(&command.project_name)?;
    // The name used for the temporary local repo directory used for fetching the template.
    let local_repo_name = command
        .template_name
        .clone()
        .unwrap_or_else(|| format!("{}-template-source", command.project_name));

    let source = source::git::Source {
        repo: Url::from_str(&command.url)?,
        reference: source::git::Reference::DefaultBranch,
    };

    let current_dir = &env::current_dir()?;

    let fetch_ts = std::time::Instant::now();
    let fetch_id = source::fetch_id(current_dir, fetch_ts);

    println_action_green("Resolving", &format!("the HEAD of {}", source.repo));
    let git_source = source::git::pin(fetch_id, &local_repo_name, source)?;

    let repo_path = source::git::commit_path(
        &local_repo_name,
        &git_source.source.repo,
        &git_source.commit_hash,
    );
    if !repo_path.exists() {
        println_action_green("Fetching", git_source.to_string().as_str());
        source::git::fetch(fetch_id, &local_repo_name, &git_source)?;
    }

    let from_path = match command.template_name {
        Some(ref template_name) => manifest::find_dir_within(&repo_path, template_name)
            .ok_or_else(|| {
                anyhow!(
                    "failed to find a template `{}` in {}",
                    template_name,
                    command.url
                )
            })?,
        None => {
            let manifest_path = repo_path.join(constants::MANIFEST_FILE_NAME);
            if PackageManifest::from_file(manifest_path).is_err() {
                anyhow::bail!("failed to find a template in {}", command.url);
            }
            repo_path
        }
    };

    // Create the target dir
    let target_dir = current_dir.join(&command.project_name);

    println_action_green(
        "Creating",
        &format!("{} from template", &command.project_name),
    );
    // Copy contents from template to target dir
    copy_template_to_target(&from_path, &target_dir)?;

    // Edit forc.toml
    edit_forc_toml(&target_dir, &command.project_name, &whoami::realname())?;
    if target_dir.join("test").exists() {
        edit_cargo_toml(&target_dir, &command.project_name, &whoami::realname())?;
    }
    Ok(())
}

fn edit_forc_toml(out_dir: &Path, project_name: &str, real_name: &str) -> Result<()> {
    let mut file = File::open(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;
    let mut manifest_toml = toml.parse::<toml_edit::DocumentMut>()?;

    let mut authors = Vec::new();
    let forc_toml: toml::Value = toml::de::from_str(&toml)?;
    if let Some(table) = forc_toml.as_table() {
        if let Some(package) = table.get("project") {
            // If authors Vec is currently populated use that
            if let Some(toml::Value::Array(authors_vec)) = package.get("authors") {
                for author in authors_vec {
                    if let toml::value::Value::String(name) = &author {
                        authors.push(name.clone());
                    }
                }
            }
        }
    }

    // Only append the users name to the authors field if it isn't already in the list
    if authors.iter().any(|e| e != real_name) {
        authors.push(real_name.to_string());
    }

    let authors: toml_edit::Array = authors.iter().collect();
    manifest_toml["project"]["authors"] = toml_edit::value(authors);
    manifest_toml["project"]["name"] = toml_edit::value(project_name);

    // Remove explicit std entry from copied template
    if let Some(project) = manifest_toml.get_mut("dependencies") {
        let _ = project
            .as_table_mut()
            .context("Unable to get forc manifest as table")?
            .remove("std");
    }

    let mut file = File::create(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn edit_cargo_toml(out_dir: &Path, project_name: &str, real_name: &str) -> Result<()> {
    let mut file = File::open(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    let mut updated_authors = toml_edit::Array::default();

    let cargo_toml: toml::Value = toml::de::from_str(&toml)?;
    if let Some(table) = cargo_toml.as_table() {
        if let Some(package) = table.get("package") {
            if let Some(toml::Value::Array(authors_vec)) = package.get("authors") {
                for author in authors_vec {
                    if let toml::value::Value::String(name) = &author {
                        updated_authors.push(name);
                    }
                }
            }
        }
    }
    updated_authors.push(real_name);

    let mut manifest_toml = toml.parse::<toml_edit::DocumentMut>()?;
    manifest_toml["package"]["authors"] = toml_edit::value(updated_authors);
    manifest_toml["package"]["name"] = toml_edit::value(project_name);

    let mut file = File::create(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn copy_template_to_target(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(from, to, &copy_options)?;
    Ok(())
}
