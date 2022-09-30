use crate::{descriptor::Descriptor, doc::Documentation};

pub(crate) struct HTMLString(pub(crate) String);

pub(crate) struct RenderedDocumentation {
    pub(crate) file_contents: HTMLString,
    pub(crate) file_name: String,
}
impl RenderedDocumentation {
    pub fn render(raw: &Documentation) -> Vec<RenderedDocumentation> {
        let mut buf: Vec<RenderedDocumentation> = Default::default();
        for (desc, (_docs, _ty)) in raw {
            let file_name = match desc.to_file_name() {
                Some(x) => x,
                None => continue,
            };
            if let Descriptor::Documentable { ty, name } = desc {
                let name_str = match name {
                    Some(name) => name.as_str(),
                    None => ty.to_name(),
                };
                buf.push(Self {
                    // proof of concept, TODO render actual HTML
                    file_contents: HTMLString(format!("Docs for {:?} {:?}", name_str, ty)),
                    file_name,
                })
            }
        }
        buf
    }
}
