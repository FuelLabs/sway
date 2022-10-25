use crate::core::token::TokenMap;
use sway_types::{Ident, Span};
use tower_lsp::lsp_types::{Position, Range};

/// Given a cursor `Position`, return the `Ident` and `Span` of a token in the
/// `TokenMap` if one exists at that position.
pub(crate) fn ident_and_span_at_position(
    cursor_position: Position,
    tokens: &TokenMap,
) -> Option<(Ident, Span)> {
    for item in tokens.iter() {
        let (ident, span) = item.key();
        let range = get_range_from_span(span);
        if cursor_position >= range.start && cursor_position <= range.end {
            return Some((ident.clone(), span.clone()));
        }
    }
    None
}

/// Given a `Span`, convert into an `lsp_types::Range` and return.
pub(crate) fn get_range_from_span(span: &Span) -> Range {
    let start = span.start_pos().line_col();
    let end = span.end_pos().line_col();

    let start_line = start.0 as u32 - 1;
    let start_character = start.1 as u32 - 1;

    let end_line = end.0 as u32 - 1;
    let end_character = end.1 as u32 - 1;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}
