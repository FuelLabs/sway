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

    if let Some(tokens) = session.get_tokens_from_file(&url) {
        let items = get_completion_items(tokens);
        Some(CompletionResponse::Array(items))
    } else {
        None
    }
}

fn get_completion_items(tokens: Vec<Token>) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for token in tokens {
        if token.expression_type == ExpressionType::Declaration {
            let item = CompletionItem {
                label: token.name,
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
