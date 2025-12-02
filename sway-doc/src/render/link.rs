//! Handles creation of links for modules.
use crate::{
    doc::module::ModuleInfo,
    render::{BlockTitle, DocStyle, Renderable},
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox, Template};
use std::collections::BTreeMap;

/// Used for creating links between docs.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DocLink {
    pub name: String,
    pub module_info: ModuleInfo,
    pub html_filename: String,
    pub preview_opt: Option<String>,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DocLinks {
    pub style: DocStyle,
    /// The title and link info for each doc item.
    pub links: BTreeMap<BlockTitle, Vec<DocLink>>,
}

impl Renderable for DocLinks {
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let mut links_vec = Vec::new();
        // sort the doc links alphabetically
        // for the AllDoc page, sort based on the module prefix and name
        match self.style {
            DocStyle::AllDoc(_) => {
                for (block_title, mut doc_link) in self.links {
                    doc_link.sort_by(|a, b| {
                        let first = a
                            .module_info
                            .to_path_literal_string(&a.name, a.module_info.project_name());
                        let second = b
                            .module_info
                            .to_path_literal_string(&b.name, b.module_info.project_name());
                        first.cmp(&second)
                    });
                    links_vec.push((block_title, doc_link));
                }
            }
            _ => {
                for (block_title, mut doc_link) in self.links {
                    doc_link.sort();
                    links_vec.push((block_title.clone(), doc_link.to_vec()));
                }
            }
        }
        let doc_links = match self.style {
            DocStyle::AllDoc(_) => box_html! {
                @ for (title, list_items) in links_vec {
                    @ if !list_items.is_empty() {
                        h2(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(
                                            class=title.class_title_str(),
                                            href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.project_name())
                                        ) {
                                            : item.module_info.to_path_literal_string(
                                                &item.name,
                                                item.module_info.project_name()
                                            );
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.clone().unwrap());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .into_string()
            .unwrap(),
            DocStyle::ProjectIndex(_) => box_html! {
                @ for (title, list_items) in links_vec {
                    @ if !list_items.is_empty() {
                        h2(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(
                                            class=title.class_title_str(),
                                            href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.project_name())
                                        ) {
                                            @ if title == BlockTitle::Modules {
                                                : item.name.clone();
                                            } else {
                                                : item.module_info.to_path_literal_string(
                                                    &item.name,
                                                    item.module_info.project_name()
                                                );
                                            }
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.clone().unwrap());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .into_string()
            .unwrap(),
            _ => box_html! {
                @ for (title, list_items) in links_vec {
                    @ if !list_items.is_empty() {
                        h2(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(
                                            class=title.class_title_str(),
                                            href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.location())
                                        ) {
                                            : item.module_info.to_path_literal_string(
                                                &item.name,
                                                item.module_info.location()
                                            );
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.clone().unwrap());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .into_string()
            .unwrap(),
        };
        Ok(box_html! {
            : Raw(doc_links);
        })
    }
}
