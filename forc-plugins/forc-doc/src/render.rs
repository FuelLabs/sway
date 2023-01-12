use crate::doc::{Documentation, ModuleInfo};
use comrak::{markdown_to_html, ComrakOptions};
use horrorshow::{box_html, helper::doctype, html, prelude::*, Raw};
use std::fmt::Write;
use sway_core::language::ty::{
    TyDeclaration, TyEnumVariant, TyStorageField, TyStructField, TyTraitFn,
};
use sway_core::transform::{AttributeKind, AttributesMap};
use sway_lsp::utils::markdown::format_docs;
use sway_types::BaseIdent;

pub(crate) const ALL_DOC_FILENAME: &str = "all.html";
pub(crate) const INDEX_FILENAME: &str = "index.html";
pub(crate) trait Renderable {
    fn render(self) -> Box<dyn RenderBox>;
}
/// A [Document] rendered to HTML.
pub(crate) struct RenderedDocument {
    pub(crate) module_info: Vec<String>,
    pub(crate) html_file_name: String,
    pub(crate) file_contents: HTMLString,
}
#[derive(Default)]
pub(crate) struct RenderedDocumentation(pub(crate) Vec<RenderedDocument>);

impl RenderedDocumentation {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from(raw: Documentation) -> RenderedDocumentation {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let mut all_docs: AllDocs = Default::default();
        for doc in raw {
            let html_file_name = doc.html_file_name();

            rendered_docs.0.push(RenderedDocument {
                module_info: doc.module_info.0.clone(),
                html_file_name: html_file_name.clone(),
                file_contents: HTMLString::from(doc.clone().render()),
            });
            all_docs.0.push(AllDocItem {
                item_name: doc.item_header.item_name.clone(),
                ty_decl: doc.item_body.ty_decl.clone(),
                module_info: doc.module_info,
                html_file_name,
            });
        }
        // AllDocIndex
        rendered_docs.0.push(RenderedDocument {
            module_info: vec![],
            html_file_name: ALL_DOC_FILENAME.to_string(),
            file_contents: HTMLString::from(
                AllDocIndex {
                    project_name: ModuleInfo::from_vec(vec![all_docs.project_name().to_owned()]),
                    all_docs,
                }
                .render(),
            ),
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
    pub(crate) module_info: ModuleInfo,
    pub(crate) friendly_name: &'static str,
    pub(crate) item_name: BaseIdent,
}
impl Renderable for ItemHeader {
    /// Basic HTML header component
    fn render(self) -> Box<dyn RenderBox> {
        let ItemHeader {
            module_info,
            friendly_name,
            item_name,
        } = self;

        let favicon = module_info.to_html_shorthand_path_string("assets/sway-logo.svg");
        let normalize = module_info.to_html_shorthand_path_string("assets/normalize.css");
        let swaydoc = module_info.to_html_shorthand_path_string("assets/swaydoc.css");
        let ayu = module_info.to_html_shorthand_path_string("assets/ayu.css");

        box_html! {
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
    pub(crate) module_info: ModuleInfo,
    pub(crate) ty_decl: TyDeclaration,
    /// The item name varies depending on type.
    /// We store it during info gathering to avoid
    /// multiple match statements.
    pub(crate) item_name: BaseIdent,
    pub(crate) code_str: String,
    pub(crate) attrs_opt: Option<String>,
    pub(crate) item_context: ItemContext,
}
impl SidebarNav for ItemBody {
    fn sidebar(&self) -> Sidebar {
        Sidebar {
            module_info: self.module_info.clone(),
            href_path: INDEX_FILENAME.to_owned(),
        }
    }
}
impl Renderable for ItemBody {
    /// HTML body component
    fn render(self) -> Box<dyn RenderBox> {
        let sidebar = self.sidebar();
        let ItemBody {
            module_info: _,
            ty_decl,
            item_name,
            code_str,
            attrs_opt,
            item_context,
        } = self;

        let decl_ty = ty_decl.doc_name();
        let friendly_name = ty_decl.friendly_name();

        box_html! {
            body(class=format!("swaydoc {decl_ty}")) {
                : sidebar.render();
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
                                            : item_name.as_str();
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
                            @ if item_context.context.is_some() {
                                : item_context.render();
                            }
                        }
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
pub(crate) enum ContextType {
    /// structs
    StructFields(Vec<TyStructField>),
    /// storage
    StorageFields(Vec<TyStorageField>),
    /// enums
    EnumVariants(Vec<TyEnumVariant>),
    /// traits and abi, this can be split
    /// at a later date if need be
    RequiredMethods(Vec<TyTraitFn>),
}
#[derive(Clone)]
pub(crate) struct ItemContext {
    pub(crate) context: Option<ContextType>,
    // TODO: All other Implementation types, eg
    // implementations on foreign types, method implementations, etc.
}
impl Renderable for ItemContext {
    fn render(self) -> Box<dyn RenderBox> {
        const FIELDS: &str = "Fields";
        const VARIANTS: &str = "Variants";
        const REQUIRED_METHODS: &str = "Required Methods";
        match self.context.unwrap() {
            ContextType::StructFields(fields) => context_section(fields, FIELDS),
            ContextType::StorageFields(fields) => context_section(fields, FIELDS),
            ContextType::EnumVariants(variants) => context_section(variants, VARIANTS),
            ContextType::RequiredMethods(methods) => context_section(methods, REQUIRED_METHODS),
        }
    }
}
/// Dynamically creates the context section of an item.
fn context_section<'title, S: Renderable + 'static>(
    list: Vec<S>,
    title: &'title str,
) -> Box<dyn RenderBox + 'title> {
    let lct = html_title_string(title);
    box_html! {
        h2(id=&lct, class=format!("{} small-section-header", &lct)) {
            : title;
            a(class="anchor", href=format!("#{}", lct));
        }
        @ for item in list {
            // TODO: Check for visibility of the field itself
            : item.render();
        }
    }
}
fn html_title_string(title: &str) -> String {
    if title.contains(' ') {
        title
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-")
    } else {
        title.to_lowercase()
    }
}
impl Renderable for TyStructField {
    fn render(self) -> Box<dyn RenderBox> {
        let struct_field_id = format!("structfield.{}", self.name.as_str());
        box_html! {
            span(id=&struct_field_id, class="structfield small-section-header") {
                a(class="anchor field", href=format!("#{}", struct_field_id));
                code {
                    : format!("{}: ", self.name.as_str());
                    // TODO: Add links to types based on visibility
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(attrsmap_to_html_str(self.attributes));
                }
            }
        }
    }
}
impl Renderable for TyStorageField {
    fn render(self) -> Box<dyn RenderBox> {
        let storage_field_id = format!("storagefield.{}", self.name.as_str());
        box_html! {
            span(id=&storage_field_id, class="storagefield small-section-header") {
                a(class="anchor field", href=format!("#{}", storage_field_id));
                code {
                    : format!("{}: ", self.name.as_str());
                    // TODO: Add links to types based on visibility
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(attrsmap_to_html_str(self.attributes));
                }
            }
        }
    }
}
impl Renderable for TyEnumVariant {
    fn render(self) -> Box<dyn RenderBox> {
        let enum_variant_id = format!("variant.{}", self.name.as_str());
        box_html! {
            h3(id=&enum_variant_id, class="variant small-section-header") {
                a(class="anchor field", href=format!("#{}", enum_variant_id));
                code {
                    : format!("{}: ", self.name.as_str());
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(attrsmap_to_html_str(self.attributes));
                }
            }
        }
    }
}
impl Renderable for TyTraitFn {
    fn render(self) -> Box<dyn RenderBox> {
        // there is likely a better way we can do this while simultaneously storing the
        // string slices we need like "&mut "
        let mut fn_sig = format!("fn {}(", self.name.as_str());
        for param in &self.parameters {
            let mut param_str = String::new();
            if param.is_reference {
                write!(param_str, "&")
                    .expect("failed to write reference to param_str for method fn");
            }
            if param.is_mutable {
                write!(param_str, "mut ")
                    .expect("failed to write mutability to param_str for method fn");
            }
            if param.is_self() {
                write!(param_str, "self,")
                    .expect("failed to write self to param_str for method fn");
            } else {
                write!(
                    fn_sig,
                    "{} {},",
                    param.name.as_str(),
                    param.type_span.as_str()
                )
                .expect("failed to write name/type to param_str for method fn");
            }
        }
        write!(fn_sig, ") -> {}", self.return_type_span.as_str())
            .expect("failed to write return type to param_str for method fn");
        let multiline = fn_sig.chars().count() >= 60;

        let method_id = format!("tymethod.{}", self.name.as_str());
        box_html! {
            div(class="methods") {
                div(id=&method_id, class="method has-srclink") {
                    h4(class="code-header") {
                        : "fn ";
                        a(class="fnname", href=format!("#{}", method_id)) {
                            : self.name.as_str();
                        }
                        : "(";
                        @ if multiline {
                            @ for param in &self.parameters {
                                br;
                                : "    ";
                                @ if param.is_reference {
                                    : "&";
                                }
                                @ if param.is_mutable {
                                    : "mut ";
                                }
                                @ if param.is_self() {
                                    : "self,"
                                } else {
                                    : param.name.as_str();
                                    : ": ";
                                    : param.type_span.as_str();
                                    : ","
                                }
                            }
                            br;
                            : ")";
                        } else {
                            @ for param in &self.parameters {
                                @ if param.is_reference {
                                    : "&";
                                }
                                @ if param.is_mutable {
                                    : "mut ";
                                }
                                @ if param.is_self() {
                                    : "self, "
                                } else {
                                    : param.name.as_str();
                                    : ": ";
                                    : param.type_span.as_str();
                                }
                                @ if param.name.as_str()
                                    != self.parameters.last()
                                    .expect("no last element in trait method parameters list")
                                    .name.as_str() {
                                    : ", ";
                                }
                            }
                            : ") -> ";
                        }
                        : self.return_type_span.as_str();
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
struct AllDocItem {
    item_name: BaseIdent,
    ty_decl: TyDeclaration,
    module_info: ModuleInfo,
    html_file_name: String,
}
impl AllDocItem {
    fn link(&self) -> ItemLink {
        ItemLink {
            name: self
                .module_info
                .to_path_literal_string(self.item_name.as_str()),
            hyperlink: self.module_info.to_file_path_string(&self.html_file_name),
        }
    }
}
/// Used for creating links.
///
/// This could be a path literal with a link e.g `proj_name::foo::Foo`,
/// or just the item name: `Foo`.
struct ItemLink {
    name: String,
    hyperlink: String,
}
#[derive(Default, Clone)]
struct AllDocs(Vec<AllDocItem>);
impl AllDocs {
    /// A wrapper for `ModuleInfo::project_name()`.
    fn project_name(&self) -> &str {
        self.0.first().unwrap().module_info.project_name()
    }
}
#[derive(Clone)]
struct AllDocIndex {
    /// A [ModuleInfo] with only the project name.
    project_name: ModuleInfo,
    /// All doc items.
    all_docs: AllDocs,
}
impl SidebarNav for AllDocIndex {
    fn sidebar(&self) -> Sidebar {
        Sidebar {
            module_info: self.project_name.clone(),
            href_path: self
                .project_name
                .to_html_shorthand_path_string(ALL_DOC_FILENAME),
        }
    }
}
impl Renderable for AllDocIndex {
    /// crate level, all items belonging to a crate
    fn render(self) -> Box<dyn RenderBox> {
        // TODO: find a better way to do this
        //
        // we need to have a finalized list of links for the all doc
        let mut struct_items: Vec<ItemLink> = Vec::new();
        let mut enum_items: Vec<ItemLink> = Vec::new();
        let mut trait_items: Vec<ItemLink> = Vec::new();
        let mut abi_items: Vec<ItemLink> = Vec::new();
        let mut storage_items: Vec<ItemLink> = Vec::new();
        let mut fn_items: Vec<ItemLink> = Vec::new();
        let mut const_items: Vec<ItemLink> = Vec::new();

        for doc_item in &self.all_docs.0 {
            use TyDeclaration::*;
            match doc_item.ty_decl {
                StructDeclaration(_) => struct_items.push(doc_item.link()),
                EnumDeclaration(_) => enum_items.push(doc_item.link()),
                TraitDeclaration(_) => trait_items.push(doc_item.link()),
                AbiDeclaration(_) => abi_items.push(doc_item.link()),
                StorageDeclaration(_) => storage_items.push(doc_item.link()),
                FunctionDeclaration(_) => fn_items.push(doc_item.link()),
                ConstantDeclaration(_) => const_items.push(doc_item.link()),
                _ => {} // TODO: ImplTraitDeclaration
            }
        }
        let sidebar = self.sidebar();
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
                : sidebar.render();
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
fn all_items_list(title: String, list_items: Vec<ItemLink>) -> Box<dyn RenderBox> {
    box_html! {
        h3(id=format!("{title}")) { : title.clone(); }
        ul(class=format!("{} docblock", title.to_lowercase())) {
            @ for item in list_items {
                li {
                    a(href=item.hyperlink) { : item.name; }
                }
            }
        }
    }
}
trait SidebarNav {
    /// Create sidebar component.
    fn sidebar(&self) -> Sidebar;
}
/// Sidebar component for quick navigation.
struct Sidebar {
    module_info: ModuleInfo,
    href_path: String,
}
impl Renderable for Sidebar {
    fn render(self) -> Box<dyn RenderBox> {
        let logo_path = self
            .module_info
            .to_html_shorthand_path_string("assets/sway-logo.svg");

        box_html! {
            nav(class="sidebar") {
                a(class="sidebar-logo", href=&self.href_path) {
                    div(class="logo-container") {
                        img(class="sway-logo", src=logo_path, alt="logo");
                    }
                }
                h2(class="location") {
                    a(href=&self.href_path) { : self.module_info.location(); }
                }
                div(class="sidebar-elems") {
                    section {
                        // TODO: add connections between item contents and
                        // sidebar nav. This will be dynamic e.g. "Variants"
                        // for Enum, and "Fields" for Structs, and also will be different
                        // based on the type of section e.g. Index, All or Item.
                    }
                }
            }
        }
    }
}
/// Creates an HTML String from an [AttributesMap]
pub(crate) fn attrsmap_to_html_str(attributes: AttributesMap) -> String {
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
pub(crate) fn trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}
