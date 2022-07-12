use std::{collections::HashMap, sync::Arc};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};

use crate::core::{session::Session, token::AstToken};
use crate::utils::common::get_range_from_span;

pub fn rename(session: Arc<Session>, params: RenameParams) -> Option<WorkspaceEdit> {
    let new_name = params.new_name;
    let url = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    match session.documents.get(url.path()) {
        Some(ref document) => {
            if let Some((_, token)) = document.token_at_position(position) {
                let mut edits = Vec::new();

                // todo: currently only supports single file rename
                for (ident, _) in document.all_references_of_token(token) {
                    let range = get_range_from_span(&ident.span());
                    edits.push(TextEdit::new(range, new_name.clone()));
                }

                let mut map_of_changes = HashMap::new();
                map_of_changes.insert(url.clone(), edits);

                Some(WorkspaceEdit::new(map_of_changes))
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
            if let Some((ident, token)) = document.token_at_position(params.position) {
                match token.parsed {
                    AstToken::Reassignment(_) => None,
                    _ => Some(PrepareRenameResponse::RangeWithPlaceholder {
                        range: get_range_from_span(&ident.span()),
                        placeholder: ident.as_str().to_string(),
                    }),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
