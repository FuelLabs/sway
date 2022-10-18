use crate::{descriptor::DescriptorType, doc::Documentation};
use horrorshow::{box_html, html, prelude::*};
use sway_core::language::ty::{
    TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration, TyImplTrait,
    TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration,
};

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
    pub fn render(raw: &Documentation) -> RenderedDocumentation {
        let mut buf: RenderedDocumentation = Default::default();
        for doc in raw {
            let rendered_content = match &doc.desc_ty {
                DescriptorType::Struct(struct_decl) => struct_decl.render(),
                DescriptorType::Enum(enum_decl) => enum_decl.render(),
                DescriptorType::Trait(trait_decl) => trait_decl.render(),
                DescriptorType::Abi(abi_decl) => abi_decl.render(),
                DescriptorType::Storage(storage_decl) => storage_decl.render(),
                DescriptorType::ImplTraitDesc(impl_trait_decl) => impl_trait_decl.render(),
                DescriptorType::Function(fn_decl) => fn_decl.render(),
                DescriptorType::Const(const_decl) => const_decl.render(),
            };
            buf.push(Self {
                module_prefix: doc.module_prefix.clone(),
                file_name: doc.file_name(),
                file_contents: HTMLString(page_from(rendered_content)),
            })
        }
        buf
    }
}

pub(crate) fn page_from(rendered_content: Box<dyn RenderBox>) -> String {
    let markup = html! {
        : rendered_content
    };

    return markup.to_string();
}

/// Basic HTML header component
pub(crate) fn header(module: String, desc_ty: String, desc_name: String) -> Box<dyn RenderBox> {
    box_html! {
        head {
            meta(charset="utf-8");
            meta(name="viewport", content="width=device-width, initial-scale=1.0");
            meta(name="generator", content="forc-doc");
            meta(name="description", content=format!("API documentation for the Sway `{desc_name}` {desc_ty} in crate `{module}`."));
            meta(name="keywords", content=format!("sway, swaylang, sway-lang, {desc_name}"));
            title: format!("{desc_name} in {module} - Sway");
            // TODO: Add links for CSS & Fonts
        }
    }
}
/// HTML body component
pub(crate) fn body(module: String, desc_ty: String, desc_name: String) -> Box<dyn RenderBox> {
    box_html! {
        // TODO: match on ty and make this dynamic
        // e.g. an enum will have variants but a trait will not
        //
        // if matching doesn't work we will have to make a separate
        // body fn for each ty
        body(class=format!("forc-doc {desc_ty}")) {
            // TODO: create nav sidebar
            // create main
            // create main content

            // this is the main code block
            div(class="docblock item-decl") {
                pre(class=format!("sway {desc_ty}")) {
                    code {
                        // code goes here
                    }
                }
            }
            // expand or hide description of main code block
            details(class="forcdoc-toggle top-doc", open) {
                summary(class="hideme") {
                    span { :"Expand description" }
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

trait Renderable {
    fn render(&self) -> Box<dyn RenderBox>;
}

impl Renderable for TyStructDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyEnumDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyTraitDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyAbiDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyStorageDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyImplTrait {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyFunctionDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
impl Renderable for TyConstantDeclaration {
    fn render(&self) -> Box<dyn RenderBox> {
        box_html! {}
    }
}
