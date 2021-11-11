use crate::core::{session::Session, token::Token, token_type::TokenType};
use lspower::lsp::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use std::sync::Arc;

pub fn get_completion(
    session: Arc<Session>,
    params: CompletionParams,
) -> Option<CompletionResponse> {
    let url = params.text_document_position.text_document.uri;

    session
        .get_completion_items(&url)
        .map(CompletionResponse::Array)
}

pub fn to_completion_items(tokens: &[Token]) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for token in tokens {
        if token.is_initial_declaration() {
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
        TokenType::FunctionDeclaration(_) | &TokenType::FunctionApplication => {
            Some(CompletionItemKind::Function)
        }
        TokenType::Library => Some(CompletionItemKind::Module),
        TokenType::Struct(_) => Some(CompletionItemKind::Struct),
        TokenType::Variable(_) => Some(CompletionItemKind::Variable),
        TokenType::Trait(_) => Some(CompletionItemKind::Interface),
        _ => None,
    }
}
