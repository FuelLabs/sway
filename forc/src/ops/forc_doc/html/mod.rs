use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    path::PathBuf,
};

use core_lang::HllParseTree;
use maud::Markup;

use crate::{
    ops::forc_doc::html::static_files::{build_css_files, build_font_files},
    utils::cli_error::CliError,
};

use self::{page_type::SWAY_TYPES, traversal::traverse_ast_node};

mod builder;
mod page_type;
mod static_files;
mod traversal;

pub(crate) fn build_static_files(project_name: &str) -> Result<PathBuf, CliError> {
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

pub(crate) fn build_and_store_markup_body(
    parse_tree: HllParseTree,
    map: &mut HashMap<&str, Vec<(String, Markup)>>,
) {
    if let Some(script_tree) = &parse_tree.script_ast {
        let nodes = &script_tree.root_nodes;

        for node in nodes {
            if let Some(page_type) = traverse_ast_node(&node) {
                let store = map.get_mut(page_type.get_type_key()).unwrap();
                let body = builder::build_body(&page_type);
                let name = page_type.get_name().into();
                store.push((name, body));
            }
        }
    }
}

pub(crate) fn build_page(name: &str, body: Markup, main_sidebar: &Markup) -> Result<(), CliError> {
    let page = builder::build_page(body, main_sidebar);

    let file_name = format!("./{}.html", name);
    let _ = File::create(&file_name)?;
    std::fs::write(&file_name, page.into_string())?;

    Ok(())
}

pub(crate) fn build_main_sidebar(
    project_name: &str,
    markups: &HashMap<&str, Vec<(String, Markup)>>,
) -> Markup {
    let mut res = vec![];

    for (key, value) in markups {
        res.push(builder::build_type_sidebar(key, value))
    }

    builder::main_sidebar(project_name, res)
}

pub(crate) fn initialize_markup_map() -> HashMap<&'static str, Vec<(String, Markup)>> {
    let mut map = HashMap::new();

    for key in SWAY_TYPES {
        map.insert(key, vec![]);
    }

    map
}
