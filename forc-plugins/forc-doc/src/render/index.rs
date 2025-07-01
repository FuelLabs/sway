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
use std::collections::BTreeMap;

/// Workspace level index page
#[derive(Clone)]
pub(crate) struct WorkspaceIndex {
    /// The workspace root module info
    workspace_info: ModuleInfo,
    /// All documented libraries in the workspace
    documented_libraries: Vec<String>,
}
impl WorkspaceIndex {
    pub(crate) fn new(workspace_info: ModuleInfo, documented_libraries: Vec<String>) -> Self {
        Self {
            workspace_info,
            documented_libraries,
        }
    }
}
impl SidebarNav for WorkspaceIndex {
    fn sidebar(&self) -> Sidebar {
        // Create empty doc links for workspace sidebar (like a simple page)
        let doc_links = DocLinks {
            style: DocStyle::WorkspaceIndex,
            links: BTreeMap::new(),
        };
        
        Sidebar::new(
            None,
            DocStyle::WorkspaceIndex,
            self.workspace_info.clone(),
            doc_links,
        )
    }
}
impl Renderable for WorkspaceIndex {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let sidebar = self.sidebar().render(render_plan)?;
        
        // For workspace index, we're at the root level, so no path prefix needed
        let assets_path = format!("{ASSETS_DIR_NAME}");
        
        // Create a custom searchbar for workspace (at root level)
        let workspace_searchbar = box_html! {
            script(src="search.js", type="text/javascript");
            script {
                : Raw(r#"
                function onSearchFormSubmit(event) {
                    event.preventDefault();
                    const searchQuery = document.getElementById("search-input").value;
                    const url = new URL(window.location.href);
                    if (searchQuery) {
                        url.searchParams.set('search', searchQuery);
                    } else {
                        url.searchParams.delete('search');
                    }
                    history.pushState({ search: searchQuery }, "", url);
                    window.dispatchEvent(new HashChangeEvent("hashchange"));
                }
                
                document.addEventListener('DOMContentLoaded', () => {
                    const searchbar = document.getElementById("search-input");
                    searchbar.addEventListener("keyup", function(event) {
                        onSearchFormSubmit(event);
                    });
                    searchbar.addEventListener("search", function(event) {
                        onSearchFormSubmit(event);
                    });
                
                    function onQueryParamsChange() {
                        const searchParams = new URLSearchParams(window.location.search);
                        const query = searchParams.get("search");
                        const searchSection = document.getElementById('search');
                        const mainSection = document.getElementById('main-content');
                        const searchInput = document.getElementById('search-input');
                        if (query) {
                            searchInput.value = query;
                            const results = Object.values(SEARCH_INDEX).flat().filter(item => {
                                const lowerQuery = query.toLowerCase();
                                return item.name.toLowerCase().includes(lowerQuery);
                            });
                            const header = `<h1>Results for ${query}</h1>`;
                            if (results.length > 0) {
                                const resultList = results.map(item => {
                                    const formattedName = `<span class="type ${item.type_name}">${item.name}</span>`;
                                    const name = item.type_name === "module"
                                        ? [...item.module_info.slice(0, -1), formattedName].join("::")
                                        : [...item.module_info, formattedName].join("::");
                                    // Fix path generation for workspace - no leading slash, proper relative path
                                    const path = [...item.module_info, item.html_filename].join("/");
                                    const left = `<td><span>${name}</span></td>`;
                                    const right = `<td><p>${item.preview}</p></td>`;
                                    return `<tr onclick="window.location='${path}';">${left}${right}</tr>`;
                                }).join('');
                                searchSection.innerHTML = `${header}<table>${resultList}</table>`;
                            } else {
                                searchSection.innerHTML = `${header}<p>No results found.</p>`;
                            }
                            searchSection.setAttribute("class", "search-results");
                            mainSection.setAttribute("class", "content hidden");
                        } else {
                            searchSection.setAttribute("class", "search-results hidden");
                            mainSection.setAttribute("class", "content");
                        }
                    }
                    window.addEventListener('hashchange', onQueryParamsChange);
                    onQueryParamsChange();
                });
                "#)
            }
            nav(class="sub") {
                form(id="search-form", class="search-form", onsubmit="onSearchFormSubmit(event)") {
                    div(class="search-container") {
                        input(
                            id="search-input",
                            class="search-input",
                            name="search",
                            autocomplete="off",
                            spellcheck="false",
                            placeholder="Search the docs...",
                            type="search"
                        );
                    }
                }
            }
        };
        
        Ok(box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content="Workspace documentation index"
                );
                meta(name="keywords", content="sway, swaylang, sway-lang, workspace");
                link(rel="icon", href=format!("{}/sway-logo.svg", assets_path));
                title: "Workspace Documentation";
                link(rel="stylesheet", type="text/css", href=format!("{}/normalize.css", assets_path));
                link(rel="stylesheet", type="text/css", href=format!("{}/swaydoc.css", assets_path), id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=format!("{}/ayu.css", assets_path));
                link(rel="stylesheet", href=format!("{}/ayu.min.css", assets_path));
            }
            body(class="swaydoc mod") {
                : sidebar;
                main {
                    div(class="width-limiter") {
                        : *workspace_searchbar;
                        section(id="main-content", class="content") {
                            div(class="main-heading") {
                                p { : "This workspace contains the following libraries:" }
                            }
                            h2(class="small-section-header") {
                                : "Libraries";
                            }
                            div(class="item-table") {
                                @ for lib in &self.documented_libraries {
                                    div(class="item-row") {
                                        div(class="item-left module-item") {
                                            a(class="mod", href=format!("{}/index.html", lib)) {
                                                : lib;
                                            }
                                        }
                                        div(class="item-right docblock-short") {
                                            : format!("Library {}", lib);
                                        }
                                    }
                                }
                            }
                        }
                        section(id="search", class="search-results");
                    }
                }
                script(src=format!("{}/highlight.js", assets_path));
                script {
                    : "hljs.highlightAll();";
                }
            }
        })
    }
}

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
