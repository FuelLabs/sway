use crate::ASSETS_DIR_NAME;
use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    doc::module::ModuleInfo,
    render::{
        constant::{ALL_DOC_FILENAME, IDENTITY, INDEX_FILENAME},
        BlockTitle, DocLinks, DocStyle, Renderable,
    },
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox, Template};

pub(crate) trait SidebarNav {
    /// Create sidebar component.
    fn sidebar(&self) -> Sidebar;
}

/// Sidebar component for quick navigation.
pub(crate) struct Sidebar {
    version_opt: Option<String>,
    style: DocStyle,
    module_info: ModuleInfo,
    /// support for page navigation
    nav: DocLinks,
}
impl Sidebar {
    pub(crate) fn new(
        version_opt: Option<String>,
        style: DocStyle,
        module_info: ModuleInfo,
        nav: DocLinks,
    ) -> Self {
        Self {
            version_opt,
            style,
            module_info,
            nav,
        }
    }
}
impl Renderable for Sidebar {
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let path_to_logo = self
            .module_info
            .to_html_shorthand_path_string(&format!("{ASSETS_DIR_NAME}/sway-logo.svg"));
        let style = self.style.clone();
        let version_opt = self.version_opt.clone();
        let location_with_prefix = match &style {
            DocStyle::AllDoc(project_kind)
            | DocStyle::ProjectIndex {
                kind: project_kind, ..
            } => {
                format!("{project_kind} {}", self.module_info.location())
            }
            DocStyle::ModuleIndex => format!(
                "{} {}",
                BlockTitle::Modules.item_title_str(),
                self.module_info.location()
            ),
            DocStyle::Item { title, name } => {
                let title = title.clone().expect("Expected a BlockTitle");
                let name = name.clone().expect("Expected a BaseIdent");
                format!("{} {}", title.item_title_str(), name.as_str())
            }
        };
        let root_path = self.module_info.to_html_shorthand_path_string(
            PathBuf::from(self.module_info.project_name())
                .join(INDEX_FILENAME)
                .to_str()
                .ok_or_else(|| anyhow::anyhow!(
                    "found invalid root file path for {}\nmake sure your project's name contains only valid unicode characters",
                    self.module_info.project_name(),
                ))?,
        );

        let logo_path_to_root = match style {
            DocStyle::AllDoc(_) | DocStyle::Item { .. } | DocStyle::ModuleIndex => root_path,
            DocStyle::ProjectIndex { .. } => IDENTITY.to_owned(),
        };

        // Unfortunately, match arms that return a closure, even if they are the same
        // type, are incompatible. The work around is to return a String instead,
        // and render it from Raw in the final output.
        let styled_content = match &self.style {
            DocStyle::ProjectIndex { members, .. } => {
                let nav_links = &self.nav.links;

                let mut members_map = BTreeMap::new();

                for member in members {
                    let root = self.module_info.to_html_shorthand_path_string(
                        PathBuf::from(member)
                            .join(INDEX_FILENAME)
                            .to_str()
                            .ok_or_else(|| {
                                anyhow::anyhow!("found invalid root file path for {}\n", member,)
                            })?,
                    );
                    members_map.insert(member, root);
                }

                box_html! {
                    div(class="sidebar-elems") {
                        a(id="all-types", href=ALL_DOC_FILENAME) {
                            p: "All Items";
                        }
                        section {
                            div(class="block") {
                                ul {
                                    @ for (title, _) in nav_links {
                                        li {
                                            a(href=format!("{}{}", IDENTITY, title.html_title_string())) {
                                                : title.as_str();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                       section(class = "sidebar-header") {
                            div(class="block method") {
                                h3 : "Crates";
                                ul {
                                    @ for (member, link) in members_map {
                                        li {
                                            a(href=link) {
                                                : member
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                .into_string()
                .unwrap()
            }
            DocStyle::AllDoc(_) => {
                let nav_links = &self.nav.links;
                box_html! {
                    div(class="sidebar-elems") {
                        a(id="all-types", href=INDEX_FILENAME) {
                            p: "Back to index";
                        }
                         section(class = "sidebar-header") {
                            div(class="block method") {
                                ul {
                                    @ for (title, _) in nav_links {
                                        li {
                                            a(href=format!("{}{}", IDENTITY, title.html_title_string())) {
                                                : title.as_str();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                .into_string()
                .unwrap()
            }
            _ => box_html! {
                div(class="sidebar-elems") {
                    @ for (title, doc_links) in &self.nav.links {
                        section(class = "sidebar-header") {
                            h3 {
                                a(href=format!("{}{}", IDENTITY, title.html_title_string())) {
                                    : title.as_str();
                                }
                            }
                            ul(class="block method") {
                                @ for doc_link in doc_links {
                                    li {
                                        a(href=format!("{}", doc_link.html_filename)) {
                                            : doc_link.name.clone();
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
            nav(class="sidebar") {
                a(class="sidebar-logo", href=&logo_path_to_root) {
                    div(class="logo-container") {
                        img(class="sway-logo", src=path_to_logo, alt="logo");
                    }
                }
                h2(class="location") {
                    : location_with_prefix;
                }
                @ if let DocStyle::ProjectIndex{..} = style.clone() {
                    @ if version_opt.is_some() {
                        div(class="version") {
                            p: version_opt.unwrap();
                        }
                    }
                }
                : Raw(styled_content);
            }
        })
    }
}
