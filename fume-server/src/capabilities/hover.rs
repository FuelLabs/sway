use std::sync::Arc;

use crate::core::{session::Session, token::Token};
use lspower::lsp::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

pub fn get_hover_data(session: Arc<Session>, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;

    session.get_token_hover_content(url, position)
}

pub fn to_hover_content(token: &Token) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("{:?} : {}", token.token_type, token.name),
            kind: MarkupKind::PlainText,
        }),
        range: Some(token.range),
    }
}
