use std::sync::Arc;

use lspower::lsp::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
};

use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
};

pub fn handle_open_file(
    session: Arc<Session>,
    params: &DidOpenTextDocumentParams,
) -> Option<Vec<Diagnostic>> {
    if let Ok(_) = session.store_document(&params.text_document) {
        match session.parse_document(&params.text_document.uri) {
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
    } else {
        None
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
    match session.parse_document(&params.text_document.uri) {
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

pub fn handle_close_file(
    session: Arc<Session>,
    params: DidCloseTextDocumentParams,
) -> Result<TextDocument, DocumentError> {
    // TODO
    // should we remove the document after closing ?
    session.remove_document(&params.text_document.uri)
}
