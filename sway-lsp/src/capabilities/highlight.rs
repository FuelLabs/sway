use crate::core::session::Session;
use std::sync::Arc;
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightParams};

pub fn get_highlights(
    session: Arc<Session>,
    params: DocumentHighlightParams,
) -> Option<Vec<DocumentHighlight>> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    session.get_token_ranges(&url, position).map(|ranges| {
        ranges
            .into_iter()
            .map(|range| DocumentHighlight { range, kind: None })
            .collect()
    })
}
