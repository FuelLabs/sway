//! Manages how the context of Sway types are rendered on corresponding item pages.
use crate::{
    doc::module::ModuleInfo,
    render::{
        item::type_anchor::render_type_anchor,
        link::{DocLink, DocLinks},
        title::BlockTitle,
        title::DocBlock,
        util::format::docstring::DocStrings,
        DocStyle, Renderable, IDENTITY,
    },
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, Raw, RenderBox, Template};
use std::{collections::BTreeMap, fmt::Write};
use sway_core::language::ty::{
    TyConstantDecl, TyEnumVariant, TyFunctionDecl, TyImplSelfOrTrait, TyStorageField,
    TyStructField, TyTraitFn, TyTraitItem, TyTraitType,
};
use sway_types::Spanned;

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
/// ```ignore
/// Context {
///     module_info: ModuleInfo, /* cloned from item origin to create links */
///     context_type: ContextType::RequiredMethods(Vec<TyTraitFn>), /* trait fn foo() stored here */
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Context {
    module_info: ModuleInfo,
    context_type: ContextType,
}

impl Context {
    pub fn new(module_info: ModuleInfo, context_type: ContextType) -> Self {
        Self {
            module_info,
            context_type,
        }
    }
}

impl Renderable for Context {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let mut rendered_list: Vec<String> = Vec::new();
        let mut is_method_block = false;
        match &self.context_type {
            ContextType::StructFields(fields) => {
                for field in fields {
                    let struct_field_id = format!("structfield.{}", field.name.as_str());
                    let type_anchor = render_type_anchor(
                        (*render_plan.engines.te().get(field.type_argument.type_id())).clone(),
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
                                    : field.type_argument.span().as_str();
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
                        (*render_plan.engines.te().get(field.type_argument.type_id())).clone(),
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
                                    : field.type_argument.span().as_str();
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
                        (*render_plan
                            .engines
                            .te()
                            .get(variant.type_argument.type_id()))
                        .clone(),
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
                                    : variant.type_argument.span().as_str();
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
                is_method_block = true;
                for method in methods {
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
                                param.type_argument.span().as_str()
                            )?;
                        }
                    }
                    write!(fn_sig, ") -> {}", method.return_type.span().as_str())?;
                    let multiline = fn_sig.chars().count() >= 60;
                    let fn_sig = format!("fn {}(", method.name);
                    let method_id = format!("tymethod.{}", method.name.as_str());
                    let method_attrs = method.attributes.clone();

                    let rendered_method = box_html! {
                        div(id=&method_id, class="method has-srclink") {
                            a(href=format!("{IDENTITY}{method_id}"), class="anchor");
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
                                            : param.type_argument.span().as_str();
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
                                            : param.type_argument.span().as_str();
                                        }
                                        @ if param.name.as_str()
                                            != method.parameters.last()
                                            .expect("no last element in trait method parameters list")
                                            .name.as_str() {
                                            : ", ";
                                        }
                                    }
                                    : ")";
                                }
                                @ if !method.return_type.span().as_str().contains(&fn_sig) {
                                    : " -> ";
                                    : method.return_type.span().as_str();
                                }
                            }
                        }
                    }.into_string()?;

                    rendered_list.push(
                        box_html! {
                            @ if !method_attrs.is_empty() {
                                details(class="swaydoc-toggle open") {
                                    summary {
                                        : Raw(rendered_method);
                                    }
                                    div(class="docblock") {
                                        : Raw(method_attrs.to_html_string());
                                    }
                                }
                            } else {
                                : Raw(rendered_method);
                            }
                        }
                        .into_string()?,
                    );
                }
            }
        };
        Ok(box_html! {
            @ if is_method_block {
                div(class="methods") {
                    @ for item in rendered_list {
                        : Raw(item);
                    }
                }
            } else {
                @ for item in rendered_list {
                    : Raw(item);
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct DocImplTrait {
    pub impl_for_module: ModuleInfo,
    pub impl_trait: TyImplSelfOrTrait,
    pub module_info_override: Option<Vec<String>>,
}

impl DocImplTrait {
    pub fn short_name(&self) -> String {
        self.impl_trait.trait_name.suffix.as_str().to_string()
    }

    pub fn type_args(&self) -> Vec<String> {
        self.impl_trait
            .trait_type_arguments
            .iter()
            .map(|arg| arg.span().as_str().to_string())
            .collect()
    }

    pub fn name_with_type_args(&self) -> String {
        let type_args = self.type_args();
        if !type_args.is_empty() {
            format!("{}<{}>", self.short_name(), type_args.join(", "))
        } else {
            self.short_name()
        }
    }

    // If the trait name is the same as the declaration's name, it's an inherent implementation.
    // Otherwise, it's a trait implementation.
    pub fn is_inherent(&self) -> bool {
        self.short_name() == self.impl_trait.implementing_for.span().as_str()
            || self.short_name() == "r#Self"
    }
}

#[derive(Clone, Debug, Default)]
/// The context section of an item that appears in the page [ItemBody].
pub struct ItemContext {
    /// [Context] can be fields on a struct, variants of an enum, etc.
    pub context_opt: Option<Context>,
    // The implementations for this type.
    pub inherent_impls: Option<Vec<DocImplTrait>>,
    /// The traits implemented for this type.
    pub impl_traits: Option<Vec<DocImplTrait>>,
}

impl ItemContext {
    pub fn to_doclinks(&self) -> DocLinks {
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

        if let Some(inherent_impls) = &self.inherent_impls {
            let mut doc_links = Vec::new();
            for inherent_impl in inherent_impls {
                for item in &inherent_impl.impl_trait.items {
                    if let TyTraitItem::Fn(item_fn) = item {
                        let method_name = item_fn.name().to_string();
                        doc_links.push(DocLink {
                            name: method_name.clone(),
                            module_info: inherent_impl.impl_for_module.clone(),
                            html_filename: format!("{IDENTITY}method.{method_name}"),
                            preview_opt: None,
                        })
                    }
                }
            }
            links.insert(BlockTitle::ImplMethods, doc_links);
        }

        if let Some(impl_traits) = &self.impl_traits {
            let doc_links = impl_traits
                .iter()
                .map(|impl_trait| DocLink {
                    name: impl_trait.name_with_type_args(),
                    module_info: impl_trait.impl_for_module.clone(),
                    html_filename: format!("{}impl-{}", IDENTITY, impl_trait.name_with_type_args()),
                    preview_opt: None,
                })
                .collect();
            links.insert(BlockTitle::ImplTraits, doc_links);
        }

        DocLinks {
            style: DocStyle::Item {
                title: None,
                name: None,
            },
            links,
        }
    }
}
impl Renderable for ItemContext {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let context_opt = match self.context_opt {
            Some(context) => {
                let title = context.context_type.title();
                let rendered_list = context.render(render_plan.clone())?;
                let lct = title.html_title_string();
                Some(
                    box_html! {
                        h2(id=&lct, class=format!("{} small-section-header", &lct)) {
                            : title.as_str();
                            a(class="anchor", href=format!("{IDENTITY}{lct}"));
                        }
                        : rendered_list;
                    }
                    .into_string()?,
                )
            }
            None => None,
        };

        let impl_traits = match self.impl_traits {
            Some(impl_traits) => {
                let mut impl_trait_vec: Vec<_> = Vec::with_capacity(impl_traits.len());
                for impl_trait in impl_traits {
                    impl_trait_vec.push(impl_trait.render(render_plan.clone())?);
                }
                impl_trait_vec
            }
            None => vec![],
        };

        let inherent_impls = match self.inherent_impls {
            Some(inherent_impls) => {
                let mut inherent_impl_vec: Vec<_> = Vec::with_capacity(inherent_impls.len());
                for inherent_impl in inherent_impls {
                    inherent_impl_vec.push(inherent_impl.render(render_plan.clone())?);
                }
                inherent_impl_vec
            }
            None => vec![],
        };

        Ok(box_html! {
            @ if let Some(context) = context_opt {
                : Raw(context);
            }
            @ if !inherent_impls.is_empty() {
                h2(id="methods", class="small-section-header") {
                    : "Implementations";
                    a(href=format!("{IDENTITY}methods"), class="anchor");
                }
                div(id="methods-list") {
                    @ for inherent_impl in inherent_impls {
                        : inherent_impl;
                    }
                }
            }
            @ if !impl_traits.is_empty() {
                h2(id="trait-implementations", class="small-section-header") {
                    : "Trait Implementations";
                    a(href=format!("{IDENTITY}trait-implementations"), class="anchor");
                }
                div(id="trait-implementations-list") {
                    @ for impl_trait in impl_traits {
                        : impl_trait;
                    }
                }
            }
        })
    }
}
impl Renderable for DocImplTrait {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let TyImplSelfOrTrait {
            trait_name,
            items,
            implementing_for,
            ..
        } = &self.impl_trait;
        let short_name = self.short_name();
        let name_with_type_args = self.name_with_type_args();
        let type_args = self.type_args();
        let is_inherent = self.is_inherent();
        let impl_for_module = &self.impl_for_module;
        let no_deps = render_plan.no_deps;
        let is_external_item = if let Some(project_root) = trait_name.prefixes.first() {
            project_root.as_str() != impl_for_module.project_name()
        } else {
            false
        };

        let trait_link = if let Some(module_prefixes) = &self.module_info_override {
            ModuleInfo::from_vec_str(module_prefixes).file_path_from_location(
                &format!("trait.{short_name}.html"),
                impl_for_module,
                is_external_item,
            )?
        } else {
            ModuleInfo::from_call_path(trait_name).file_path_from_location(
                &format!("trait.{short_name}.html"),
                impl_for_module,
                is_external_item,
            )?
        };

        let mut rendered_items = Vec::with_capacity(items.len());
        for item in items {
            rendered_items.push(item.clone().render(render_plan.clone())?)
        }

        let impl_for = box_html! {
                div(id=format!("impl-{}", name_with_type_args), class="impl has-srclink") {
                a(href=format!("{IDENTITY}impl-{}", name_with_type_args), class="anchor");
                h3(class="code-header in-band") {
                    : "impl ";
                    @ if !is_inherent {
                        @ if no_deps && is_external_item {
                            : name_with_type_args;
                        } else {
                            a(class="trait", href=format!("{trait_link}")) {
                                : short_name;
                            }
                            @ for arg in &type_args {
                                @ if arg == type_args.first().unwrap() {
                                    : "<";
                                }
                                : arg;
                                @ if arg != type_args.last().unwrap() {
                                    : ", ";
                                }
                                @ if arg == type_args.last().unwrap() {
                                    : ">";
                                }
                            }
                        }
                        : " for ";
                    }
                    : implementing_for.span().as_str();
                }
            }
        }
        .into_string()?;

        Ok(box_html! {
            // check if the implementation has methods
            @ if !rendered_items.is_empty() {
                details(class="swaydoc-toggle implementors-toggle", open) {
                    summary {
                        : Raw(impl_for);
                    }
                    div(class="impl-items") {
                        @ for item in rendered_items {
                            : item;
                        }
                    }
                }
            } else {
                : Raw(impl_for);
            }
        })
    }
}
impl Renderable for TyTraitItem {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        match self {
            TyTraitItem::Fn(decl_ref) => {
                let decl = render_plan.engines.de().get_function(decl_ref.id());
                <TyFunctionDecl as Clone>::clone(&decl).render(render_plan)
            }
            TyTraitItem::Constant(ref decl_ref) => {
                let decl = render_plan.engines.de().get_constant(decl_ref.id());
                <TyConstantDecl as Clone>::clone(&decl).render(render_plan)
            }
            TyTraitItem::Type(ref decl_ref) => {
                let decl = render_plan.engines.de().get_type(decl_ref.id());
                <TyTraitType as Clone>::clone(&decl).render(render_plan)
            }
        }
    }
}

impl Renderable for TyFunctionDecl {
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let attributes = self.attributes.to_html_string();

        let mut fn_sig = format!("fn {}(", self.name.as_str());
        for param in self.parameters.iter() {
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
                    param.type_argument.span().as_str()
                )?;
            }
        }
        write!(fn_sig, ") -> {}", self.return_type.span().as_str())?;
        let multiline = fn_sig.chars().count() >= 60;

        let method_id = format!("method.{}", self.name.as_str());

        let impl_list = box_html! {
            div(id=format!("{method_id}"), class="method trait-impl") {
                        a(href=format!("{IDENTITY}{method_id}"), class="anchor");
                        h4(class="code-header") {
                            @ if self.visibility.is_public() {
                                : "pub ";
                            }
                            : "fn ";
                            a(class="fnname", href=format!("{IDENTITY}{method_id}")) {
                                : self.name.as_str();
                            }
                            : "(";
                            @ if multiline {
                                @ for param in self.parameters.iter() {
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
                                        : param.type_argument.span().as_str();
                                        : ","
                                    }
                                }
                                br;
                                : ")";
                            } else {
                                @ for param in self.parameters.iter() {
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
                                        : param.type_argument.span().as_str();
                                    }
                                    @ if param.name.as_str()
                                        != self.parameters.last()
                                        .expect("no last element in trait method parameters list")
                                        .name.as_str() {
                                        : ", ";
                                    }
                                }
                                : ")";
                            }
                            @ if self.span.as_str().contains("->") {
                                : " -> ";
                                : self.return_type.span().as_str();
                            }
                        }
                    }
        }
        .into_string()?;

        Ok(box_html! {
            @ if !attributes.is_empty() {
                details(class="swaydoc-toggle method-toggle", open) {
                    summary {
                        : Raw(impl_list);
                    }
                    div(class="docblock") {
                        : Raw(attributes);
                    }
                }
            } else {
                : Raw(impl_list);
            }
        })
    }
}

