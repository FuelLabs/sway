use crate::doc::{Documentation, ModuleInfo, ModulePrefix};
use anyhow::Result;
use comrak::{markdown_to_html, ComrakOptions};
use horrorshow::{box_html, helper::doctype, html, prelude::*, Raw};
use std::collections::BTreeMap;
use std::fmt::Write;
use sway_core::language::ty::{
    TyDeclaration::{self, *},
    TyEnumVariant, TyStorageField, TyStructField, TyTraitFn,
};
use sway_core::transform::{AttributeKind, AttributesMap};
use sway_lsp::utils::markdown::format_docs;
use sway_types::BaseIdent;

pub(crate) const ALL_DOC_FILENAME: &str = "all.html";
pub(crate) const INDEX_FILENAME: &str = "index.html";
pub(crate) const IDENTITY: &str = "#";
pub(crate) trait Renderable {
    fn render(self) -> Result<Box<dyn RenderBox>>;
}
/// A [Document] rendered to HTML.
pub(crate) struct RenderedDocument {
    pub(crate) module_info: ModuleInfo,
    pub(crate) html_filename: String,
    pub(crate) file_contents: HTMLString,
}
#[derive(Default)]
pub(crate) struct RenderedDocumentation(pub(crate) Vec<RenderedDocument>);

