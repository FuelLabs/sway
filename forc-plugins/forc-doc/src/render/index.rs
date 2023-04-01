use crate::{
    doc::ModuleInfo,
    render::{constant::IDENTITY, link::DocLinks, sidebar::*, BlockTitle, DocStyle, Renderable},
    RenderPlan,
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
    fn new(project_name: ModuleInfo, all_docs: DocLinks) -> Self {
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
                link(rel="icon", href="assets/sway-logo.svg");
                title: "List of all items in this project";
                link(rel="stylesheet", type="text/css", href="assets/normalize.css");
                link(rel="stylesheet", type="text/css", href="assets/swaydoc.css", id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href="assets/ayu.css");
                link(rel="stylesheet", href="assets/ayu.min.css");
            }
            body(class="swaydoc mod") {
                : sidebar;
                main {
                    div(class="width-limiter") {
                        // div(class="sub-container") {
                        //     nav(class="sub") {
                        //         form(class="search-form") {
                        //             div(class="search-container") {
                        //                 span;
                        //                 input(
                        //                     class="search-input",
                        //                     name="search",
                        //                     autocomplete="off",
                        //                     spellcheck="false",
                        //                     // TODO: Add functionality.
                        //                     placeholder="Searchbar unimplemented, see issue #3480...",
                        //                     type="search"
                        //                 );
                        //                 div(id="help-button", title="help", tabindex="-1") {
                        //                     button(type="button") { : "?" }
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                        section(id="main-content", class="content") {
                            h1(class="fqn") {
                                span(class="in-band") { : "List of all items" }
                            }
                            : doc_links;
                        }
                    }
                }
                script(src="assets/highlight.js");
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
impl SidebarNav for ModuleIndex {
    fn sidebar(&self) -> Sidebar {
        let style = match self.module_info.is_root_module() {
            true => self.module_docs.style.clone(),
            false => DocStyle::ModuleIndex,
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
            .to_html_shorthand_path_string("assets/sway-logo.svg");
        let normalize = self
            .module_info
            .to_html_shorthand_path_string("assets/normalize.css");
        let swaydoc = self
            .module_info
            .to_html_shorthand_path_string("assets/swaydoc.css");
        let ayu = self
            .module_info
            .to_html_shorthand_path_string("assets/ayu.css");
        let sway_hjs = self
            .module_info
            .to_html_shorthand_path_string("assets/highlight.js");
        let ayu_hjs = self
            .module_info
            .to_html_shorthand_path_string("assets/ayu.min.css");
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
                        // div(class="sub-container") {
                        //     nav(class="sub") {
                        //         form(class="search-form") {
                        //             div(class="search-container") {
                        //                 span;
                        //                 input(
                        //                     class="search-input",
                        //                     name="search",
                        //                     autocomplete="off",
                        //                     spellcheck="false",
                        //                     // TODO: Add functionality.
                        //                     placeholder="Searchbar unimplemented, see issue #3480...",
                        //                     type="search"
                        //                 );
                        //                 div(id="help-button", title="help", tabindex="-1") {
                        //                     button(type="button") { : "?" }
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
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
