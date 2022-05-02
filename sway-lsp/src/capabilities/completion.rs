use crate::core::{session::Session, token::Token, token_type::TokenType};
use std::sync::Arc;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
};

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
        TokenType::VariableDeclaration(_) | TokenType::VariableExpression => {
            Some(CompletionItemKind::VARIABLE)
        }
        TokenType::FunctionDeclaration(_)
        | &TokenType::FunctionApplication
        | TokenType::TraitFunction => Some(CompletionItemKind::FUNCTION),
        TokenType::TraitDeclaration(_) | TokenType::ImplTrait => {
            Some(CompletionItemKind::INTERFACE)
        }
        TokenType::StructDeclaration(_) | TokenType::Struct => Some(CompletionItemKind::STRUCT),
        TokenType::EnumDeclaration(_) | TokenType::EnumVariant | TokenType::EnumApplication => {
            Some(CompletionItemKind::ENUM)
        }
        TokenType::ConstantDeclaration(_) => Some(CompletionItemKind::CONSTANT),
        TokenType::Library => Some(CompletionItemKind::MODULE),
        TokenType::Reassignment => Some(CompletionItemKind::OPERATOR),
        _ => None,
    }
}
