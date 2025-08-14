//! Handles creation of the head and body of an HTML doc.
use crate::{
    doc::module::ModuleInfo,
    render::{
        item::context::ItemContext,
        search::generate_searchbar,
        sidebar::{Sidebar, SidebarNav},
        DocStyle, Renderable, IDENTITY,
    },
    RenderPlan, ASSETS_DIR_NAME,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox};

use sway_types::BaseIdent;

use super::documentable_type::DocumentableType;

// Asset file names to avoid repeated string formatting
const SWAY_LOGO_FILE: &str = "sway-logo.svg";
const NORMALIZE_CSS_FILE: &str = "normalize.css";
const SWAYDOC_CSS_FILE: &str = "swaydoc.css";
const AYU_CSS_FILE: &str = "ayu.css";
const AYU_MIN_CSS_FILE: &str = "ayu.min.css";

/// All necessary components to render the header portion of
/// the item html doc.
#[derive(Clone, Debug)]
pub struct ItemHeader {
    pub module_info: ModuleInfo,
    pub friendly_name: &'static str,
    pub item_name: BaseIdent,
}
impl Renderable for ItemHeader {
    /// Basic HTML header component
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let ItemHeader {
            module_info,
            friendly_name,
            item_name,
        } = self;

        let favicon = module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/{SWAY_LOGO_FILE}"));
        let normalize = module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/{NORMALIZE_CSS_FILE}"));
        let swaydoc = module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/{SWAYDOC_CSS_FILE}"));
        let ayu =
            module_info.to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/{AYU_CSS_FILE}"));
        let ayu_hjs = module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/{AYU_MIN_CSS_FILE}"));

        Ok(box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content=format!(
                        "API documentation for the Sway `{}` {} in `{}`.",
                        item_name.as_str(), friendly_name, module_info.location(),
                    )
                );
                meta(name="keywords", content=format!("sway, swaylang, sway-lang, {}", item_name.as_str()));
                link(rel="icon", href=favicon);
                title: format!("{} in {} - Sway", item_name.as_str(), module_info.location());
                link(rel="stylesheet", type="text/css", href=normalize);
                link(rel="stylesheet", type="text/css", href=swaydoc, id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=ayu);
                link(rel="stylesheet", href=ayu_hjs);
                // TODO: Add links for fonts
            }
        })
    }
}

/// All necessary components to render the body portion of
/// the item html doc. Many parts of the HTML body structure will be the same
/// for each item, but things like struct fields vs trait methods will be different.
#[derive(Clone, Debug)]
pub struct ItemBody {
    pub module_info: ModuleInfo,
    pub ty: DocumentableType,
    /// The item name varies depending on type.
    /// We store it during info gathering to avoid
    /// multiple match statements.
    pub item_name: BaseIdent,
    pub code_str: String,
    pub attrs_opt: Option<String>,
    pub item_context: ItemContext,
}
impl SidebarNav for ItemBody {
    fn sidebar(&self) -> Sidebar {
        let style = DocStyle::Item {
            title: Some(self.ty.as_block_title()),
            name: Some(self.item_name.clone()),
        };
        Sidebar::new(
            None,
            style,
            self.module_info.clone(),
            self.item_context.to_doclinks(),
        )
    }
}
impl Renderable for ItemBody {
    /// HTML body component
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let sidebar = self.sidebar();
        let ItemBody {
            module_info,
            ty,
            item_name,
            code_str,
            attrs_opt,
            item_context,
        } = self;

        let doc_name = ty.doc_name().to_string();
        let block_title = ty.as_block_title();
        let sidebar = sidebar.render(render_plan.clone())?;
        let item_context = (item_context.context_opt.is_some()
            || item_context.impl_traits.is_some())
        .then(|| -> Result<Box<dyn RenderBox>> { item_context.render(render_plan.clone()) });
        let sway_hjs =
            module_info.to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/highlight.js"));
        let rendered_module_anchors = module_info.get_anchors()?;

        Ok(box_html! {
            body(class=format!("swaydoc {doc_name}")) {
                : sidebar;
                // this is the main code block
                main {
                    div(class="width-limiter") {
                        : generate_searchbar(&module_info);
                        section(id="main-content", class="content") {
                            div(class="main-heading") {
                                h1(class="fqn") {
                                    span(class="in-band") {
                                        : format!("{} ", block_title.item_title_str());
                                        @ for anchor in rendered_module_anchors {
                                            : Raw(anchor);
                                        }
                                        a(class=&doc_name, href=IDENTITY) {
                                            : item_name.as_str();
                                        }
                                    }
                                }
                            }
                            div(class="docblock item-decl") {
                                pre(class=format!("sway {}", &doc_name)) {
                                    code { : code_str; }
                                }
                            }
                            @ if attrs_opt.is_some() {
                                // expand or hide description of main code block
                                details(class="swaydoc-toggle top-doc", open) {
                                    summary(class="hideme") {
                                        span { : "Expand description" }
                                    }
                                    // this is the description
                                    div(class="docblock") {
                                        : Raw(attrs_opt.unwrap())
                                    }
                                }
                            }
                            @ if item_context.is_some() {
                                : item_context.unwrap();
                            }
                        }
                        section(id="search", class="search-results");
                    }
                }
                script(src=sway_hjs);
                script {
                    : "hljs.highlightAll();";
                }
            }
        })
    }
}
