use std::{fmt::Write, path::PathBuf};

use crate::doc::Documentation;
use comrak::{markdown_to_html, ComrakOptions};
use horrorshow::{box_html, helper::doctype, html, prelude::*, Raw};
use sway_core::language::ty::{TyDeclaration, TyStructField};
use sway_core::transform::{AttributeKind, AttributesMap};
use sway_lsp::utils::markdown::format_docs;

pub(crate) const ALL_DOC_FILENAME: &str = "all.html";
pub(crate) trait Renderable {
    fn render(self) -> Box<dyn RenderBox>;
}
/// A [Document] rendered to HTML.
pub(crate) struct RenderedDocument {
    pub(crate) module_prefix: Vec<String>,
    pub(crate) file_name: String,
    pub(crate) file_contents: HTMLString,
}
#[derive(Default)]
pub(crate) struct RenderedDocumentation(pub(crate) Vec<RenderedDocument>);

impl RenderedDocumentation {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from(raw: Documentation) -> RenderedDocumentation {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let mut all_doc: AllDoc = Default::default();
        for doc in raw {
            let module_prefix = doc.module_prefix.clone();
            let file_name = doc.file_name();
            rendered_docs.0.push(RenderedDocument {
                module_prefix: module_prefix.clone(),
                file_name: file_name.clone(),
                file_contents: HTMLString::from(doc.clone().render()),
            });

            let item_name = doc.item_header.item_name.as_str().to_string();
            // need to think about how to do this for larger paths
            let path_str = if doc.item_header.module_depth == 0 {
                item_name
            } else {
                format!("{}::{}", &doc.item_header.module, &item_name)
            };
            all_doc.0.push(AllDocItem {
                ty_decl: doc.item_body.ty_decl.clone(),
                path_str,
                module_prefix,
                file_name,
            });
        }
        // All Doc
        rendered_docs.0.push(RenderedDocument {
            module_prefix: vec![],
            file_name: ALL_DOC_FILENAME.to_string(),
            file_contents: HTMLString::from(all_doc.render()),
        });
        rendered_docs
    }
}
/// The finalized HTML file contents.
pub(crate) struct HTMLString(pub(crate) String);
impl HTMLString {
    /// Final rendering of a [Document] HTML page to String.
    fn from(rendered_content: Box<dyn RenderBox>) -> Self {
        let markup = html! {
            : doctype::HTML;
            html {
                : rendered_content
            }
        };

        Self(markup.into_string().unwrap())
    }
}

/// All necessary components to render the header portion of
/// the item html doc.
#[derive(Clone)]
pub(crate) struct ItemHeader {
    pub(crate) module_depth: usize,
    pub(crate) module: String,
    pub(crate) friendly_name: String,
    pub(crate) item_name: String,
}
impl Renderable for ItemHeader {
    /// Basic HTML header component
    fn render(self) -> Box<dyn RenderBox> {
        let ItemHeader {
            module_depth,
            module: location,
            friendly_name,
            item_name,
        } = self;

        let prefix = module_depth_to_path_prefix(module_depth);
        let mut favicon = prefix.clone();
        let mut normalize = prefix.clone();
        let mut swaydoc = prefix.clone();
        let mut ayu = prefix;
        favicon.push_str("assets/sway-logo.svg");
        normalize.push_str("assets/normalize.css");
        swaydoc.push_str("assets/swaydoc.css");
        ayu.push_str("assets/ayu.css");

        box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content=format!(
                        "API documentation for the Sway `{}` {} in `{}`.",
                        item_name.clone(), friendly_name, location,
                    )
                );
                meta(name="keywords", content=format!("sway, swaylang, sway-lang, {}", item_name));
                link(rel="icon", href=favicon);
                title: format!("{} in {} - Sway", item_name, location);
                link(rel="stylesheet", type="text/css", href=normalize);
                link(rel="stylesheet", type="text/css", href=swaydoc, id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href=ayu);
                // TODO: Add links for fonts
            }
        }
    }
}
/// All necessary components to render the body portion of
/// the item html doc. Many parts of the HTML body structure will be the same
/// for each item, but things like struct fields vs trait methods will be different.
#[derive(Clone)]
pub(crate) struct ItemBody {
    pub(crate) module_depth: usize,
    pub(crate) ty_decl: TyDeclaration,
    /// The item name varies depending on type.
    /// We store it during info gathering to avoid
    /// multiple match statements.
    pub(crate) item_name: String,
    pub(crate) code_str: String,
    pub(crate) attrs_opt: Option<String>,
}

