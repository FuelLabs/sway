use crate::{
    core::document::Documents,
    error::{DocumentError, LanguageServerError},
};
use lsp_types::{Position, Range, TextEdit, Url};
use std::sync::Arc;
use swayfmt::Formatter;

pub fn format_text(documents: &Documents, url: &Url) -> Result<Vec<TextEdit>, LanguageServerError> {
    let document = documents.try_get(url.path()).try_unwrap().ok_or_else(|| {
        DocumentError::DocumentNotFound {
            path: url.path().to_string(),
        }
    })?;

    get_page_text_edit(Arc::from(document.get_text()), &mut <_>::default())
        .map(|page_text_edit| vec![page_text_edit])
}

pub fn get_page_text_edit(
    text: Arc<str>,
    formatter: &mut Formatter,
) -> Result<TextEdit, LanguageServerError> {
    // we only format if code is correct
    let formatted_code = formatter
        .format(text.clone(), None)
        .map_err(LanguageServerError::FormatError)?;

    let text_lines_count = text.split('\n').count();
    let num_of_lines = formatted_code.split('\n').count();
    let line_end = std::cmp::max(num_of_lines, text_lines_count) as u32;

    Ok(TextEdit {
        range: Range::new(Position::new(0, 0), Position::new(line_end, 0)),
        new_text: formatted_code,
    })
}
