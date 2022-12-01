use crate::error::LanguageServerError;
use std::sync::Arc;
use swayfmt::Formatter;
use tower_lsp::lsp_types::{Position, Range, TextEdit};

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
        range: Range::new(Position::new(0, 0), Position::new(line_end as u32, 0)),
        new_text: formatted_code,
    })
}
