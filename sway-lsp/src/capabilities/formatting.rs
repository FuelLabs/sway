use crate::{
    core::document::Documents,
    error::{DocumentError, LanguageServerError},
};
use lsp_types::{Position, Range, TextEdit, Url};
use sway_types::span::Source;
use swayfmt::Formatter;

pub fn format_text(documents: &Documents, url: &Url) -> Result<Vec<TextEdit>, LanguageServerError> {
    let _p = tracing::trace_span!("format_text").entered();
    let document = documents.try_get(url.path()).try_unwrap().ok_or_else(|| {
        DocumentError::DocumentNotFound {
            path: url.path().to_string(),
        }
    })?;

    get_page_text_edit(document.get_text().into(), &mut <_>::default())
        .map(|page_text_edit| vec![page_text_edit])
}

pub fn get_page_text_edit(
    src: Source,
    formatter: &mut Formatter,
) -> Result<TextEdit, LanguageServerError> {
    // we only format if code is correct
    let formatted_code = formatter
        .format(src.clone())
        .map_err(LanguageServerError::FormatError)?;

    let text_lines_count = src.text.split('\n').count();
    let num_of_lines = formatted_code.split('\n').count();
    let line_end = std::cmp::max(num_of_lines, text_lines_count) as u32;

    Ok(TextEdit {
        range: Range::new(Position::new(0, 0), Position::new(line_end, 0)),
        new_text: formatted_code,
    })
}
