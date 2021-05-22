use crate::core::{
    session::Session,
    token::{ContentType, DeclarationType, Token},
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
        if token.is_initial_declaration() {
            let item = CompletionItem {
                label: token.name.clone(),
                kind: get_kind(&token.content_type),
                ..Default::default()
            };
            completion_items.push(item);
        }
    }

    completion_items
}

fn get_kind(content_type: &ContentType) -> Option<CompletionItemKind> {
    if let ContentType::Declaration(dec) = content_type {
        match dec {
            DeclarationType::Enum => Some(CompletionItemKind::Enum),
            DeclarationType::Function => Some(CompletionItemKind::Function),
            DeclarationType::Library => Some(CompletionItemKind::Module),
            DeclarationType::Struct => Some(CompletionItemKind::Struct),
            DeclarationType::Variable => Some(CompletionItemKind::Variable),
            DeclarationType::Trait => Some(CompletionItemKind::Interface),
            _ => None,
        }
    } else {
        None
    }
}
