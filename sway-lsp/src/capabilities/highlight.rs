use crate::core::session::Session;
use lsp_types::{DocumentHighlight, Position, Url};
use std::sync::Arc;

pub fn get_highlights(
    session: Arc<Session>,
    url: &Url,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    let _p = tracing::trace_span!("get_highlights").entered();
    session.token_ranges(url, position).map(|ranges| {
        ranges
            .into_iter()
            .map(|range| DocumentHighlight { range, kind: None })
            .collect()
    })
}
