use crate::core::session::Session;
use lspower::lsp::{DocumentHighlight, DocumentHighlightParams};
use std::sync::Arc;

pub fn get_highlights(
    session: Arc<Session>,
    params: DocumentHighlightParams,
) -> Option<Vec<DocumentHighlight>> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    match session.get_token_ranges(&url, position) {
        Some(ranges) => Some(
            ranges
                .into_iter()
                .map(|range| DocumentHighlight { range, kind: None })
                .collect(),
        ),
        _ => None,
    }
}
