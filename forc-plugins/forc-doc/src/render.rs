use crate::{descriptor::DescriptorType, doc::Documentation};
use horrorshow::{box_html, helper::doctype, html, prelude::*};
use sway_core::language::ty::{
    TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration, TyImplTrait,
    TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration,
};
use sway_core::transform::{AttributeKind, AttributesMap};

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
type AllDoc = Vec<(ItemType, (String, String))>;
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
            let module = if module_prefix.last().is_some() {
                module_prefix.last().unwrap().to_string()
            } else {
                project_name.to_string()
            };
            let file_name = doc.file_name();
            let decl_ty = doc.desc_ty.as_str().to_string();
            let rendered_content = match &doc.desc_ty {
                DescriptorType::Struct(struct_decl) => {
                    all_doc.push((
                        ItemType::Struct,
                        (
                            format!("{}::{}", &module, &struct_decl.name),
                            file_name.clone(),
                        ),
                    ));
                    struct_decl.render(module, decl_ty)
                }
                DescriptorType::Enum(enum_decl) => {
                    all_doc.push((
                        ItemType::Enum,
                        (
                            format!("{}::{}", &module, &enum_decl.name),
                            file_name.clone(),
                        ),
                    ));
                    enum_decl.render(module, decl_ty)
                }
                DescriptorType::Trait(trait_decl) => {
                    all_doc.push((
                        ItemType::Trait,
                        (
                            format!("{}::{}", &module, &trait_decl.name),
                            file_name.clone(),
                        ),
                    ));
                    trait_decl.render(module, decl_ty)
                }
                DescriptorType::Abi(abi_decl) => {
                    all_doc.push((
                        ItemType::Abi,
                        (
                            format!("{}::{}", &module, &abi_decl.name),
                            file_name.clone(),
                        ),
                    ));
                    abi_decl.render(module, decl_ty)
                }
                DescriptorType::Storage(storage_decl) => {
                    all_doc.push((
                        ItemType::Storage,
                        (format!("{}::ContractStorage", &module), file_name.clone()),
                    ));
                    storage_decl.render(module, decl_ty)
                }
                // TODO: Figure out how to represent impl traits
                DescriptorType::ImplTraitDesc(impl_trait_decl) => {
                    impl_trait_decl.render(module, decl_ty)
                }
                DescriptorType::Function(fn_decl) => {
                    all_doc.push((
                        ItemType::Function,
                        (format!("{}::{}", &module, &fn_decl.name), file_name.clone()),
                    ));
                    fn_decl.render(module, decl_ty)
                }
                DescriptorType::Const(const_decl) => {
                    all_doc.push((
                        ItemType::Constant,
                        (
                            format!("{}::{}", &module, &const_decl.name),
                            file_name.clone(),
                        ),
                    ));
                    const_decl.render(module, decl_ty)
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
            file_name: "all.html".to_string(),
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
fn html_head(location: String, decl_ty: String, decl_name: String) -> Box<dyn RenderBox> {
    box_html! {
        head {
            meta(charset="utf-8");
            meta(name="viewport", content="width=device-width, initial-scale=1.0");
            meta(name="generator", content="forcdoc");
            meta(
                name="description",
                content=format!("API documentation for the Sway `{decl_name}` {decl_ty} in `{location}`.")
            );
            meta(name="keywords", content=format!("sway, swaylang, sway-lang, {decl_name}"));
            title: format!("{decl_name} in {location} - Sway");
            // TODO: Add links for CSS & Fonts
        }
    }
}
/// HTML body component
fn html_body(
    module: String,
    decl_ty: String,
    decl_name: String,
    code_span: String,
    item_attrs: String,
) -> Box<dyn RenderBox> {
    box_html! {
        body(class=format!("forcdoc {decl_ty}")) {
            // TODO: create nav sidebar
            // create main
            // create main content

            // this is the main code block
            div(class="docblock item-decl") {
                pre(class=format!("sway {decl_ty}")) {
                    code { : code_span; }
                }
            }
            // expand or hide description of main code block
            details(class="forcdoc-toggle top-doc", open) {
                summary(class="hideme") {
                    span { : "Expand description" }
                }
                // this is the description
                div(class="docblock") {
                    p { : item_attrs; }
                }
            }
        }
    }
}
// crate level index.html
fn crate_index() -> Box<dyn RenderBox> {
    box_html! {}
}
// crate level, all items belonging to a crate
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
    for (ty, (path_str, file_name)) in all_doc {
        match ty {
            ItemType::Struct => struct_items.push((path_str.clone(), file_name.clone())),
            ItemType::Enum => enum_items.push((path_str.clone(), file_name.clone())),
            ItemType::Trait => trait_items.push((path_str.clone(), file_name.clone())),
            ItemType::Abi => abi_items.push((path_str.clone(), file_name.clone())),
            ItemType::Storage => storage_items.push((path_str.clone(), file_name.clone())),
            ItemType::Function => fn_items.push((path_str.clone(), file_name.clone())),
            ItemType::Constant => const_items.push((path_str.clone(), file_name.clone())),
        }
    }
    box_html! {
        head {
            meta(charset="utf-8");
            meta(name="viewport", content="width=device-width, initial-scale=1.0");
            meta(name="generator", content="forcdoc");
            meta(
                name="description",
                content="List of all items in this crate"
            );
            meta(name="keywords", content="sway, swaylang, sway-lang");
            title: "List of all items in this crate";
        }
        body(class="forcdoc mod") {
            : sidebar(format!("Crate {crate_name}"));
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
fn all_items_list(title: String, list_items: Vec<(String, String)>) -> Box<dyn RenderBox> {
    box_html! {
        h3(id=format!("{title}")) { : title.clone(); }
        ul(class=format!("{} docblock", title.to_lowercase())) {
            @ for (path_str, file_name) in list_items {
                li {
                    a(href=file_name) { : path_str; }
                }
            }
        }
    }
}
// module level index.html
// for each module we need to create an index
// that will have all of the item docs in it
fn module_index() -> Box<dyn RenderBox> {
    box_html! {}
}
fn sidebar(location: String /* sidebar_items: Option<Vec<String>>, */) -> Box<dyn RenderBox> {
    box_html! {
        nav(class="sidebar") {
            a(class="sidebar-logo", href="../index.html") {
                div(class="logo-container") {
                    img(class="sway-logo", src="../sway-logo.svg", alt="logo");
                }
            }
            h2(class="location") { : location; }
        }
    }
}

fn doc_attributes_to_string_vec(attributes: &AttributesMap) -> String {
    let attributes = attributes.get(&AttributeKind::Doc);
    let mut attr_strings = String::new();
    if let Some(vec_attrs) = attributes {
        for attribute in vec_attrs {
            for ident in &attribute.args {
                attr_strings.push_str(ident.as_str())
            }
        }
    }

    attr_strings
}
trait Renderable {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox>;
}

impl Renderable for TyStructDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyStructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            attributes,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let struct_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, struct_attributes);
        }
    }
}
impl Renderable for TyEnumDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyEnumDeclaration {
            name,
            type_parameters,
            attributes,
            variants,
            visibility,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let enum_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, enum_attributes);
        }
    }
}
impl Renderable for TyTraitDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyTraitDeclaration {
            name,
            interface_surface,
            methods,
            visibility,
            attributes,
            supertraits,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let trait_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, trait_attributes);
        }
    }
}
impl Renderable for TyAbiDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyAbiDeclaration {
            name,
            interface_surface,
            methods,
            attributes,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let abi_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, abi_attributes);
        }
    }
}
impl Renderable for TyStorageDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyStorageDeclaration {
            fields,
            span,
            attributes,
        } = &self;
        let name = "Contract Storage".to_string();
        let code_span = span.as_str().to_string();
        let storage_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, storage_attributes);
        }
    }
}
impl Renderable for TyImplTrait {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyImplTrait {
            impl_type_parameters,
            trait_name,
            trait_type_parameters,
            methods,
            implementing_for_type_id,
            type_implementing_for_span,
            span,
        } = &self;
        let name = trait_name.suffix.as_str().to_string();
        let code_span = span.as_str().to_string();
        // let impl_trait_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, "".to_string());
        }
    }
}
impl Renderable for TyFunctionDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyFunctionDeclaration {
            name,
            body,
            parameters,
            span,
            attributes,
            return_type,
            initial_return_type,
            type_parameters,
            return_type_span,
            purity,
            is_contract_call,
            visibility,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let function_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, function_attributes);
        }
    }
}
impl Renderable for TyConstantDeclaration {
    fn render(&self, module: String, decl_ty: String) -> Box<dyn RenderBox> {
        let TyConstantDeclaration {
            name,
            value,
            attributes,
            visibility,
            span,
        } = &self;
        let name = name.as_str().to_string();
        let code_span = span.as_str().to_string();
        let const_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone(), code_span, const_attributes);
        }
    }
}