impl Renderable for ItemBody {
    /// HTML body component
    fn render(self) -> Box<dyn RenderBox> {
        let ItemBody {
            module_depth,
            ty_decl,
            item_name: decl_name,
            code_str,
            attrs_opt,
        } = self;

        let decl_ty = ty_decl.doc_name();
        let friendly_name = ty_decl.friendly_name();
        let mut all_path = module_depth_to_path_prefix(module_depth);
        all_path.push_str(ALL_DOC_FILENAME);

        box_html! {
            body(class=format!("swaydoc {decl_ty}")) {
                : sidebar(module_depth, decl_name.clone(), all_path);
                // this is the main code block
                main {
                    div(class="width-limiter") {
                        div(class="sub-container") {
                            nav(class="sub") {
                                form(class="search-form") {
                                    div(class="search-container") {
                                        span;
                                        input(
                                            class="search-input",
                                            name="search",
                                            autocomplete="off",
                                            spellcheck="false",
                                            // TODO: https://github.com/FuelLabs/sway/issues/3480
                                            placeholder="Searchbar unimplemented, see issue #3480...",
                                            type="search"
                                        );
                                        div(id="help-button", title="help", tabindex="-1") {
                                            button(type="button") { : "?" }
                                        }
                                    }
                                }
                            }
                        }
                        section(id="main-content", class="content") {
                            div(class="main-heading") {
                                h1(class="fqn") {
                                    span(class="in-band") {
                                        // TODO: pass the decl ty info or match
                                        // for uppercase naming like: "Enum"
                                        : format!("{} ", friendly_name);
                                        // TODO: add qualified path anchors
                                        a(class=&decl_ty, href="#") {
                                            : decl_name;
                                        }
                                    }
                                }
                            }
                            div(class="docblock item-decl") {
                                pre(class=format!("sway {}", &decl_ty)) {
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
                            // : item_context.0;
                        }
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
struct AllDocItem {
    ty_decl: TyDeclaration,
    path_str: String,
    module_prefix: Vec<String>,
    file_name: String,
}
struct ItemPath {
    path_literal_str: String,
    qualified_file_path: String,
}
#[derive(Default, Clone)]
struct AllDoc(Vec<AllDocItem>);

impl Renderable for AllDoc {
    /// crate level, all items belonging to a crate
    fn render(self) -> Box<dyn RenderBox> {
        let AllDoc(all_doc) = self;
        // TODO: find a better way to do this
        //
        // we need to have a finalized list for the all doc
        let mut struct_items: Vec<ItemPath> = Vec::new();
        let mut enum_items: Vec<ItemPath> = Vec::new();
        let mut trait_items: Vec<ItemPath> = Vec::new();
        let mut abi_items: Vec<ItemPath> = Vec::new();
        let mut storage_items: Vec<ItemPath> = Vec::new();
        let mut fn_items: Vec<ItemPath> = Vec::new();
        let mut const_items: Vec<ItemPath> = Vec::new();

        for doc_item in all_doc.clone() {
            let AllDocItem {
                ty_decl,
                path_str,
                module_prefix,
                file_name,
            } = doc_item;
            use TyDeclaration::*;
            match ty_decl {
                StructDeclaration(_) => struct_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                EnumDeclaration(_) => enum_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                TraitDeclaration(_) => trait_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                AbiDeclaration(_) => abi_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                StorageDeclaration(_) => storage_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                FunctionDeclaration(_) => fn_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                ConstantDeclaration(_) => const_items.push(ItemPath {
                    path_literal_str: path_str.to_string(),
                    qualified_file_path: qualified_file_path(&module_prefix, file_name.to_string()),
                }),
                _ => {}
            }
        }
        let project_name = all_doc
            .first()
            .unwrap()
            .module_prefix
            .first()
            .unwrap()
            .clone();
        box_html! {
            head {
                meta(charset="utf-8");
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                meta(name="generator", content="swaydoc");
                meta(
                    name="description",
                    content="List of all items in this crate"
                );
                meta(name="keywords", content="sway, swaylang, sway-lang");
                link(rel="icon", href="assets/sway-logo.svg");
                title: "List of all items in this crate";
                link(rel="stylesheet", type="text/css", href="assets/normalize.css");
                link(rel="stylesheet", type="text/css", href="assets/swaydoc.css", id="mainThemeStyle");
                link(rel="stylesheet", type="text/css", href="assets/ayu.css");
            }
            body(class="swaydoc mod") {
                : sidebar(0, format!("Crate {project_name}"), ALL_DOC_FILENAME.to_string());
                main {
                    div(class="width-limiter") {
                        div(class="sub-container") {
                            nav(class="sub") {
                                form(class="search-form") {
                                    div(class="search-container") {
                                        span;
                                        input(
                                            class="search-input",
                                            name="search",
                                            autocomplete="off",
                                            spellcheck="false",
                                            // TODO: Add functionality.
                                            placeholder="Searchbar unimplemented, see issue #3480...",
                                            type="search"
                                        );
                                        div(id="help-button", title="help", tabindex="-1") {
                                            button(type="button") { : "?" }
                                        }
                                    }
                                }
                            }
                        }
                        section(id="main-content", class="content") {
                            h1(class="fqn") {
                                span(class="in-band") { : "List of all items" }
                            }
                            @ if !storage_items.is_empty() {
                                : all_items_list("Contract Storage".to_string(), storage_items);
                            }
                            @ if !abi_items.is_empty() {
                                : all_items_list("Abi".to_string(), abi_items);
                            }
                            @ if !trait_items.is_empty() {
                                : all_items_list("Traits".to_string(), trait_items);
                            }
                            @ if !struct_items.is_empty() {
                                : all_items_list("Structs".to_string(), struct_items);
                            }
                            @ if !enum_items.is_empty() {
                                : all_items_list("Enums".to_string(), enum_items);
                            }
                            @ if !fn_items.is_empty() {
                                : all_items_list("Functions".to_string(), fn_items);
                            }
                            @ if !const_items.is_empty() {
                                : all_items_list("Constants".to_string(), const_items);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Renders the items list from each item kind and adds the links to each file path
fn all_items_list(title: String, list_items: Vec<ItemPath>) -> Box<dyn RenderBox> {
    box_html! {
        h3(id=format!("{title}")) { : title.clone(); }
        ul(class=format!("{} docblock", title.to_lowercase())) {
            @ for item in list_items {
                li {
                    a(href=item.qualified_file_path) { : item.path_literal_str; }
                }
            }
        }
    }
}
// module level index.html
// for each module we need to create an index
// that will have all of the item docs in it
fn _module_index() -> Box<dyn RenderBox> {
    box_html! {}
}
/// Sidebar component
fn sidebar(
    module_depth: usize,
    location: String,
    href: String, /* TODO: sidebar_items */
) -> Box<dyn RenderBox> {
    let mut logo_path = module_depth_to_path_prefix(module_depth);
    logo_path.push_str("assets/sway-logo.svg");

    box_html! {
        nav(class="sidebar") {
            a(class="sidebar-logo", href=href) {
                div(class="logo-container") {
                    img(class="sway-logo", src=logo_path, alt="logo");
                }
            }
            h2(class="location") {
                a(href="#") { : location; }
            }
            div(class="sidebar-elems") {
                section {
                    // TODO: add connections between item contents and
                    // sidebar nav. This will be dynamic e.g. "Variants"
                    // for Enum, and "Fields" for Structs
                }
            }
        }
    }
}
/// Creates a String version of the path to an item, used in navigating from
/// all.html to items.
fn qualified_file_path(module_prefix: &Vec<String>, file_name: String) -> String {
    let mut file_path = PathBuf::new();
    for prefix in module_prefix {
        if prefix != &module_prefix[0] {
            file_path.push(prefix)
        }
    }
    file_path.push(file_name);

    file_path.to_str().unwrap().to_string()
}
/// Create a path prefix string for navigation from the `module_depth`
fn module_depth_to_path_prefix(module_depth: usize) -> String {
    (1..module_depth).map(|_| "../").collect::<String>()
}
/// Creates an HTML String from an [AttributesMap]
pub(crate) fn attrsmap_to_html_string(attributes: &AttributesMap) -> String {
    let attributes = attributes.get(&AttributeKind::DocComment);
    let mut docs = String::new();

    if let Some(vec_attrs) = attributes {
        for ident in vec_attrs.iter().flat_map(|attribute| &attribute.args) {
            writeln!(docs, "{}", ident.as_str())
                .expect("problem appending `ident.as_str()` to `docs` with `writeln` macro.");
        }
    }

    let mut options = ComrakOptions::default();
    options.render.hardbreaks = true;
    options.render.github_pre_lang = true;
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.parse.smart = true;
    options.parse.default_info_string = Some("sway".into());
    markdown_to_html(&format_docs(&docs), &options)
}
/// Takes a formatted String fn and returns only the function signature.
fn _trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}
/// Creates the HTML needed for the Fields section of a Struct document.
pub(crate) fn _struct_field_section(fields: Vec<TyStructField>) -> Box<dyn RenderBox> {
    box_html! {
        h2(id="fields", class="fields small-section-header") {
            : "Fields";
            a(class="anchor", href="#fields");
        }
        @ for field in fields {
            // TODO: Check for visibility of the field itself
            : _struct_field(field);
        }
    }
}
// make this and future kin funtions part of the renderable trait family
fn _struct_field(field: TyStructField) -> Box<dyn RenderBox> {
    let field_name = field.name.as_str().to_string();
    let struct_field_id = format!("structfield.{}", &field_name);
    box_html! {
        span(id=&struct_field_id, class="structfield small-section-header") {
            a(class="anchor field", href=format!("#{}", struct_field_id));
            code {
                : format!("{}: ", field_name);
                // TODO: Add links to types based on visibility
                : field.type_span.as_str().to_string();
            }
        }
    }
}
