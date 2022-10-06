use crate::{descriptor::Descriptor, doc::Documentation};
use horrorshow::{box_html, prelude::*};

pub(crate) struct HTMLString(pub(crate) String);

pub(crate) struct RenderedDocumentation {
    pub(crate) file_contents: HTMLString,
    pub(crate) file_name: String,
    pub(crate) module_prefix: Vec<String>,
}
impl RenderedDocumentation {
    pub fn render(raw: &Documentation) -> Vec<RenderedDocumentation> {
        let mut buf: Vec<RenderedDocumentation> = Default::default();
        for (desc, (_docs, _ty)) in raw {
            let file_name = match desc.to_file_name() {
                Some(x) => x,
                None => continue,
            };
            if let Descriptor::Documentable {
                ty,
                name,
                module_prefix,
            } = desc
            {
                let name_str = match name {
                    Some(name) => name.as_str(),
                    None => ty.to_name(),
                };
                buf.push(Self {
                    module_prefix: module_prefix.clone(),
                    // proof of concept, TODO render actual HTML
                    file_contents: HTMLString(todo!()),
                    file_name,
                })
            }
        }
        buf
    }
}

pub(crate) fn create_html_file_name(ty: &str, name: &str) -> String {
    format!("{}.{}.html", ty, name)
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
pub(crate) fn body() -> Box<dyn RenderBox> {
    box_html! {}
}
// TODO: Create `fn index` and `fn all`
