use crate::core::{
    session::Session,
    token::{ExpressionType, Token, TokenType},
};
use lspower::lsp::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use std::sync::Arc;

pub fn get_completion(
    session: Arc<Session>,
    params: CompletionParams,
) -> Option<CompletionResponse> {
    let url = params.text_document_position.text_document.uri;

    match session.get_completion_items(&url) {
        Some(items) => Some(CompletionResponse::Array(items)),
        _ => None,
    }
}

pub fn to_completion_items(tokens: &Vec<Token>) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for token in tokens {
        if token.expression_type == ExpressionType::Declaration {
            let item = CompletionItem {
                label: token.name.clone(),
                kind: get_kind(&token.token_type),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

fn get_kind(token_type: &TokenType) -> Option<CompletionItemKind> {
    match token_type {
        TokenType::Enum => Some(CompletionItemKind::Enum),
        TokenType::Function => Some(CompletionItemKind::Function),
        TokenType::Library => Some(CompletionItemKind::Module),
        TokenType::Struct => Some(CompletionItemKind::Struct),
        TokenType::Variable => Some(CompletionItemKind::Variable),
        TokenType::Trait => Some(CompletionItemKind::Interface),
        _ => None,
    }
}
