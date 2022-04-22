use std::{collections::HashMap, sync::Arc};

use tower_lsp::lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};

use crate::core::{session::Session, token::Token, token_type::TokenType};

pub fn rename(session: Arc<Session>, params: RenameParams) -> Option<WorkspaceEdit> {
    let new_name = params.new_name;
    let url = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    match session.documents.get(url.path()) {
        Some(ref document) => {
            if let Some(token) = document.get_token_at_position(position) {
                if let Some(tokens) = document.get_all_tokens_by_single_name(&token.name) {
                    // todo: currently only supports single file rename
                    let edits = prepare_token_rename(&tokens, new_name);
                    let mut map_of_changes = HashMap::new();
                    map_of_changes.insert(url.clone(), edits);

                    Some(WorkspaceEdit::new(map_of_changes))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn prepare_rename(
    session: Arc<Session>,
    params: TextDocumentPositionParams,
) -> Option<PrepareRenameResponse> {
    let url = params.text_document.uri;

    match session.documents.get(url.path()) {
        Some(ref document) => {
            if let Some(token) = document.get_token_at_position(params.position) {
                match token.token_type {
                    TokenType::Library | TokenType::Reassignment => None,
                    _ => Some(PrepareRenameResponse::RangeWithPlaceholder {
                        range: token.range,
                        placeholder: token.name.clone(),
                    }),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn prepare_token_rename(tokens: &[&Token], new_name: String) -> Vec<TextEdit> {
    tokens
        .iter()
        .map(|token| TextEdit::new(token.range, new_name.clone()))
        .collect()
}
