use crate::core::session::Session;
use lsp_types::{DocumentHighlight, Position, Url};

pub fn get_highlights(
    session: &Session,
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
