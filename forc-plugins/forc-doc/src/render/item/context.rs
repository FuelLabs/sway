use crate::{
    doc::module::ModuleInfo,
    render::{
        constant::IDENTITY, item::type_anchor::render_type_anchor, link::*, title::DocBlockTitle,
        title::*, util::format::docstring::DocStrings, DocStyle, Renderable,
    },
    RenderPlan,
};
use anyhow::{anyhow, Result};
use horrorshow::{box_html, Raw, RenderBox, Template};
use std::{collections::BTreeMap, fmt::Write};
use sway_core::language::ty::{
    TyEnumVariant, TyImplTrait, TyStorageField, TyStructField, TyTraitFn,
};

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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
/// The context section of an item that appears in the page [ItemBody].
pub(crate) struct ItemContext {
    /// [Context] can be fields on a struct, variants of an enum, etc.
    pub(crate) context_opt: Option<Context>,
    /// The traits implemented for this type.
    pub(crate) impl_traits: Option<Vec<TyImplTrait>>,
    // TODO: All other Implementation types, eg
    // implementations on foreign types, method implementations, etc.
}
impl ItemContext {
    pub(crate) fn to_doclinks(&self) -> DocLinks {
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
        let (title, rendered_list) = match self.context_opt {
            Some(context) => {
                let title = context.context_type.as_block_title();
                let rendered_list = context.render(render_plan.clone())?;
                Ok((title, rendered_list))
            }
            None => Err(anyhow!(
                "Safeguard against render call on empty context failed."
            )),
        }?;
        let mut impl_vec: Vec<_> = Vec::new();
        if let Some(impl_traits) = self.impl_traits {
            for impl_trait in impl_traits {
                impl_vec.push(impl_trait.render(render_plan.clone()))
            }
        };
        let lct = title.html_title_string();
        Ok(box_html! {
            h2(id=&lct, class=format!("{} small-section-header", &lct)) {
                : title.as_str();
                a(class="anchor", href=format!("{IDENTITY}{lct}"));
            }
            : rendered_list;
            @ if !impl_vec.is_empty() {
                h2(id="trait-implementations", class="small-section-header") {
                    : "Trait Implementations";
                    a(href="#trait-implementations", class="anchor");
                }
                div(id="trait-implementations-list") {
                    @ for impl_trait in impl_vec {
                        : impl_trait;
                    }
                }
            }
        })
    }
}
impl Renderable for TyImplTrait {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let TyImplTrait {
            impl_type_parameters,
            trait_name,
            trait_type_arguments,
            items,
            trait_decl_ref,
            implementing_for,
            span,
        } = self;
        Ok(box_html! {
            details(class="swaydoc-toggle implementors-toggle") {
                summary {
                    div(id=format!("impl-{}", trait_name.suffix)) {
                        a(href=format!("{IDENTITY}impl-{}", trait_name.suffix), class="anchor");
                        h3(class="code-header in-band") {
                            : "impl";

                        }
                    }
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
/// Represents the type of [Context] for item declarations that have
/// fields, variants or methods, and acts as a wrapper for those values for rendering.
pub(crate) enum ContextType {
    /// Stores the fields on a struct to be rendered.
    StructFields(Vec<TyStructField>),
    /// Stores the fields in storage to be rendered.
    StorageFields(Vec<TyStorageField>),
    /// Stores the variants of an enum to be rendered.
    EnumVariants(Vec<TyEnumVariant>),
    /// Stores the methods of a trait or abi to be rendered.
    RequiredMethods(Vec<TyTraitFn>),
}
impl DocBlockTitle for ContextType {
    fn as_block_title(&self) -> BlockTitle {
        match self {
            ContextType::StructFields(_) => BlockTitle::Fields,
            ContextType::StorageFields(_) => BlockTitle::Fields,
            ContextType::EnumVariants(_) => BlockTitle::Variants,
            ContextType::RequiredMethods(_) => BlockTitle::RequiredMethods,
        }
    }
}
