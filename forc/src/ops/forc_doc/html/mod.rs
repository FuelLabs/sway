use std::{fs::create_dir_all, path::PathBuf};

use core_lang::HllParseTree;

use crate::{
    ops::forc_doc::html::static_files::{build_css_files, build_font_files},
    utils::cli_error::CliError,
};

mod builder;
mod common;
mod static_files;
mod traversal;

pub fn build_static_files(project_name: &str) -> Result<PathBuf, CliError> {
    // Build static folders
    let mut project_path = PathBuf::from(format!("{}-doc", project_name));

    project_path.push("static");
    project_path.push("fonts");
    create_dir_all(&project_path)?;

    project_path.pop();
    project_path.push("css");
    create_dir_all(&project_path)?;

    project_path.pop();
    project_path.pop();

    // build css and fonts
    build_css_files(&project_path)?;
    build_font_files(&project_path)?;

    Ok(project_path)
}

pub fn build_from_tree(parse_tree: HllParseTree) -> Result<(), String> {
    traversal::traverse_and_build(parse_tree)
}
