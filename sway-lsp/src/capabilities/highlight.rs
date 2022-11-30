use crate::core::session::Session;
use std::sync::Arc;
use tower_lsp::lsp_types::{DocumentHighlight, Position, Url};

pub(crate) fn get_highlights(
    session: Arc<Session>,
    url: Url,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    session.token_ranges(&url, position).map(|ranges| {
        ranges
            .into_iter()
            .map(|range| DocumentHighlight { range, kind: None })
            .collect()
    })
}
