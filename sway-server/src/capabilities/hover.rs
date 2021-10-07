use crate::{
    core::{session::Session, token::Token, token_type::TokenType},
    utils::common::extract_visibility,
};
use lspower::lsp::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use std::sync::Arc;

pub fn get_hover_data(session: Arc<Session>, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;

    session.get_token_hover_content(url, position)
}

pub fn to_hover_content(token: &Token) -> Hover {
    let value = get_hover_format(token);

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("```sway\n{}\n```", value),
            kind: MarkupKind::Markdown,
        }),
        range: Some(token.range),
    }
}

fn get_hover_format(token: &Token) -> String {
    match &token.token_type {
        TokenType::FunctionDeclaration(func_details) => func_details.signature.clone(),
        TokenType::Struct(struct_details) => format!(
            "{}struct {}",
            extract_visibility(&struct_details.visibility),
            &token.name
        ),
        TokenType::Trait(_) => format!("trait {}", &token.name),
        TokenType::Enum => format!("enum {}", &token.name),
        _ => token.name.clone(),
    }
}