impl RenderedDocumentation {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from(raw: Documentation, forc_version: Option<String>) -> Result<RenderedDocumentation> {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let root_module = match raw.first() {
            Some(doc) => ModuleInfo::from_vec(vec![doc.module_info.project_name().to_owned()]),
            None => panic!("Project does not contain a root module"),
        };
        let mut all_docs = DocLinks {
            style: DocStyle::AllDoc,
            links: Default::default(),
        };
        let mut module_map: BTreeMap<ModulePrefix, BTreeMap<BlockTitle, Vec<DocLink>>> =
            BTreeMap::new();
        for doc in raw {
            rendered_docs.0.push(RenderedDocument {
                module_info: doc.module_info.clone(),
                html_filename: doc.html_filename(),
                file_contents: HTMLString::from(doc.clone().render()?),
            });
            // Here we gather all of the `doc_links` based on which module they belong to.
            let location = doc.module_info.location().to_string();
            match module_map.get_mut(&location) {
                Some(doc_links) => {
                    match doc.item_body.ty_decl {
                        StructDeclaration(_) => match doc_links.get_mut(&BlockTitle::Structs) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Structs, vec![doc.link()]);
                            }
                        },
                        EnumDeclaration(_) => match doc_links.get_mut(&BlockTitle::Enums) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Enums, vec![doc.link()]);
                            }
                        },
                        TraitDeclaration(_) => match doc_links.get_mut(&BlockTitle::Traits) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Traits, vec![doc.link()]);
                            }
                        },
                        AbiDeclaration(_) => match doc_links.get_mut(&BlockTitle::Abi) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Abi, vec![doc.link()]);
                            }
                        },
                        StorageDeclaration(_) => {
                            match doc_links.get_mut(&BlockTitle::ContractStorage) {
                                Some(links) => links.push(doc.link()),
                                None => {
                                    doc_links.insert(BlockTitle::ContractStorage, vec![doc.link()]);
                                }
                            }
                        }
                        FunctionDeclaration(_) => match doc_links.get_mut(&BlockTitle::Functions) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Functions, vec![doc.link()]);
                            }
                        },
                        ConstantDeclaration(_) => match doc_links.get_mut(&BlockTitle::Constants) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Constants, vec![doc.link()]);
                            }
                        },
                        _ => {} // TODO: ImplTraitDeclaration
                    }
                }
                None => {
                    let mut doc_links: BTreeMap<BlockTitle, Vec<DocLink>> = BTreeMap::new();
                    match doc.item_body.ty_decl {
                        StructDeclaration(_) => {
                            doc_links.insert(BlockTitle::Structs, vec![doc.link()]);
                        }
                        EnumDeclaration(_) => {
                            doc_links.insert(BlockTitle::Enums, vec![doc.link()]);
                        }
                        TraitDeclaration(_) => {
                            doc_links.insert(BlockTitle::Traits, vec![doc.link()]);
                        }
                        AbiDeclaration(_) => {
                            doc_links.insert(BlockTitle::Abi, vec![doc.link()]);
                        }
                        StorageDeclaration(_) => {
                            doc_links.insert(BlockTitle::ContractStorage, vec![doc.link()]);
                        }
                        FunctionDeclaration(_) => {
                            doc_links.insert(BlockTitle::Functions, vec![doc.link()]);
                        }
                        ConstantDeclaration(_) => {
                            doc_links.insert(BlockTitle::Constants, vec![doc.link()]);
                        }
                        _ => {} // TODO: ImplTraitDeclaration
                    }
                    module_map.insert(location.clone(), doc_links);
                }
            }
            // Create links to child modules.
            if let Some(parent_module) = doc.module_info.parent() {
                let module_link = DocLink {
                    name: location.clone(),
                    module_info: doc.module_info.to_owned(),
                    html_filename: INDEX_FILENAME.to_owned(),
                    preview_opt: None,
                };
                match module_map.get_mut(parent_module) {
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
                        module_map.insert(parent_module.clone(), doc_links);
                    }
                }
            }
            // Above we check for the module a link belongs to, here we want _all_ links so the check is much more shallow.
            match doc.item_body.ty_decl {
                StructDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Structs) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Structs, vec![doc.link()]);
                    }
                },
                EnumDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Enums) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Enums, vec![doc.link()]);
                    }
                },
                TraitDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Traits) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Traits, vec![doc.link()]);
                    }
                },
                AbiDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Abi) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Abi, vec![doc.link()]);
                    }
                },
                StorageDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::ContractStorage)
                {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs
                            .links
                            .insert(BlockTitle::ContractStorage, vec![doc.link()]);
                    }
                },
                FunctionDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Functions) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs
                            .links
                            .insert(BlockTitle::Functions, vec![doc.link()]);
                    }
                },
                ConstantDeclaration(_) => match all_docs.links.get_mut(&BlockTitle::Constants) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs
                            .links
                            .insert(BlockTitle::Constants, vec![doc.link()]);
                    }
                },
                _ => {} // TODO: ImplTraitDeclaration
            }
        }
        // ProjectIndex
        match module_map.get(root_module.location()) {
            Some(doc_links) => rendered_docs.0.push(RenderedDocument {
                module_info: root_module.clone(),
                html_filename: INDEX_FILENAME.to_string(),
                file_contents: HTMLString::from(
                    ModuleIndex {
                        version_opt: forc_version,
                        module_info: root_module.clone(),
                        module_docs: DocLinks {
                            style: DocStyle::ProjectIndex,
                            links: doc_links.to_owned(),
                        },
                    }
                    .render()?,
                ),
            }),
            None => panic!("Project does not contain a root module."),
        }
        if module_map.len() > 1 {
            module_map.remove_entry(root_module.location());

            // ModuleIndex(s)
            for (_, doc_links) in module_map {
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
                        file_contents: HTMLString::from(
                            ModuleIndex {
                                version_opt: None,
                                module_info,
                                module_docs: DocLinks {
                                    style: DocStyle::ModuleIndex,
                                    links: doc_links.to_owned(),
                                },
                            }
                            .render()?,
                        ),
                    })
                }
            }
        }
        // AllDocIndex
        rendered_docs.0.push(RenderedDocument {
            module_info: root_module.clone(),
            html_filename: ALL_DOC_FILENAME.to_string(),
            file_contents: HTMLString::from(
                AllDocIndex {
                    project_name: root_module,
                    all_docs,
                }
                .render()?,
            ),
        });

        Ok(rendered_docs)
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
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let ItemHeader {
            module_info,
            friendly_name,
            item_name,
        } = self;

        let favicon = module_info.to_html_shorthand_path_string("assets/sway-logo.svg");
        let normalize = module_info.to_html_shorthand_path_string("assets/normalize.css");
        let swaydoc = module_info.to_html_shorthand_path_string("assets/swaydoc.css");
        let ayu = module_info.to_html_shorthand_path_string("assets/ayu.css");

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
                // TODO: Add links for fonts
            }
        })
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
            version_opt: None,
            style: DocStyle::Item,
            module_info: self.module_info.clone(),
            href_path: INDEX_FILENAME.to_owned(),
            nav: self.item_context.to_doclinks(),
        }
    }
}
impl Renderable for ItemBody {
    /// HTML body component
    fn render(self) -> Result<Box<dyn RenderBox>> {
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
        let friendly_name = ty_decl.friendly_type_name();
        let sidebar = sidebar.render()?;
        let item_context = (item_context.context.is_some())
            .then(|| -> Result<Box<dyn RenderBox>> { item_context.render() });

        Ok(box_html! {
            body(class=format!("swaydoc {decl_ty}")) {
                : sidebar;
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
                                        : format!("{friendly_name} ");
                                        // TODO: add qualified path anchors
                                        a(class=&decl_ty, href=IDENTITY) {
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
                            @ if item_context.is_some() {
                                : item_context.unwrap();
                            }
                        }
                    }
                }
            }
        })
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
impl ItemContext {
    fn to_doclinks(&self) -> DocLinks {
        let mut links: BTreeMap<BlockTitle, Vec<DocLink>> = BTreeMap::new();
        if let Some(context) = &self.context {
            match context {
                ContextType::StructFields(fields) => {
                    let doc_links = fields
                        .iter()
                        .map(|field| DocLink {
                            name: field.name.as_str().to_string(),
                            module_info: ModuleInfo::from_vec(vec![]),
                            html_filename: format!(
                                "{}structfield.{}",
                                IDENTITY,
                                field.name.as_str()
                            ),
                            preview_opt: None,
                        })
                        .collect();
                    links.insert(BlockTitle::Fields, doc_links);
                }
                ContextType::StorageFields(fields) => {
                    let doc_links = fields
                        .iter()
                        .map(|field| DocLink {
                            name: field.name.as_str().to_string(),
                            module_info: ModuleInfo::from_vec(vec![]),
                            html_filename: format!(
                                "{}storagefield.{}",
                                IDENTITY,
                                field.name.as_str()
                            ),
                            preview_opt: None,
                        })
                        .collect();
                    links.insert(BlockTitle::Fields, doc_links);
                }
                ContextType::EnumVariants(variants) => {
                    let doc_links = variants
                        .iter()
                        .map(|variant| DocLink {
                            name: variant.name.as_str().to_string(),
                            module_info: ModuleInfo::from_vec(vec![]),
                            html_filename: format!("{}variant.{}", IDENTITY, variant.name.as_str()),
                            preview_opt: None,
                        })
                        .collect();
                    links.insert(BlockTitle::Variants, doc_links);
                }
                ContextType::RequiredMethods(methods) => {
                    let doc_links = methods
                        .iter()
                        .map(|method| DocLink {
                            name: method.name.as_str().to_string(),
                            module_info: ModuleInfo::from_vec(vec![]),
                            html_filename: format!(
                                "{}structfield.{}",
                                IDENTITY,
                                method.name.as_str()
                            ),
                            preview_opt: None,
                        })
                        .collect();
                    links.insert(BlockTitle::RequiredMethods, doc_links);
                }
            }
        }
        DocLinks {
            style: DocStyle::Item,
            links,
        }
    }
}
impl Renderable for ItemContext {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        match self.context.unwrap() {
            ContextType::StructFields(fields) => Ok(context_section(fields, BlockTitle::Fields)?),
            ContextType::StorageFields(fields) => Ok(context_section(fields, BlockTitle::Fields)?),
            ContextType::EnumVariants(variants) => {
                Ok(context_section(variants, BlockTitle::Variants)?)
            }
            ContextType::RequiredMethods(methods) => {
                Ok(context_section(methods, BlockTitle::RequiredMethods)?)
            }
        }
    }
}
/// Dynamically creates the context section of an item.
fn context_section<'title, S: Renderable + 'static>(
    list: Vec<S>,
    title: BlockTitle,
) -> Result<Box<dyn RenderBox + 'title>> {
    let lct = title.html_title_string();
    let mut rendered_list: Vec<_> = Vec::new();
    for item in list {
        rendered_list.push(item.render()?)
    }
    Ok(box_html! {
        h2(id=&lct, class=format!("{} small-section-header", &lct)) {
            : title.as_str();
            a(class="anchor", href=format!("{IDENTITY}{lct}"));
        }
        @ for item in rendered_list {
            // TODO: Check for visibility of the field itself
            : item;
        }
    })
}
impl Renderable for TyStructField {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let struct_field_id = format!("structfield.{}", self.name.as_str());
        Ok(box_html! {
            span(id=&struct_field_id, class="structfield small-section-header") {
                a(class="anchor field", href=format!("{IDENTITY}{struct_field_id}"));
                code {
                    : format!("{}: ", self.name.as_str());
                    // TODO: Add links to types based on visibility
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(self.attributes.to_html_string());
                }
            }
        })
    }
}
impl Renderable for TyStorageField {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let storage_field_id = format!("storagefield.{}", self.name.as_str());
        Ok(box_html! {
            span(id=&storage_field_id, class="storagefield small-section-header") {
                a(class="anchor field", href=format!("{IDENTITY}{storage_field_id}"));
                code {
                    : format!("{}: ", self.name.as_str());
                    // TODO: Add links to types based on visibility
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(self.attributes.to_html_string());
                }
            }
        })
    }
}

