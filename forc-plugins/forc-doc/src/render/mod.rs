use crate::{
    doc::{
        module::{ModuleInfo, ModulePrefixes},
        Document, Documentation,
    },
    render::{
        constant::{ALL_DOC_FILENAME, INDEX_FILENAME},
        index::{AllDocIndex, ModuleIndex},
        link::*,
        title::*,
        util::format::docstring::DocStrings,
    },
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, helper::doctype, html, prelude::*};
use std::collections::BTreeMap;
use sway_core::{language::ty::TyProgramKind, transform::AttributesMap};
use sway_types::BaseIdent;

pub mod constant;
mod index;
pub mod item;
pub mod link;
mod sidebar;
mod title;
pub mod util;

pub(crate) trait Renderable {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>>;
}
impl Renderable for Document {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let header = self.item_header.render(render_plan.clone())?;
        let body = self.item_body.render(render_plan)?;
        Ok(box_html! {
            : header;
            : body;
        })
    }
}

/// A [Document] rendered to HTML.
#[derive(Debug)]
pub(crate) struct RenderedDocument {
    pub(crate) module_info: ModuleInfo,
    pub(crate) html_filename: String,
    pub(crate) file_contents: HTMLString,
}
impl RenderedDocument {
    fn from_doc(doc: &Document, render_plan: RenderPlan) -> Result<Self> {
        Ok(Self {
            module_info: doc.module_info.clone(),
            html_filename: doc.html_filename(),
            file_contents: HTMLString::from_rendered_content(doc.clone().render(render_plan)?)?,
        })
    }
}

#[derive(Default)]
pub(crate) struct RenderedDocumentation(pub(crate) Vec<RenderedDocument>);

