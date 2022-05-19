use crate::cli::TemplateCommand;
use crate::utils::SWAY_GIT_TAG;
use anyhow::{anyhow, Result};
use forc_pkg::{fetch_git, fetch_id, find_dir_within, git_commit_path, pin_git, SourceGit};
use std::env;
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

    let fetch_ts = std::time::Instant::now();
    let fetch_id = fetch_id(&env::current_dir().expect("cant get current dir"), fetch_ts);

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

    match command.template_name {
        Some(_) => {
            let path =
                find_dir_within(&repo_path, &template_name, SWAY_GIT_TAG).ok_or_else(|| {
                    anyhow!(
                        "failed to find package `{}` in {}",
                        template_name,
                        git_source.to_string()
                    )
                })?;
            println!("{:?}", path);
        }
        None => {
            println!("{:?}", repo_path);
        }
    }

    Ok(())
}
