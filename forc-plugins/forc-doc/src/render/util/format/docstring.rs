use comrak::{markdown_to_html, ComrakOptions};
use std::fmt::Write;
use sway_core::transform::{AttributeKind, AttributesMap};
use sway_lsp::utils::markdown::format_docs;

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
            for arg in vec_attrs.iter().flat_map(|attribute| &attribute.args) {
                writeln!(docs, "{}", arg.name.as_str()).expect(
                    "problem appending `arg.name.as_str()` to `docs` with `writeln` macro.",
                );
            }
        }
        docs
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
