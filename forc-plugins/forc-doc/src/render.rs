use crate::{
    doc::{Documentation, ModuleInfo, ModulePrefix},
    RenderPlan,
};
use anyhow::{anyhow, Result};
use comrak::{markdown_to_html, ComrakOptions};
use horrorshow::{box_html, helper::doctype, html, prelude::*, Raw};
use std::{collections::BTreeMap, fmt::Write};
use sway_core::{
    language::ty::{
        TyDeclaration::{self, *},
        TyEnumVariant, TyProgramKind, TyStorageField, TyStructField, TyTraitFn,
    },
    transform::{AttributeKind, AttributesMap},
    AbiName, TypeInfo,
};
use sway_lsp::utils::markdown::format_docs;
use sway_types::{BaseIdent, Spanned};

pub(crate) const ALL_DOC_FILENAME: &str = "all.html";
pub(crate) const INDEX_FILENAME: &str = "index.html";
pub(crate) const IDENTITY: &str = "#";

pub(crate) trait Renderable {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>>;
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
    pub fn from(
        raw: Documentation,
        render_plan: RenderPlan,
        root_attributes: Option<AttributesMap>,
        program_kind: TyProgramKind,
        forc_version: Option<String>,
    ) -> Result<RenderedDocumentation> {
        let mut rendered_docs: RenderedDocumentation = Default::default();
        let root_module = match raw.first() {
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
        let mut module_map: BTreeMap<ModulePrefix, BTreeMap<BlockTitle, Vec<DocLink>>> =
            BTreeMap::new();
        for doc in raw {
            rendered_docs.0.push(RenderedDocument {
                module_info: doc.module_info.clone(),
                html_filename: doc.html_filename(),
                file_contents: HTMLString::from(doc.clone().render(render_plan.clone())?),
            });
            // Here we gather all of the `doc_links` based on which module they belong to.
            let location = doc.module_info.location().to_string();
            match module_map.get_mut(&location) {
                Some(doc_links) => {
                    match doc.item_body.ty_decl {
                        StructDeclaration { .. } => match doc_links.get_mut(&BlockTitle::Structs) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Structs, vec![doc.link()]);
                            }
                        },
                        EnumDeclaration { .. } => match doc_links.get_mut(&BlockTitle::Enums) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Enums, vec![doc.link()]);
                            }
                        },
                        TraitDeclaration { .. } => match doc_links.get_mut(&BlockTitle::Traits) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Traits, vec![doc.link()]);
                            }
                        },
                        AbiDeclaration { .. } => match doc_links.get_mut(&BlockTitle::Abi) {
                            Some(links) => links.push(doc.link()),
                            None => {
                                doc_links.insert(BlockTitle::Abi, vec![doc.link()]);
                            }
                        },
                        StorageDeclaration { .. } => {
                            match doc_links.get_mut(&BlockTitle::ContractStorage) {
                                Some(links) => links.push(doc.link()),
                                None => {
                                    doc_links.insert(BlockTitle::ContractStorage, vec![doc.link()]);
                                }
                            }
                        }
                        FunctionDeclaration { .. } => {
                            match doc_links.get_mut(&BlockTitle::Functions) {
                                Some(links) => links.push(doc.link()),
                                None => {
                                    doc_links.insert(BlockTitle::Functions, vec![doc.link()]);
                                }
                            }
                        }
                        ConstantDeclaration { .. } => {
                            match doc_links.get_mut(&BlockTitle::Constants) {
                                Some(links) => links.push(doc.link()),
                                None => {
                                    doc_links.insert(BlockTitle::Constants, vec![doc.link()]);
                                }
                            }
                        }
                        _ => {} // TODO: ImplTraitDeclaration
                    }
                }
                None => {
                    let mut doc_links: BTreeMap<BlockTitle, Vec<DocLink>> = BTreeMap::new();
                    match doc.item_body.ty_decl {
                        StructDeclaration { .. } => {
                            doc_links.insert(BlockTitle::Structs, vec![doc.link()]);
                        }
                        EnumDeclaration { .. } => {
                            doc_links.insert(BlockTitle::Enums, vec![doc.link()]);
                        }
                        TraitDeclaration { .. } => {
                            doc_links.insert(BlockTitle::Traits, vec![doc.link()]);
                        }
                        AbiDeclaration { .. } => {
                            doc_links.insert(BlockTitle::Abi, vec![doc.link()]);
                        }
                        StorageDeclaration { .. } => {
                            doc_links.insert(BlockTitle::ContractStorage, vec![doc.link()]);
                        }
                        FunctionDeclaration { .. } => {
                            doc_links.insert(BlockTitle::Functions, vec![doc.link()]);
                        }
                        ConstantDeclaration { .. } => {
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
                    preview_opt: doc.module_info.preview_opt(),
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
                StructDeclaration { .. } => match all_docs.links.get_mut(&BlockTitle::Structs) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Structs, vec![doc.link()]);
                    }
                },
                EnumDeclaration { .. } => match all_docs.links.get_mut(&BlockTitle::Enums) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Enums, vec![doc.link()]);
                    }
                },
                TraitDeclaration { .. } => match all_docs.links.get_mut(&BlockTitle::Traits) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Traits, vec![doc.link()]);
                    }
                },
                AbiDeclaration { .. } => match all_docs.links.get_mut(&BlockTitle::Abi) {
                    Some(links) => links.push(doc.link()),
                    None => {
                        all_docs.links.insert(BlockTitle::Abi, vec![doc.link()]);
                    }
                },
                StorageDeclaration { .. } => {
                    match all_docs.links.get_mut(&BlockTitle::ContractStorage) {
                        Some(links) => links.push(doc.link()),
                        None => {
                            all_docs
                                .links
                                .insert(BlockTitle::ContractStorage, vec![doc.link()]);
                        }
                    }
                }
                FunctionDeclaration { .. } => {
                    match all_docs.links.get_mut(&BlockTitle::Functions) {
                        Some(links) => links.push(doc.link()),
                        None => {
                            all_docs
                                .links
                                .insert(BlockTitle::Functions, vec![doc.link()]);
                        }
                    }
                }
                ConstantDeclaration { .. } => {
                    match all_docs.links.get_mut(&BlockTitle::Constants) {
                        Some(links) => links.push(doc.link()),
                        None => {
                            all_docs
                                .links
                                .insert(BlockTitle::Constants, vec![doc.link()]);
                        }
                    }
                }
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
                            style: DocStyle::ProjectIndex(program_kind.as_title_str().to_string()),
                            links: doc_links.to_owned(),
                        },
                    }
                    .render(render_plan.clone())?,
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
                            .render(render_plan.clone())?,
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
                .render(render_plan)?,
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
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let ItemHeader {
            module_info,
            friendly_name,
            item_name,
        } = self;

        let favicon = module_info.to_html_shorthand_path_string("assets/sway-logo.svg");
        let normalize = module_info.to_html_shorthand_path_string("assets/normalize.css");
        let swaydoc = module_info.to_html_shorthand_path_string("assets/swaydoc.css");
        let ayu = module_info.to_html_shorthand_path_string("assets/ayu.css");
        let ayu_hjs = module_info.to_html_shorthand_path_string("assets/ayu.min.css");

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
                link(rel="stylesheet", href=ayu_hjs);
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
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let sidebar = self.sidebar();
        let ItemBody {
            module_info,
            ty_decl,
            item_name,
            code_str,
            attrs_opt,
            item_context,
        } = self;

        let decl_ty = ty_decl.doc_name();
        let friendly_name = ty_decl.friendly_type_name();
        let sidebar = sidebar.render(render_plan.clone())?;
        let item_context = (item_context.context_opt.is_some())
            .then(|| -> Result<Box<dyn RenderBox>> { item_context.render(render_plan) });
        let sway_hjs = module_info.to_html_shorthand_path_string("assets/highlight.js");

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
                script(src=sway_hjs);
                script {
                    : "hljs.highlightAll();";
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
impl ContextType {
    fn as_block_title(&self) -> BlockTitle {
        match self {
            ContextType::StructFields(_) => BlockTitle::Fields,
            ContextType::StorageFields(_) => BlockTitle::Fields,
            ContextType::EnumVariants(_) => BlockTitle::Variants,
            ContextType::RequiredMethods(_) => BlockTitle::RequiredMethods,
        }
    }
}
/// The actual context of the item displayed by [ItemContext].
/// This uses [ContextType] to determine how to represent the context of an item.
///
/// Example:
/// ```sw
/// struct Foo {}
/// trait Foo {
///     fn foo() -> Foo;
/// }
/// ```
/// Becomes:
/// ```rust
/// Context {
///     module_info: ModuleInfo, /* cloned from item origin to create links */
///     context_type: ContextType::RequiredMethods(Vec<TyTraitFn>), /* trait fn foo() stored here */
/// }
/// ```
#[derive(Clone)]
pub(crate) struct Context {
    module_info: ModuleInfo,
    context_type: ContextType,
}
impl Context {
    pub(crate) fn new(module_info: ModuleInfo, context_type: ContextType) -> Self {
        Self {
            module_info,
            context_type,
        }
    }
}
impl Renderable for Context {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let mut rendered_list: Vec<String> = Vec::new();
        match self.context_type {
            ContextType::StructFields(fields) => {
                for field in fields {
                    let struct_field_id = format!("structfield.{}", field.name.as_str());
                    let type_anchor = render_type_anchor(
                        render_plan.type_engine.get(field.type_argument.type_id),
                        &render_plan,
                        &self.module_info,
                    );
                    rendered_list.push(box_html! {
                        span(id=&struct_field_id, class="structfield small-section-header") {
                            a(class="anchor field", href=format!("{IDENTITY}{struct_field_id}"));
                            code {
                                : format!("{}: ", field.name.as_str());
                                @ if let Ok(type_anchor) = type_anchor {
                                    : type_anchor;
                                } else {
                                    : field.type_argument.span.as_str();
                                }
                            }
                        }
                        @ if !field.attributes.is_empty() {
                            div(class="docblock") {
                                : Raw(field.attributes.to_html_string());
                            }
                        }
                    }.into_string()?);
                }
            }
            ContextType::StorageFields(fields) => {
                for field in fields {
                    let storage_field_id = format!("storagefield.{}", field.name.as_str());
                    let type_anchor = render_type_anchor(
                        render_plan.type_engine.get(field.type_argument.type_id),
                        &render_plan,
                        &self.module_info,
                    );
                    rendered_list.push(box_html! {
                        span(id=&storage_field_id, class="storagefield small-section-header") {
                            a(class="anchor field", href=format!("{IDENTITY}{storage_field_id}"));
                            code {
                                : format!("{}: ", field.name.as_str());
                                @ if let Ok(type_anchor) = type_anchor {
                                    : type_anchor;
                                } else {
                                    : field.type_argument.span.as_str();
                                }
                            }
                        }
                        @ if !field.attributes.is_empty() {
                            div(class="docblock") {
                                : Raw(field.attributes.to_html_string());
                            }
                        }
                    }.into_string()?);
                }
            }
            ContextType::EnumVariants(variants) => {
                for variant in variants {
                    let enum_variant_id = format!("variant.{}", variant.name.as_str());
                    let type_anchor = render_type_anchor(
                        render_plan.type_engine.get(variant.type_argument.type_id),
                        &render_plan,
                        &self.module_info,
                    );
                    rendered_list.push(box_html! {
                        h3(id=&enum_variant_id, class="variant small-section-header") {
                            a(class="anchor field", href=format!("{IDENTITY}{enum_variant_id}"));
                            code {
                                : format!("{}: ", variant.name.as_str());
                                @ if let Ok(type_anchor) = type_anchor {
                                    : type_anchor;
                                } else {
                                    : variant.type_argument.span.as_str();
                                }
                            }
                        }
                        @ if !variant.attributes.is_empty() {
                            div(class="docblock") {
                                : Raw(variant.attributes.to_html_string());
                            }
                        }
                    }.into_string()?);
                }
            }
            ContextType::RequiredMethods(methods) => {
                for method in methods {
                    // there is likely a better way we can do this while simultaneously storing the
                    // string slices we need like "&mut "
                    let mut fn_sig = format!("fn {}(", method.name.as_str());
                    for param in &method.parameters {
                        let mut param_str = String::new();
                        if param.is_reference {
                            write!(param_str, "ref ")?;
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
                    write!(fn_sig, ") -> {}", method.return_type_span.as_str())?;
                    let multiline = fn_sig.chars().count() >= 60;

                    let method_id = format!("tymethod.{}", method.name.as_str());
                    rendered_list.push(box_html! {
                        div(class="methods") {
                            div(id=&method_id, class="method has-srclink") {
                                h4(class="code-header") {
                                    : "fn ";
                                    a(class="fnname", href=format!("{IDENTITY}{method_id}")) {
                                        : method.name.as_str();
                                    }
                                    : "(";
                                    @ if multiline {
                                        @ for param in &method.parameters {
                                            br;
                                            : "    ";
                                            @ if param.is_reference {
                                                : "ref";
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
                                        @ for param in &method.parameters {
                                            @ if param.is_reference {
                                                : "ref";
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
                                                != method.parameters.last()
                                                .expect("no last element in trait method parameters list")
                                                .name.as_str() {
                                                : ", ";
                                            }
                                        }
                                        : ") -> ";
                                    }
                                    : method.return_type_span.as_str();
                                }
                            }
                        }
                    }.into_string()?);
                }
            }
        };
        Ok(box_html! {
            @ for item in rendered_list {
                : Raw(item);
            }
        })
    }
}
#[derive(Clone)]
/// The context section of an item that appears in the page [ItemBody].
pub(crate) struct ItemContext {
    pub(crate) context_opt: Option<Context>,
    // TODO: All other Implementation types, eg
    // implementations on foreign types, method implementations, etc.
}
impl ItemContext {
    fn to_doclinks(&self) -> DocLinks {
        let mut links: BTreeMap<BlockTitle, Vec<DocLink>> = BTreeMap::new();
        if let Some(context) = &self.context_opt {
            match context.context_type.clone() {
                ContextType::StructFields(fields) => {
                    let doc_links = fields
                        .iter()
                        .map(|field| DocLink {
                            name: field.name.as_str().to_string(),
                            module_info: ModuleInfo::from_ty_module(vec![], None),
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
                            module_info: ModuleInfo::from_ty_module(vec![], None),
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
                            module_info: ModuleInfo::from_ty_module(vec![], None),
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
                            module_info: ModuleInfo::from_ty_module(vec![], None),
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
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let (title, rendered_list) = match self.context_opt {
            Some(context) => {
                let title = context.context_type.as_block_title();
                let rendered_list = context.render(render_plan)?;
                Ok((title, rendered_list))
            }
            None => Err(anyhow!(
                "Safeguard against render call on empty context failed."
            )),
        }?;
        let lct = title.html_title_string();
        Ok(box_html! {
            h2(id=&lct, class=format!("{} small-section-header", &lct)) {
                : title.as_str();
                a(class="anchor", href=format!("{IDENTITY}{lct}"));
            }
            : rendered_list;
        })
    }
}
/// Handles types & nested types that should have links
/// eg. (`[]` represent types with links).
///
/// ```sway
/// struct Foo {
///     foo: ([Foo], (u32, [Foo], ([Foo], [Foo])))
/// }
/// ```
//
// TODO: Add checks for multiline types
fn render_type_anchor(
    type_info: TypeInfo,
    render_plan: &RenderPlan,
    current_module_info: &ModuleInfo,
) -> Result<Box<dyn RenderBox>> {
    match type_info {
        TypeInfo::Array(ty_arg, len) => {
            let inner = render_type_anchor(
                render_plan.type_engine.get(ty_arg.type_id),
                render_plan,
                current_module_info,
            )?;
            Ok(box_html! {
                : "[";
                : inner;
                : format!("; {}]", len.val());
            })
        }
        TypeInfo::Tuple(ty_args) => {
            let mut rendered_args: Vec<_> = Vec::new();
            for ty_arg in ty_args {
                rendered_args.push(render_type_anchor(
                    render_plan.type_engine.get(ty_arg.type_id),
                    render_plan,
                    current_module_info,
                )?)
            }
            Ok(box_html! {
                : "(";
                @ for arg in rendered_args {
                    : arg;
                }
                : ")";
            })
        }
        TypeInfo::Enum(decl_ref) => {
            let enum_decl = render_plan.decl_engine.get_enum(&decl_ref);
            if !render_plan.document_private_items && enum_decl.visibility.is_private() {
                Ok(box_html! {
                    : decl_ref.name.as_str();
                })
            } else {
                let module_info = ModuleInfo::from_call_path(enum_decl.call_path);
                let file_name = format!("enum.{}.html", decl_ref.name.as_str());
                let href = module_info.file_path_from_location(&file_name, current_module_info)?;
                Ok(box_html! {
                    a(class="enum", href=href) {
                        : decl_ref.name.as_str();
                    }
                })
            }
        }
        TypeInfo::Struct(decl_ref) => {
            let struct_decl = render_plan.decl_engine.get_struct(&decl_ref);
            if !render_plan.document_private_items && struct_decl.visibility.is_private() {
                Ok(box_html! {
                    : decl_ref.name.as_str();
                })
            } else {
                let module_info = ModuleInfo::from_call_path(struct_decl.call_path);
                let file_name = format!("struct.{}.html", decl_ref.name.as_str());
                let href = module_info.file_path_from_location(&file_name, current_module_info)?;
                Ok(box_html! {
                    a(class="struct", href=href) {
                        : decl_ref.name.as_str();
                    }
                })
            }
        }
        TypeInfo::UnknownGeneric { name, .. } => Ok(box_html! {
            : name.as_str();
        }),
        TypeInfo::Str(len) => Ok(box_html! {
            : len.span().as_str();
        }),
        TypeInfo::UnsignedInteger(int_bits) => {
            use sway_types::integer_bits::IntegerBits;
            let uint = match int_bits {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            };
            Ok(box_html! {
                : uint;
            })
        }
        TypeInfo::Boolean => Ok(box_html! {
            : "bool";
        }),
        TypeInfo::ContractCaller { abi_name, .. } => {
            // TODO: determine whether we should give a link to this
            if let AbiName::Known(name) = abi_name {
                Ok(box_html! {
                    : name.suffix.as_str();
                })
            } else {
                Err(anyhow!("Deferred AbiName is unhandled"))
            }
        }
        TypeInfo::Custom { call_path, .. } => Ok(box_html! {
            : call_path.suffix.as_str();
        }),
        TypeInfo::SelfType => Ok(box_html! {
            : "Self";
        }),
        TypeInfo::B256 => Ok(box_html! {
            : "b256";
        }),
        _ => Err(anyhow!("Undetermined or unusable TypeInfo")),
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
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let doc_links = match self.style {
            DocStyle::AllDoc(_) => box_html! {
                @ for (title, list_items) in self.links {
                    @ if !list_items.is_empty() {
                        h3(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.project_name())) {
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
            DocStyle::ProjectIndex(_) => box_html! {
                @ for (title, list_items) in self.links {
                    @ if !list_items.is_empty() {
                        h3(id=format!("{}", title.html_title_string())) { : title.as_str(); }
                        div(class="item-table") {
                            @ for item in list_items {
                                div(class="item-row") {
                                    div(class=format!("item-left {}-item", title.item_title_str())) {
                                        a(href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.project_name())) {
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
                                        a(href=item.module_info.file_path_at_location(&item.html_filename, item.module_info.location())) {
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
            style: self.all_docs.style.clone(),
            module_info: self.project_name.clone(),
            href_path: INDEX_FILENAME.to_owned(),
            nav: self.all_docs.clone(),
        }
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

trait SidebarNav {
    /// Create sidebar component.
    fn sidebar(&self) -> Sidebar;
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
enum DocStyle {
    AllDoc(String),
    ProjectIndex(String),
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
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let path_to_logo = self
            .module_info
            .to_html_shorthand_path_string("assets/sway-logo.svg");
        let location_with_prefix = match &self.style {
            DocStyle::AllDoc(project_kind) | DocStyle::ProjectIndex(project_kind) => {
                format!("{project_kind} {}", self.module_info.location())
            }
            DocStyle::ModuleIndex | DocStyle::Item => format!(
                "{} {}",
                BlockTitle::Modules.item_title_str(),
                self.module_info.location()
            ),
        };
        let (logo_path_to_parent, path_to_parent_or_self) = match &self.style {
            DocStyle::AllDoc(_) | DocStyle::Item => {
                (self.href_path.clone(), self.href_path.clone())
            }
            DocStyle::ProjectIndex(_) => (IDENTITY.to_owned(), IDENTITY.to_owned()),
            DocStyle::ModuleIndex => (format!("../{INDEX_FILENAME}"), IDENTITY.to_owned()),
        };
        // Unfortunately, match arms that return a closure, even if they are the same
        // type, are incompatible. The work around is to return a String instead,
        // and render it from Raw in the final output.
        let styled_content = match &self.style {
            DocStyle::ProjectIndex(_) => {
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
