use crate::cli::TemplateCommand;
use anyhow::{anyhow, Context, Result};
use forc_pkg::{
    fetch_git, fetch_id, find_dir_within, git_commit_path, pin_git, PackageManifest, SourceGit,
};
use forc_util::validate_name;
use fs_extra::dir::{copy, CopyOptions};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use sway_utils::constants;
use tracing::info;
use url::Url;

pub fn init(command: TemplateCommand) -> Result<()> {
    validate_name(&command.project_name, "project name")?;
    // The name used for the temporary local repo directory used for fetching the template.
    let local_repo_name = command
        .template_name
        .clone()
        .unwrap_or_else(|| format!("{}-template-source", command.project_name));

    let source = SourceGit {
        repo: Url::parse(&command.url)?,
        reference: forc_pkg::GitReference::DefaultBranch,
    };

    let current_dir = &env::current_dir()?;

    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(current_dir, fetch_ts);

    info!("Resolving the HEAD of {}", source.repo);
    let git_source = pin_git(fetch_id, &local_repo_name, source)?;

    let repo_path = git_commit_path(
        &local_repo_name,
        &git_source.source.repo,
        &git_source.commit_hash,
    );
    if !repo_path.exists() {
        info!("  Fetching {}", git_source.to_string());
        fetch_git(fetch_id, &local_repo_name, &git_source)?;
    }

    let from_path = match command.template_name {
        Some(ref template_name) => find_dir_within(&repo_path, template_name).ok_or_else(|| {
            anyhow!(
                "failed to find a template `{}` in {}",
                template_name,
                command.url
            )
        })?,
        None => {
            let manifest_path = repo_path.join(constants::MANIFEST_FILE_NAME);
            // TODO: Remove old `OLD_MANIFEST_FILE_NAME` once deprecation period over.
            let old_manifest_path = repo_path.join(constants::OLD_MANIFEST_FILE_NAME);
            if PackageManifest::from_file(&manifest_path).is_err()
                && PackageManifest::from_file(&old_manifest_path).is_err()
            {
                anyhow::bail!("failed to find a template in {}", command.url);
            }
            repo_path
        }
    };

    // Create the target dir
    let target_dir = current_dir.join(&command.project_name);
    let forc_manifest_path: PathBuf = forc_util::find_manifest_file(&target_dir)
        .ok_or_else(|| anyhow!("target directory missing forc manifest"))?;

    info!("Creating {} from template", &command.project_name);
    // Copy contents from template to target dir
    copy_template_to_target(&from_path, &target_dir)?;

    // Edit `forc.toml
    let realname = whoami::realname();
    edit_forc_toml(&forc_manifest_path, &command.project_name, &realname)?;
    if target_dir.join("test").exists() {
        let cargo_manifest_path = target_dir.join(constants::TEST_MANIFEST_FILE_NAME);
        edit_cargo_toml(&cargo_manifest_path, &command.project_name, &realname)?;
    }
    Ok(())
}

const PACKAGE: &str = "package";
const PROJECT: &str = "project";
const AUTHORS: &str = "authors";
const NAME: &str = "name";
const DEPENDENCIES: &str = "dependencies";

fn edit_toml<F, O>(toml_path: &Path, mut edit: F) -> Result<O>
where
    F: FnMut(&mut toml_edit::Document) -> Result<O>,
{
    let mut file = File::open(toml_path)?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;
    let mut toml_doc = toml.parse::<toml_edit::Document>()?;
    let res = edit(&mut toml_doc)?;
    file.write_all(toml_doc.to_string().as_bytes())?;
    Ok(res)
}

fn edit_forc_toml(manifest_path: &Path, project_name: &str, real_name: &str) -> Result<()> {
    edit_toml(manifest_path, |manifest_toml| {
        use toml_edit::{Item, Value};

        // If authors Vec is populated use that.
        let mut new_authors = Vec::new();
        let table = manifest_toml.as_table();
        if let Some(Item::Table(project)) = table.get(PROJECT) {
            if let Some(Item::Value(Value::Array(authors))) = project.get(AUTHORS) {
                for author in authors {
                    if let Value::String(name) = &author {
                        new_authors.push(name.to_string());
                    }
                }
            }
        }

        // Only append the users name to the authors field if it isn't already in the list
        if new_authors.iter().any(|e| e != real_name) {
            new_authors.push(real_name.to_string());
        }

        let authors: toml_edit::Array = new_authors.iter().collect();
        manifest_toml[PROJECT][AUTHORS] = toml_edit::value(authors);
        manifest_toml[PROJECT][NAME] = toml_edit::value(project_name);

        // Remove explicit std entry from copied template
        if let Some(project) = manifest_toml.get_mut(DEPENDENCIES) {
            let _ = project
                .as_table_mut()
                .context("Unable to get forc manifest as table")?
                .remove("std");
        }
        Ok(())
    })
}

fn edit_cargo_toml(manifest_path: &Path, project_name: &str, real_name: &str) -> Result<()> {
    edit_toml(manifest_path, |manifest_toml| {
        use toml_edit::{Item, Value};

        // If authors Vec is populated use that.
        let mut new_authors = Vec::new();
        let table = manifest_toml.as_table();
        if let Some(Item::Table(project)) = table.get(PACKAGE) {
            if let Some(Item::Value(Value::Array(authors))) = project.get(AUTHORS) {
                for author in authors {
                    if let Value::String(name) = &author {
                        new_authors.push(name.to_string());
                    }
                }
            }
        }
        new_authors.push(real_name.to_string());

        let authors: toml_edit::Array = new_authors.iter().collect();
        manifest_toml[PACKAGE][AUTHORS] = toml_edit::value(authors);
        manifest_toml[PACKAGE][NAME] = toml_edit::value(project_name);
        Ok(())
    })
}

fn copy_template_to_target(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(from, to, &copy_options)?;
    Ok(())
}
