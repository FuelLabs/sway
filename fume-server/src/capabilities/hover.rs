use std::sync::Arc;

use crate::core::session::Session;
use lspower::lsp::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

pub fn get_hover_data(session: Arc<Session>, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;

    match session.get_token_at_position(url, position) {
        Some(token) => Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                value: format!("{:?} : {}", token.token_type, token.name),
                kind: MarkupKind::PlainText,
            }),
            range: Some(token.range),
        }),
        _ => None,
    }
}