impl Renderable for TyEnumVariant {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let enum_variant_id = format!("variant.{}", self.name.as_str());
        Ok(box_html! {
            h3(id=&enum_variant_id, class="variant small-section-header") {
                a(class="anchor field", href=format!("{IDENTITY}{enum_variant_id}"));
                code {
                    : format!("{}: ", self.name.as_str());
                    : self.type_span.as_str();
                }
            }
            @ if !self.attributes.is_empty() {
                div(class="docblock") {
                    : Raw(self.attributes.to_html_string());
                }
            }
        })
    }
}
impl Renderable for TyTraitFn {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        // there is likely a better way we can do this while simultaneously storing the
        // string slices we need like "&mut "
        let mut fn_sig = format!("fn {}(", self.name.as_str());
        for param in &self.parameters {
            let mut param_str = String::new();
            if param.is_reference {
                write!(param_str, "&")?;
            }
            if param.is_mutable {
                write!(param_str, "mut ")?;
            }
            if param.is_self() {
                write!(param_str, "self,")?;
            } else {
                write!(
                    fn_sig,
                    "{} {},",
                    param.name.as_str(),
                    param.type_argument.span.as_str()
                )?;
            }
        }
        write!(fn_sig, ") -> {}", self.return_type_span.as_str())?;
        let multiline = fn_sig.chars().count() >= 60;