impl RenderedDocumentation {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from_raw_docs(
        raw_docs: Documentation,
        render_plan: RenderPlan,
        root_attributes: Option<AttributesMap>,
        program_kind: TyProgramKind,
        forc_version: Option<String>,
    ) -> Result<RenderedDocumentation> {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let root_module = match raw_docs.0.first() {
            Some(doc) => ModuleInfo::from_ty_module(
                vec![doc.module_info.project_name().to_owned()],
                root_attributes.map(|attrs_map| attrs_map.to_html_string()),
            ),
            None => panic!("Project does not contain a root module"),
        };

        let mut all_docs = DocLinks {
            style: DocStyle::AllDoc(program_kind.as_title_str().to_string()),
            links: Default::default(),
        };
        let mut module_map: BTreeMap<ModulePrefixes, BTreeMap<BlockTitle, Vec<DocLink>>> =
            BTreeMap::new();
        for doc in raw_docs.0 {
            rendered_docs
                .0
                .push(RenderedDocument::from_doc(&doc, render_plan.clone())?);

            // Here we gather all of the `doc_links` based on which module they belong to.
            populate_decls(&doc, &mut module_map);
            // Create links to child modules.
            populate_modules(&doc, &mut module_map);
            // Above we check for the module a link belongs to, here we want _all_ links so the check is much more shallow.
            populate_all_doc(&doc, &mut all_docs);
        }

        // ProjectIndex
        match module_map.get(&root_module.module_prefixes) {
            Some(doc_links) => rendered_docs.0.push(RenderedDocument {
                module_info: root_module.clone(),
                html_filename: INDEX_FILENAME.to_string(),
                file_contents: HTMLString::from_rendered_content(
                    ModuleIndex::new(
                        forc_version,
                        root_module.clone(),
                        DocLinks {
                            style: DocStyle::ProjectIndex(program_kind.as_title_str().to_string()),
                            links: doc_links.to_owned(),
                        },
                    )
                    .render(render_plan.clone())?,
                )?,
            }),
            None => panic!("Project does not contain a root module."),
        }
        if module_map.len() > 1 {
            module_map.remove_entry(&root_module.module_prefixes);

            // ModuleIndex(s)
            for (module_prefixes, doc_links) in module_map {
                let module_info_opt = match doc_links.values().last() {
                    Some(doc_links) => doc_links
                        .first()
                        .map(|doc_link| doc_link.module_info.clone()),
                    // No module to be documented
                    None => None,
                };
                if let Some(module_info) = module_info_opt {
                    rendered_docs.0.push(RenderedDocument {
                        module_info: module_info.clone(),
                        html_filename: INDEX_FILENAME.to_string(),
                        file_contents: HTMLString::from_rendered_content(
                            ModuleIndex::new(
                                None,
                                module_info.clone(),
                                DocLinks {
                                    style: DocStyle::ModuleIndex,
                                    links: doc_links.to_owned(),
                                },
                            )
                            .render(render_plan.clone())?,
                        )?,
                    });
                    if module_info.module_prefixes != module_prefixes {
                        let module_info = ModuleInfo::from_ty_module(module_prefixes, None);
                        rendered_docs.0.push(RenderedDocument {
                            module_info: module_info.clone(),
                            html_filename: INDEX_FILENAME.to_string(),
                            file_contents: HTMLString::from_rendered_content(
                                ModuleIndex::new(
                                    None,
                                    module_info,
                                    DocLinks {
                                        style: DocStyle::ModuleIndex,
                                        links: doc_links.to_owned(),
                                    },
                                )
                                .render(render_plan.clone())?,
                            )?,
                        })
                    }
                }
            }
        }
        // AllDocIndex
        rendered_docs.0.push(RenderedDocument {
            module_info: root_module.clone(),
            html_filename: ALL_DOC_FILENAME.to_string(),
            file_contents: HTMLString::from_rendered_content(
                AllDocIndex::new(root_module, all_docs).render(render_plan)?,
            )?,
        });

        Ok(rendered_docs)
    }
}
fn populate_doc_links(doc: &Document, doc_links: &mut BTreeMap<BlockTitle, Vec<DocLink>>) {
    let key = doc.item_body.ty_decl.as_block_title();
    match doc_links.get_mut(&key) {
        Some(links) => links.push(doc.link()),
        None => {
            doc_links.insert(key, vec![doc.link()]);
        }
    }
}
fn populate_decls(
    doc: &Document,
    module_map: &mut BTreeMap<ModulePrefixes, BTreeMap<BlockTitle, Vec<DocLink>>>,
) {
    let module_prefixes = &doc.module_info.module_prefixes;
    match module_map.get_mut(module_prefixes) {
        Some(doc_links) => populate_doc_links(doc, doc_links),
        None => {
            let mut doc_links = BTreeMap::<BlockTitle, Vec<DocLink>>::new();
            populate_doc_links(doc, &mut doc_links);
            module_map.insert(module_prefixes.clone(), doc_links);
        }
    }
}
fn populate_modules(
    doc: &Document,
    module_map: &mut BTreeMap<ModulePrefixes, BTreeMap<BlockTitle, Vec<DocLink>>>,
) {
    let mut module_clone = doc.module_info.clone();
    while module_clone.parent().is_some() {
        let html_filename = if module_clone.depth() > 2 {
            format!("{}/{INDEX_FILENAME}", module_clone.location())
        } else {
            INDEX_FILENAME.to_string()
        };
        let module_link = DocLink {
            name: module_clone.location().to_owned(),
            module_info: module_clone.to_owned(),
            html_filename,
            preview_opt: doc.module_info.preview_opt(),
        };
        let module_prefixes = module_clone
            .module_prefixes
            .clone()
            .split_last()
            .unwrap()
            .1
            .to_vec();
        match module_map.get_mut(&module_prefixes) {
            Some(doc_links) => match doc_links.get_mut(&BlockTitle::Modules) {
                Some(links) => {
                    if !links.contains(&module_link) {
                        links.push(module_link)
                    }
                }
                None => {
                    doc_links.insert(BlockTitle::Modules, vec![module_link]);
                }
            },
            None => {
                let mut doc_links: BTreeMap<BlockTitle, Vec<DocLink>> = BTreeMap::new();
                doc_links.insert(BlockTitle::Modules, vec![module_link]);
                module_map.insert(module_prefixes.clone(), doc_links);
            }
        }
        module_clone.module_prefixes.pop();
    }
}
fn populate_all_doc(doc: &Document, all_docs: &mut DocLinks) {
    populate_doc_links(doc, &mut all_docs.links);
}

/// The finalized HTML file contents.
#[derive(Debug)]
pub(crate) struct HTMLString(pub(crate) String);
impl HTMLString {
    /// Final rendering of a [Document] HTML page to String.
    fn from_rendered_content(rendered_content: Box<dyn RenderBox>) -> Result<Self> {
        Ok(Self(
            html! {
                : doctype::HTML;
                html {
                    : rendered_content
                }
            }
            .into_string()?,
        ))
    }
}

/// The type of document. Helpful in detemining what to represent in
/// the sidebar & page content.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum DocStyle {
    AllDoc(String),
    ProjectIndex(String),
    ModuleIndex,
    Item {
        title: Option<BlockTitle>,
        name: Option<BaseIdent>,
    },
}
