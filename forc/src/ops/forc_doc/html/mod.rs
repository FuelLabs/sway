use std::{fs::create_dir_all, path::PathBuf};

use core_lang::HllParseTree;
use maud::Markup;

use crate::{
    ops::forc_doc::html::static_files::{build_css_files, build_font_files},
    utils::cli_error::CliError,
};

use self::page_type::PageType;

mod builder;
mod common;
mod page_type;
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

pub(crate) fn get_page_types(parse_tree: HllParseTree) -> Vec<PageType> {
    traversal::traverse_for_page_types(parse_tree)
}

pub(crate) fn build_index_html(page_types: Vec<PageType>) -> Result<(), CliError> {
    //let structs = page_types.iter().filter(|t| t.is_struct());
    Ok(())
}

pub(crate) fn build_page(page_type: &PageType, main_sidebar: &Markup) -> Result<(), CliError> {
    builder::build_page(page_type, main_sidebar)
}

pub(crate) fn build_main_sidebar(project_name: &str, page_types: &Vec<PageType>) -> Markup {
    let structs: Vec<&PageType> = page_types.iter().filter(|t| t.is_struct()).collect();
    let structs = builder::build_type_sidebar("Structs", structs);

    common::main_sidebar(project_name, vec![structs])
}