        let method_id = format!("tymethod.{}", self.name.as_str());
        Ok(box_html! {
            div(class="methods") {
                div(id=&method_id, class="method has-srclink") {
                    h4(class="code-header") {
                        : "fn ";
                        a(class="fnname", href=format!("{IDENTITY}{method_id}")) {
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
                                    : param.type_argument.span.as_str();
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
                                    : "self"
                                } else {
                                    : param.name.as_str();
                                    : ": ";
                                    : param.type_argument.span.as_str();
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
        })
    }
}
/// Used for creating links between docs.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct DocLink {
    pub(crate) name: String,
    pub(crate) module_info: ModuleInfo,
    pub(crate) html_filename: String,
    pub(crate) preview_opt: Option<String>,
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct DocLinks {
    style: DocStyle,
    /// The title and link info for each doc item.
    links: BTreeMap<BlockTitle, Vec<DocLink>>,
}
impl Renderable for DocLinks {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let doc_links = match self.style {
            DocStyle::AllDoc => box_html! {
                @ for (title, list_items) in self.links {
                    @ if !list_items.is_empty() {
                        h3(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(href=item.module_info.to_file_path_string(&item.html_filename, item.module_info.project_name())) {
                                            : item.module_info.to_path_literal_string(
                                                &item.name,
                                                item.module_info.project_name()
                                            );
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.unwrap());
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
            DocStyle::ProjectIndex => box_html! {
                @ for (title, list_items) in self.links {
                    @ if !list_items.is_empty() {
                        h3(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(href=item.module_info.to_file_path_string(&item.html_filename, item.module_info.project_name())) {
                                            @ if title == BlockTitle::Modules {
                                                : item.name;
                                            } else {
                                                : item.module_info.to_path_literal_string(
                                                    &item.name,
                                                    item.module_info.project_name()
                                                );
                                            }
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.unwrap());
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
            _ => box_html! {
                @ for (title, list_items) in self.links {
                    @ if !list_items.is_empty() {
                        h3(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(href=item.module_info.to_file_path_string(&item.html_filename, item.module_info.location())) {
                                            : item.module_info.to_path_literal_string(
                                                &item.name,
                                                item.module_info.location()
                                            );
                                        }
                                    }
                                    @ if item.preview_opt.is_some() {
                                        div(class="item-right docblock-short") {
                                            : Raw(item.preview_opt.unwrap());
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
            : Raw(doc_links);
        })
    }
}
/// Represents all of the possible titles
/// belonging to an index or sidebar.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum BlockTitle {
    Modules,
    Structs,
    Enums,
    Traits,
    Abi,
    ContractStorage,
    Constants,
    Functions,
    Fields,
    Variants,
    RequiredMethods,
}
impl BlockTitle {
    fn as_str(&self) -> &str {
        match self {
            Self::Modules => "Modules",
            Self::Structs => "Structs",
            Self::Enums => "Enums",
            Self::Traits => "Traits",
            Self::Abi => "Abi",
            Self::ContractStorage => "Contract Storage",
            Self::Constants => "Constants",
            Self::Functions => "Functions",
            Self::Fields => "Fields",
            Self::Variants => "Variants",
            Self::RequiredMethods => "Required Methods",
        }
    }
    fn item_title_str(&self) -> &str {
        match self {
            Self::Modules => "Module",
            Self::Structs => "Struct",
            Self::Enums => "Enum",
            Self::Traits => "Trait",
            Self::Abi => "Abi",
            Self::ContractStorage => "Contract Storage",
            Self::Constants => "Constant",
            Self::Functions => "Function",
            Self::Fields => "Fields",
            Self::Variants => "Variants",
            Self::RequiredMethods => "Required Methods",
        }
    }
    fn html_title_string(&self) -> String {
        if self.as_str().contains(' ') {
            self.as_str()
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("-")
        } else {
            self.as_str().to_lowercase()
        }
    }
}
/// Project level, all items belonging to a project
#[derive(Clone)]
struct AllDocIndex {
    /// A [ModuleInfo] with only the project name.
    project_name: ModuleInfo,
    /// All doc items.
    all_docs: DocLinks,
}
impl SidebarNav for AllDocIndex {
    fn sidebar(&self) -> Sidebar {
        Sidebar {
            version_opt: None,
            style: DocStyle::AllDoc,
            module_info: self.project_name.clone(),
            href_path: INDEX_FILENAME.to_owned(),
            nav: self.all_docs.clone(),
        }
    }
}
impl Renderable for AllDocIndex {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let doc_links = self.all_docs.clone().render()?;
        let sidebar = self.sidebar().render()?;
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
            }
            body(class="swaydoc mod") {
                : sidebar;
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
                            : doc_links;
                        }
                    }
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
            true => DocStyle::ProjectIndex,
            false => DocStyle::ModuleIndex,
        };
        Sidebar {
            version_opt: self.version_opt.clone(),
            style,
            module_info: self.module_info.clone(),
            href_path: INDEX_FILENAME.to_owned(),
            nav: self.module_docs.clone(),
        }
    }
}
impl Renderable for ModuleIndex {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let doc_links = self.module_docs.clone().render()?;
        let sidebar = self.sidebar().render()?;
        let title_prefix = match self.module_docs.style {
            DocStyle::ProjectIndex => "Project ",
            DocStyle::ModuleIndex => "Module ",
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
            }
            body(class="swaydoc mod") {
                : sidebar;
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
                            div(class="main-heading") {
                                h1(class="fqn") {
                                    span(class="in-band") {
                                        : title_prefix;
                                        a(class="module", href=IDENTITY) {
                                            : self.module_info.location();
                                        }
                                    }
                                }
                            }
                            : doc_links;
                        }
                    }
                }
            }
        })
    }
}

trait SidebarNav {
    /// Create sidebar component.
    fn sidebar(&self) -> Sidebar;
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
enum DocStyle {
    AllDoc,
    ProjectIndex,
    ModuleIndex,
    Item,
}
/// Sidebar component for quick navigation.
struct Sidebar {
    version_opt: Option<String>,
    style: DocStyle,
    module_info: ModuleInfo,
    /// the path to the current module
    href_path: String,
    /// support for page navigation
    nav: DocLinks,
}
impl Renderable for Sidebar {
    fn render(self) -> Result<Box<dyn RenderBox>> {
        let path_to_logo = self
            .module_info
            .to_html_shorthand_path_string("assets/sway-logo.svg");
        let location_with_prefix = match &self.style {
            DocStyle::AllDoc | DocStyle::ProjectIndex => {
                format!("Project {}", self.module_info.location())
            }
            DocStyle::ModuleIndex | DocStyle::Item => format!(
                "{} {}",
                BlockTitle::Modules.item_title_str(),
                self.module_info.location()
            ),
        };
        let (logo_path_to_parent, path_to_parent_or_self) = match &self.style {
            DocStyle::AllDoc | DocStyle::Item => (self.href_path.clone(), self.href_path.clone()),
            DocStyle::ProjectIndex => (IDENTITY.to_owned(), IDENTITY.to_owned()),
            DocStyle::ModuleIndex => (format!("../{INDEX_FILENAME}"), IDENTITY.to_owned()),
        };
        // Unfortunately, match arms that return a closure, even if they are the same
        // type, are incompatible. The work around is to return a String instead,
        // and render it from Raw in the final output.
        let styled_content = match &self.style {
            DocStyle::ProjectIndex => {
                let nav_links = self.nav.links;
                let version = match self.version_opt {
                    Some(ref v) => v.as_str(),
                    None => "0.0.0",
                };
                box_html! {
                    div(class="sidebar-elems") {
                        div(class="block") {
                            ul {
                                li(class="version") {
                                    : format!("Version {version}");
                                }
                                li {
                                    a(id="all-types", href=ALL_DOC_FILENAME) {
                                        : "All Items";
                                    }
                                }
                            }
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
                    }
                }
                .into_string()
                .unwrap()
            }
            _ => box_html! {
                div(class="sidebar-elems") {
                    section {
                        div(class="block") {
                            ul {
                                @ for (title, _) in self.nav.links {
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
            .unwrap(),
        };
        Ok(box_html! {
            nav(class="sidebar") {
                a(class="sidebar-logo", href=&logo_path_to_parent) {
                    div(class="logo-container") {
                        img(class="sway-logo", src=path_to_logo, alt="logo");
                    }
                }
                h2(class="location") {
                    a(href=path_to_parent_or_self) { : location_with_prefix; }
                }
                : Raw(styled_content);
            }
        })
    }
}
pub(crate) trait DocStrings {
    fn to_html_string(&self) -> String;
    fn to_raw_string(&self) -> String;
}
/// Creates an HTML String from an [AttributesMap]
impl DocStrings for AttributesMap {
    fn to_html_string(&self) -> String {
        let docs = self.to_raw_string();

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
    fn to_raw_string(&self) -> String {
        let attributes = self.get(&AttributeKind::DocComment);
        let mut docs = String::new();

        if let Some(vec_attrs) = attributes {
            for ident in vec_attrs.iter().flat_map(|attribute| &attribute.args) {
                writeln!(docs, "{}", ident.as_str())
                    .expect("problem appending `ident.as_str()` to `docs` with `writeln` macro.");
            }
        }
        docs
    }
}
/// Takes a formatted String fn and returns only the function signature.
pub(crate) fn trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}

/// Checks if some raw html (rendered from markdown) contains a header.
/// If it does, it splits at the header and returns the slice that preceeded it.
pub(crate) fn split_at_markdown_header(raw_html: &str) -> &str {
    const H1: &str = "<h1>";
    const H2: &str = "<h2>";
    const H3: &str = "<h3>";
    const H4: &str = "<h4>";
    const H5: &str = "<h5>";
    if raw_html.contains(H1) {
        let v: Vec<_> = raw_html.split(H1).collect();
        v.first().expect("expected a non-empty str")
    } else if raw_html.contains(H2) {
        let v: Vec<_> = raw_html.split(H2).collect();
        v.first().expect("expected a non-empty str")
    } else if raw_html.contains(H3) {
        let v: Vec<_> = raw_html.split(H3).collect();
        v.first().expect("expected a non-empty str")
    } else if raw_html.contains(H4) {
        let v: Vec<_> = raw_html.split(H4).collect();
        v.first().expect("expected a non-empty str")
    } else if raw_html.contains(H5) {
        let v: Vec<_> = raw_html.split(H5).collect();
        v.first().expect("expected a non-empty str")
    } else {
        raw_html
    }
}
