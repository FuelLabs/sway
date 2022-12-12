use std::{fmt::Write, path::PathBuf};

use crate::{descriptor::DescriptorType, doc::Documentation};
use comrak::{markdown_to_html, ComrakOptions};
use horrorshow::{box_html, helper::doctype, html, prelude::*, Raw};
use sway_core::language::ty::{
    TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration, TyImplTrait,
    TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration,
};
use sway_core::transform::{AttributeKind, AttributesMap};
use sway_lsp::utils::markdown::format_docs;
use swayfmt::parse;

pub(crate) const ALL_DOC_FILENAME: &str = "all.html";

pub(crate) struct HTMLString(pub(crate) String);
pub(crate) type RenderedDocumentation = Vec<RenderedDocument>;
enum ItemType {
    Struct,
    Enum,
    Trait,
    Abi,
    Storage,
    Function,
    Constant,
}
type AllDoc = Vec<(ItemType, (String, Vec<String>, String))>;
/// A [Document] rendered to HTML.
pub(crate) struct RenderedDocument {
    pub(crate) module_prefix: Vec<String>,
    pub(crate) file_name: String,
    pub(crate) file_contents: HTMLString,
}
impl RenderedDocument {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from_raw_docs(raw: &Documentation, project_name: &String) -> RenderedDocumentation {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let mut all_doc: AllDoc = Default::default();
        for doc in raw {
            let module_prefix = doc.module_prefix.clone();
            let module_depth = module_prefix.len();
            let module = if module_prefix.last().is_some() {
                module_prefix.last().unwrap().to_string()
            } else {
                project_name.to_string()
            };
            let file_name = doc.file_name();
            let decl_ty = doc.desc_ty.as_str().to_string();
            let rendered_content = match &doc.desc_ty {
                DescriptorType::Struct(struct_decl) => {
                    let path_str = if module_depth == 0 {
                        struct_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &struct_decl.name)
                    };
                    all_doc.push((
                        ItemType::Struct,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    struct_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Enum(enum_decl) => {
                    let path_str = if module_depth == 0 {
                        enum_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &enum_decl.name)
                    };
                    all_doc.push((
                        ItemType::Enum,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    enum_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Trait(trait_decl) => {
                    let path_str = if module_depth == 0 {
                        trait_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &trait_decl.name)
                    };
                    all_doc.push((
                        ItemType::Trait,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    trait_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Abi(abi_decl) => {
                    let path_str = if module_depth == 0 {
                        abi_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &abi_decl.name)
                    };
                    all_doc.push((
                        ItemType::Abi,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    abi_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Storage(storage_decl) => {
                    all_doc.push((
                        ItemType::Storage,
                        (
                            format!("{}::ContractStorage", &module),
                            module_prefix.clone(),
                            file_name.clone(),
                        ),
                    ));
                    storage_decl.render(module, module_depth, decl_ty)
                }
                // TODO: Figure out how to represent impl traits
                DescriptorType::ImplTraitDesc(impl_trait_decl) => {
                    impl_trait_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Function(fn_decl) => {
                    let path_str = if module_depth == 0 {
                        fn_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &fn_decl.name)
                    };
                    all_doc.push((
                        ItemType::Function,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    fn_decl.render(module, module_depth, decl_ty)
                }
                DescriptorType::Const(const_decl) => {
                    let path_str = if module_depth == 0 {
                        const_decl.name.as_str().to_string()
                    } else {
                        format!("{}::{}", &module, &const_decl.name)
                    };
                    all_doc.push((
                        ItemType::Constant,
                        (path_str, module_prefix.clone(), file_name.clone()),
                    ));
                    const_decl.render(module, module_depth, decl_ty)
                }
            };
            rendered_docs.push(Self {
                module_prefix,
                file_name,
                file_contents: HTMLString(page_from(rendered_content)),
            })
        }
        // All Doc
        rendered_docs.push(Self {
            module_prefix: vec![],
            file_name: ALL_DOC_FILENAME.to_string(),
            file_contents: HTMLString(page_from(all_items(project_name.to_string(), &all_doc))),
        });
        rendered_docs
    }
}

fn page_from(rendered_content: Box<dyn RenderBox>) -> String {
    let markup = html! {
        : doctype::HTML;
        html {
            : rendered_content
        }
    };

    markup.into_string().unwrap()
}

/// Basic HTML header component
fn html_head(
    module_depth: usize,
    location: String,
    decl_ty: String,
    decl_name: String,
) -> Box<dyn RenderBox> {
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
                content=format!("API documentation for the Sway `{decl_name}` {decl_ty} in `{location}`.")
            );
            meta(name="keywords", content=format!("sway, swaylang, sway-lang, {decl_name}"));
            link(rel="icon", href=favicon);
            title: format!("{decl_name} in {location} - Sway");
            link(rel="stylesheet", type="text/css", href=normalize);
            link(rel="stylesheet", type="text/css", href=swaydoc, id="mainThemeStyle");
            link(rel="stylesheet", type="text/css", href=ayu);
            // TODO: Add links for fonts
        }
    }
}
/// HTML body component
fn html_body(
    module_depth: usize,
    decl_ty: String,
    decl_name: String,
    code_str: String,
    item_attrs: String,
) -> Box<dyn RenderBox> {
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
                                    : format!("{} ", &decl_ty);
                                    // TODO: add qualified path anchors
                                    a(class=&decl_ty, href="#") {
                                        : &decl_name;
                                    }
                                }
                            }
                        }
                        div(class="docblock item-decl") {
                            pre(class=format!("sway {}", &decl_ty)) {
                                code { : code_str; }
                            }
                        }
                        @ if !item_attrs.is_empty() {
                            // expand or hide description of main code block
                            details(class="swaydoc-toggle top-doc", open) {
                                summary(class="hideme") {
                                    span { : "Expand description" }
                                }
                                // this is the description
                                div(class="docblock") {
                                    : Raw(item_attrs)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
/// crate level index.html
fn _crate_index() -> Box<dyn RenderBox> {
    box_html! {}
}
/// crate level, all items belonging to a crate
fn all_items(crate_name: String, all_doc: &AllDoc) -> Box<dyn RenderBox> {
    // TODO: find a better way to do this
    //
    // we need to have a finalized list for the all doc
    let mut struct_items: Vec<(String, String)> = Vec::new();
    let mut enum_items: Vec<(String, String)> = Vec::new();
    let mut trait_items: Vec<(String, String)> = Vec::new();
    let mut abi_items: Vec<(String, String)> = Vec::new();
    let mut storage_items: Vec<(String, String)> = Vec::new();
    let mut fn_items: Vec<(String, String)> = Vec::new();
    let mut const_items: Vec<(String, String)> = Vec::new();
    for (ty, (path_str, module_prefix, file_name)) in all_doc {
        match ty {
            ItemType::Struct => struct_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Enum => enum_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Trait => trait_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Abi => abi_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Storage => storage_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Function => fn_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
            ItemType::Constant => const_items.push((
                path_str.clone(),
                qualified_file_path(module_prefix, file_name.clone()),
            )),
        }
    }
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
            : sidebar(0, format!("Crate {crate_name}"), ALL_DOC_FILENAME.to_string());
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
                                        placeholder="Search...",
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
/// Renders the items list from each item kind and adds the links to each file path
fn all_items_list(title: String, list_items: Vec<(String, String)>) -> Box<dyn RenderBox> {
    box_html! {
        h3(id=format!("{title}")) { : title.clone(); }
        ul(class=format!("{} docblock", title.to_lowercase())) {
            @ for (path_str, file_path) in list_items {
                li {
                    a(href=file_path) { : path_str; }
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
        file_path.push(prefix)
    }
    file_path.push(file_name);

    file_path.to_str().unwrap().to_string()
}
/// Create a path prefix string for navigation from the `module_depth`
fn module_depth_to_path_prefix(module_depth: usize) -> String {
    (0..module_depth).map(|_| "../").collect::<String>()
}
/// Creates an HTML String from an [AttributesMap]
fn attrsmap_to_html_string(attributes: &AttributesMap) -> String {
    let attributes = attributes.get(&AttributeKind::Doc);
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
fn trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}

trait Renderable {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox>;
}

impl Renderable for TyStructDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyStructDeclaration {
            name,
            fields: _,
            type_parameters: _,
            visibility: _,
            attributes,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemStruct>(span.as_str());
        let struct_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, struct_attributes);
        }
    }
}
impl Renderable for TyEnumDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyEnumDeclaration {
            name,
            type_parameters: _,
            attributes,
            variants: _,
            visibility: _,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemEnum>(span.as_str());
        let enum_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, enum_attributes);
        }
    }
}
impl Renderable for TyTraitDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyTraitDeclaration {
            name,
            interface_surface: _,
            methods: _,
            visibility: _,
            attributes,
            supertraits: _,
            span,
            type_parameters: _,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemTrait>(span.as_str());
        let trait_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, trait_attributes);
        }
    }
}
impl Renderable for TyAbiDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyAbiDeclaration {
            name,
            interface_surface: _,
            methods: _,
            attributes,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemAbi>(span.as_str());
        let abi_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, abi_attributes);
        }
    }
}
impl Renderable for TyStorageDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyStorageDeclaration {
            fields: _,
            span,
            attributes,
        } = &self;
        let name = "Contract Storage".to_string();
        let code_str = parse::parse_format::<sway_ast::ItemStorage>(span.as_str());
        let storage_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, storage_attributes);
        }
    }
}
impl Renderable for TyImplTrait {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyImplTrait {
            impl_type_parameters: _,
            trait_name,
            trait_type_arguments: _,
            methods: _,
            implementing_for_type_id: _,
            type_implementing_for_span: _,
            span,
        } = &self;
        let name = trait_name.suffix.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemImpl>(span.as_str());
        // let impl_trait_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, "".to_string());
        }
    }
}
impl Renderable for TyFunctionDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyFunctionDeclaration {
            name,
            body: _,
            parameters: _,
            span,
            attributes,
            return_type: _,
            initial_return_type: _,
            type_parameters: _,
            return_type_span: _,
            purity: _,
            is_contract_call: _,
            visibility: _,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = trim_fn_body(parse::parse_format::<sway_ast::ItemFn>(span.as_str()));
        let function_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, function_attributes);
        }
    }
}
impl Renderable for TyConstantDeclaration {
    fn render(&self, module: String, module_depth: usize, decl_ty: String) -> Box<dyn RenderBox> {
        let TyConstantDeclaration {
            name,
            value: _,
            attributes,
            visibility: _,
            return_type: _,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_str = parse::parse_format::<sway_ast::ItemConst>(span.as_str());
        let const_attributes = attrsmap_to_html_string(attributes);
        box_html! {
            : html_head(module_depth, module.clone(), decl_ty.clone(), name.clone());
            : html_body(module_depth, decl_ty.clone(), name.clone(), code_str, const_attributes);
        }
    }
}
