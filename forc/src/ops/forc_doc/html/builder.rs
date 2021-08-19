use maud::{html, Markup};

use crate::{
    ops::forc_doc::html::{common::page, page_type::PageType},
    utils::cli_error::CliError,
};
use std::fs::File;

pub(crate) fn build_page(page_type: &PageType, main_sidebar: &Markup) -> Result<(), CliError> {
    let page = page(&page_type, main_sidebar);

    let file_name = format!("./{}.html", page_type.name());
    let _ = File::create(&file_name)?;
    std::fs::write(&file_name, page.into_string())?;

    Ok(())
}

pub(crate) fn build_type_sidebar(type_name: &str, page_types: Vec<&PageType>) -> Markup {
    html! {
        div class="block" {
            h3 { (type_name) }
            ul {
                @for page in &page_types {
                    li {
                        a href=(format!("{}.html", page.name())) {
                            (page.name())
                        }
                    }
                }

            }
        }
    }
}
