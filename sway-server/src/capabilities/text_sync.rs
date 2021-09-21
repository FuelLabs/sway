use std::sync::Arc;

use lspower::lsp::{
    Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
};

use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
};

pub fn handle_open_file(
    session: Arc<Session>,
    params: &DidOpenTextDocumentParams,
) -> Vec<Diagnostic> {
    let path = params.text_document.uri.path();

    if !session.contains_sway_file(&params.text_document.uri) {
        if let Ok(text_document) = TextDocument::build_from_path(path) {
            let _ = session.store_document(text_document);
        }
    }

    match session.parse_document(path) {
        Ok(diagnostics) => diagnostics,
        Err(DocumentError::FailedToParse(diagnostics)) => diagnostics,
        _ => vec![],
    }
}

pub fn handle_change_file(
    session: Arc<Session>,
    params: DidChangeTextDocumentParams,
) -> Result<(), DocumentError> {
    session.update_text_document(&params.text_document.uri, params.content_changes)
}

pub fn handle_save_file(
    session: Arc<Session>,
    params: &DidSaveTextDocumentParams,
) -> Option<Vec<Diagnostic>> {
    let path = params.text_document.uri.path();

    match session.parse_document(path) {
        Ok(diagnostics) => {
            if diagnostics.is_empty() {
                None
            } else {
                Some(diagnostics)
            }
        }
        Err(DocumentError::FailedToParse(diagnostics)) => Some(diagnostics),
        _ => None,
    }
}
