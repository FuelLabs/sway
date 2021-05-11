use crate::core::token::Token;
use lspower::lsp::{Hover, HoverContents, MarkupContent, MarkupKind};

pub fn get_hover_data(token: Token) -> Option<Hover> {
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("{:?} : {}", token.token_type, token.name),
            kind: MarkupKind::PlainText,
        }),
        range: Some(token.range),
    })
}