impl Renderable for TyTraitType {
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let attributes = self.attributes.to_html_string();
        let trait_type_id = format!("traittype.{}", self.name.as_str());
        let contents = box_html! {
            div(id=format!("{trait_type_id}"), class="type trait-impl") {
                        a(href=format!("{IDENTITY}{trait_type_id}"), class="anchor");
                        h4(class="code-header") {
                            : self.span.as_str();
                        }
                    }
        }
        .into_string()?;

        Ok(box_html! {
            @ if !attributes.is_empty() {
                details(class="swaydoc-toggle method-toggle", open) {
                    summary {
                        : Raw(contents);
                    }
                    div(class="docblock") {
                        : Raw(attributes);
                    }
                }
            } else {
                : Raw(contents);
            }
        })
    }
}

impl Renderable for TyConstantDecl {
    fn render(self, _render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let attributes = self.attributes.to_html_string();
        let const_id = format!("const.{}", self.call_path.suffix.as_str());
        let contents = box_html! {
            div(id=format!("{const_id}"), class="const trait-impl") {
                        a(href=format!("{IDENTITY}{const_id}"), class="anchor");
                        h4(class="code-header") {
                            : self.span.as_str();
                        }
                    }
        }
        .into_string()?;

        Ok(box_html! {
            @ if !attributes.is_empty() {
                details(class="swaydoc-toggle method-toggle", open) {
                    summary {
                        : Raw(contents);
                    }
                    div(class="docblock") {
                        : Raw(attributes);
                    }
                }
            } else {
                : Raw(contents);
            }
        })
    }
}

#[derive(Clone, Debug)]
/// Represents the type of [Context] for item declarations that have
/// fields, variants or methods, and acts as a wrapper for those values for rendering.
pub enum ContextType {
    /// Stores the fields on a struct to be rendered.
    StructFields(Vec<TyStructField>),
    /// Stores the fields in storage to be rendered.
    StorageFields(Vec<TyStorageField>),
    /// Stores the variants of an enum to be rendered.
    EnumVariants(Vec<TyEnumVariant>),
    /// Stores the methods of a trait or abi to be rendered.
    RequiredMethods(Vec<TyTraitFn>),
}
impl DocBlock for ContextType {
    fn title(&self) -> BlockTitle {
        match self {
            ContextType::StructFields(_) | ContextType::StorageFields(_) => BlockTitle::Fields,
            ContextType::EnumVariants(_) => BlockTitle::Variants,
            ContextType::RequiredMethods(_) => BlockTitle::RequiredMethods,
        }
    }

    fn name(&self) -> &str {
        match self {
            ContextType::StructFields(_) => "struct_fields",
            ContextType::StorageFields(_) => "storage_fields",
            ContextType::EnumVariants(_) => "enum_variants",
            ContextType::RequiredMethods(_) => "required_methods",
        }
    }
}
