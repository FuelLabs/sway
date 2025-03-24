//! Rendering and formatting for Sway doc attributes.
use crate::render::util::format::constant::*;
use comrak::{markdown_to_html, ComrakOptions};
use std::fmt::Write;
use sway_core::transform::{AttributeKind, Attributes};
use sway_lsp::utils::markdown::format_docs;

pub(crate) trait DocStrings {
    fn to_html_string(&self) -> String;
    fn to_raw_string(&self) -> String;
}
/// Creates an HTML String from [Attributes].
impl DocStrings for Attributes {
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
        options.parse.default_info_string = Some(SWAY_FILEINE.into());
        markdown_to_html(&format_docs(&docs), &options)
    }
    fn to_raw_string(&self) -> String {
        let mut docs = String::new();
        // TODO: Change this logic once https://github.com/FuelLabs/sway/issues/6938 gets implemented.
        for arg in self
            .of_kind(AttributeKind::DocComment)
            .flat_map(|attribute| &attribute.args)
        {
            writeln!(docs, "{}", arg.name.as_str())
                .expect("problem appending `arg.name.as_str()` to `docs` with `writeln` macro.");
        }
        docs
    }
}

/// Create a docstring preview from raw html attributes.
///
/// Returns `None` if there are no attributes.
pub(crate) fn create_preview(raw_attributes: Option<String>) -> Option<String> {
    raw_attributes.as_ref().map(|description| {
        let preview = split_at_markdown_header(description);
        if preview.chars().count() > MAX_PREVIEW_CHARS && preview.contains(CLOSING_PARAGRAPH_TAG) {
            let closing_tag_index = preview
                .find(CLOSING_PARAGRAPH_TAG)
                .expect("closing tag out of range");
            // We add 1 here to get the index of the char after the closing tag.
            // This ensures we retain the closing tag and don't break the html.
            let (preview, _) =
                preview.split_at(closing_tag_index + CLOSING_PARAGRAPH_TAG.len() + 1);
            if preview.chars().count() > MAX_PREVIEW_CHARS && preview.contains(NEWLINE_CHAR) {
                let newline_index = preview
                    .find(NEWLINE_CHAR)
                    .expect("new line char out of range");
                preview.split_at(newline_index).0.to_string()
            } else {
                preview.to_string()
            }
        } else {
            preview.to_string()
        }
    })
}

/// Checks if some raw html (rendered from markdown) contains a header.
/// If it does, it splits at the header and returns the slice that preceded it.
pub(crate) fn split_at_markdown_header(raw_html: &str) -> &str {
    for header in HTML_HEADERS {
        if raw_html.contains(header) {
            let v: Vec<_> = raw_html.split(header).collect();
            return v.first().expect("expected non-empty &str");
        }
        continue;
    }
    raw_html
}
