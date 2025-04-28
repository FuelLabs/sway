use crate::core::{session::Session, token_map::TokenMap};
use lsp_types::{DocumentHighlight, Position, Url};
use std::sync::Arc;

pub fn get_highlights(
    session: Arc<Session>,
    token_map: &TokenMap,
    url: &Url,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    let _p = tracing::trace_span!("get_highlights").entered();
    session.token_ranges(token_map, url, position).map(|ranges| {
        ranges
            .into_iter()
            .map(|range| DocumentHighlight { range, kind: None })
            .collect()
    })
}
