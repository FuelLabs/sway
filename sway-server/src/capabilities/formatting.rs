use std::sync::Arc;

use crate::core::session::Session;
use formatter::get_formatted_data;
use lspower::lsp::{
    DocumentFormattingParams, FormattingOptions, Position, Range, TextDocumentIdentifier, TextEdit,
};

pub fn format_document(
    session: Arc<Session>,
    params: DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    let options: FormattingOptions = params.options;
    let text_document: TextDocumentIdentifier = params.text_document;
    let url = text_document.uri;

    session.format_text(&url, options)
}

pub fn get_format_text_edits(text: &str, options: FormattingOptions) -> Option<Vec<TextEdit>> {
    // we only format if code is correct
    match get_formatted_data(text, options.tab_size) {
        Ok((num_of_lines, formatted_text)) => {
            let text_lines_count = text.split('\n').count();
            let line_end = std::cmp::max(num_of_lines, text_lines_count) as u32;

            let main_edit = TextEdit {
                range: Range::new(Position::new(0, 0), Position::new(line_end as u32, 0)),
                new_text: formatted_text,
            };

            Some(vec![main_edit])
        }
        _ => None,
    }
}
