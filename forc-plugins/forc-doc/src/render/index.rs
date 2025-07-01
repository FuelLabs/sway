//! Handles creation of `index.html` files.
use crate::{
    doc::module::ModuleInfo,
    render::{
        link::DocLinks, search::generate_searchbar, sidebar::*, BlockTitle, DocStyle, Renderable,
        IDENTITY,
    },
    RenderPlan, ASSETS_DIR_NAME,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox};

/// Project level, all items belonging to a project
#[derive(Clone)]
pub(crate) struct AllDocIndex {
    /// A [ModuleInfo] with only the project name.
    project_name: ModuleInfo,
    /// All doc items.
    all_docs: DocLinks,
}
impl AllDocIndex {
    pub(crate) fn new(project_name: ModuleInfo, all_docs: DocLinks) -> Self {
        Self {
            project_name,
            all_docs,
        }
    }
}
impl SidebarNav for AllDocIndex {
    fn sidebar(&self) -> Sidebar {
        Sidebar::new(
            None,
            self.all_docs.style.clone(),
            self.project_name.clone(),
            self.all_docs.clone(),
        )
    }
}
impl Renderable for AllDocIndex {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let doc_links = self.all_docs.clone().render(render_plan.clone())?;
        let sidebar = self.sidebar().render(render_plan)?;
        Ok(box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content="List of all items in this project"
                );
                meta(name="keywords", content="sway, swaylang, sway-lang");
                link(rel="icon", href=format!("../{ASSETS_DIR_NAME}/sway-logo.svg"));
                title: "List of all items in this project";
                link(rel="stylesheet", type="text/css", href=format!("../{ASSETS_DIR_NAME}/normalize.css"));
                link(rel="stylesheet", type="text/css", href=format!("../{ASSETS_DIR_NAME}/swaydoc.css"), id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=format!("../{ASSETS_DIR_NAME}/ayu.css"));
                link(rel="stylesheet", href=format!("../{ASSETS_DIR_NAME}/ayu.min.css"));
            }
            body(class="swaydoc mod") {
                : sidebar;
                main {
                    div(class="width-limiter") {
                        : generate_searchbar(&self.project_name);
                        section(id="main-content", class="content") {
                            h1(class="fqn") {
                                span(class="in-band") { : "List of all items" }
                            }
                            : doc_links;
                        }
                        section(id="search", class="search-results");
                    }
                }
                script(src=format!("../{ASSETS_DIR_NAME}/highlight.js"));
                script {
                    : "hljs.highlightAll();";
                }
            }
        })
    }
}

/// The index for each module in a Sway project.
pub(crate) struct ModuleIndex {
    /// used only for the root module
    version_opt: Option<String>,
    module_info: ModuleInfo,
    module_docs: DocLinks,
}
impl ModuleIndex {
    pub(crate) fn new(
        version_opt: Option<String>,
        module_info: ModuleInfo,
        module_docs: DocLinks,
    ) -> Self {
        Self {
            version_opt,
            module_info,
            module_docs,
        }
    }
}
impl SidebarNav for ModuleIndex {
    fn sidebar(&self) -> Sidebar {
        let style = if self.module_info.is_root_module() {
            self.module_docs.style.clone()
        } else {
            DocStyle::ModuleIndex
        };
        Sidebar::new(
            self.version_opt.clone(),
            style,
            self.module_info.clone(),
            self.module_docs.clone(),
        )
    }
}
impl Renderable for ModuleIndex {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let doc_links = self.module_docs.clone().render(render_plan.clone())?;
        let sidebar = self.sidebar().render(render_plan)?;
        let title_prefix = match self.module_docs.style {
            DocStyle::ProjectIndex(ref program_type) => format!("{program_type} "),
            DocStyle::ModuleIndex => "Module ".to_string(),
            _ => unreachable!("Module Index can only be either a project or module at this time."),
        };

        let favicon = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/sway-logo.svg"));
        let normalize = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/normalize.css"));
        let swaydoc = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/swaydoc.css"));
        let ayu = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/ayu.css"));
        let sway_hjs = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/highlight.js"));
        let ayu_hjs = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/ayu.min.css"));
        let mut rendered_module_anchors = self.module_info.get_anchors()?;
        rendered_module_anchors.pop();

        Ok(box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content=format!(
                        "API documentation for the Sway `{}` module in `{}`.",
                        self.module_info.location(), self.module_info.project_name(),
                    )
                );
                meta(name="keywords", content=format!("sway, swaylang, sway-lang, {}", self.module_info.location()));
                link(rel="icon", href=favicon);
                title: format!("{} in {} - Sway", self.module_info.location(), self.module_info.project_name());
                link(rel="stylesheet", type="text/css", href=normalize);
                link(rel="stylesheet", type="text/css", href=swaydoc, id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=ayu);
                link(rel="stylesheet", href=ayu_hjs);
            }
            body(class="swaydoc mod") {
                : sidebar;
                main {
                    div(class="width-limiter") {
                        : generate_searchbar(&self.module_info);
                        section(id="main-content", class="content") {
                            div(class="main-heading") {
                                h1(class="fqn") {
                                    span(class="in-band") {
                                        : title_prefix;
                                        @ for anchor in rendered_module_anchors {
                                            : Raw(anchor);
                                        }
                                        a(class=BlockTitle::Modules.class_title_str(), href=IDENTITY) {
                                            : self.module_info.location();
                                        }
                                    }
                                }
                            }
                            @ if self.module_info.attributes.is_some() {
                                details(class="swaydoc-toggle top-doc", open) {
                                    summary(class="hideme") {
                                        span { : "Expand description" }
                                    }
                                    div(class="docblock") {
                                        : Raw(self.module_info.attributes.unwrap())
                                    }
                                }
                            }
                            : doc_links;
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
