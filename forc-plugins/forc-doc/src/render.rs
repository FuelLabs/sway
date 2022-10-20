use crate::{descriptor::DescriptorType, doc::Documentation};
use horrorshow::{box_html, helper::doctype, html, prelude::*};
use sway_core::language::ty::{
    TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration, TyImplTrait,
    TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration,
};
use sway_core::transform::{AttributeKind, AttributesMap};

pub(crate) struct HTMLString(pub(crate) String);
pub(crate) type RenderedDocumentation = Vec<RenderedDocument>;
/// A [Document] rendered to HTML.
pub(crate) struct RenderedDocument {
    pub(crate) module_prefix: Vec<String>,
    pub(crate) file_name: String,
    pub(crate) file_contents: HTMLString,
}
impl RenderedDocument {
    /// Top level HTML rendering for all [Documentation] of a program.
    pub fn from_raw_docs(raw: &Documentation) -> RenderedDocumentation {
        let mut buf: RenderedDocumentation = Default::default();
        for doc in raw {
            let module_prefix = doc.module_prefix.clone();
            let module = if module_prefix.last().is_some() {
                module_prefix.last().unwrap().to_string()
            } else {
                "root".to_string()
            };
            let decl_ty = doc.desc_ty.as_str().to_string();
            let rendered_content = match &doc.desc_ty {
                DescriptorType::Struct(struct_decl) => struct_decl.render(module, decl_ty),
                DescriptorType::Enum(enum_decl) => enum_decl.render(module, decl_ty),
                DescriptorType::Trait(trait_decl) => trait_decl.render(module, decl_ty),
                DescriptorType::Abi(abi_decl) => abi_decl.render(module, decl_ty),
                DescriptorType::Storage(storage_decl) => storage_decl.render(module, decl_ty),
                DescriptorType::ImplTraitDesc(impl_trait_decl) => {
                    impl_trait_decl.render(module, decl_ty)
                }
                DescriptorType::Function(fn_decl) => fn_decl.render(module, decl_ty),
                DescriptorType::Const(const_decl) => const_decl.render(module, decl_ty),
            };
            buf.push(Self {
                module_prefix,
                file_name: doc.file_name(),
                file_contents: HTMLString(page_from(rendered_content)),
            })
        }
        buf
    }
}

pub(crate) fn page_from(rendered_content: Box<dyn RenderBox>) -> String {
    let markup = html! {
        : doctype::HTML;
        html {
            : rendered_content
        }
    };

    markup.into_string().unwrap()
}

/// Basic HTML header component
pub(crate) fn html_head(module: String, decl_ty: String, decl_name: String) -> Box<dyn RenderBox> {
    box_html! {
        head {
            meta(charset="utf-8");
            meta(name="viewport", content="width=device-width, initial-scale=1.0");
            meta(name="generator", content="forc-doc");
            meta(
                name="description",
                content=format!("API documentation for the Sway `{decl_name}` {decl_ty} in crate `{module}`.")
            );
            meta(name="keywords", content=format!("sway, swaylang, sway-lang, {decl_name}"));
            title: format!("{decl_name} in {module} - Sway");
            // TODO: Add links for CSS & Fonts
        }
    }
}
/// HTML body component
pub(crate) fn html_body(module: String, decl_ty: String, decl_name: String) -> Box<dyn RenderBox> {
    box_html! {
        // TODO: match on ty and make this dynamic
        // e.g. an enum will have variants but a trait will not
        //
        // if matching doesn't work we will have to make a separate
        // body fn for each ty
        body(class=format!("forc-doc {decl_ty}")) {
            // TODO: create nav sidebar
            // create main
            // create main content

            // this is the main code block
            div(class="docblock item-decl") {
                pre(class=format!("sway {decl_ty}")) {
                    code {
                        // code goes here
                    }
                }
            }
            // expand or hide description of main code block
            details(class="forcdoc-toggle top-doc", open) {
                summary(class="hideme") {
                    span { : "Expand description" }
                }
                // this is the description
                div(class="docblock") {
                    p {
                        // description goes here
                    }
                }
            }
        }
    }
}

// TODO: Create `fn index` and `fn all`

fn doc_attributes_to_string_vec(attributes: &AttributesMap) -> Vec<String> {
    let attributes = attributes.get(&AttributeKind::Doc);
    let mut attr_strings = Vec::new();
    if let Some(vec_attrs) = attributes {
        for attribute in vec_attrs {
            for ident in &attribute.args {
                attr_strings.push(ident.as_str().to_string())
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
        let struct_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        let enum_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        } = &self;
        let name = name.as_str().to_string();
        let trait_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        let abi_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        let storage_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        // let impl_trait_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        let function_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
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
        } = &self;
        let name = name.as_str().to_string();
        let const_attributes = doc_attributes_to_string_vec(attributes);
        box_html! {
            : html_head(module.clone(), decl_ty.clone(), name.clone());
            : html_body(module.clone(), decl_ty.clone(), name.clone());
        }
    }
}
