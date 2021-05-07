use dashmap::DashMap;
use lspower::lsp::{Position, TextDocumentContentChangeEvent, TextDocumentItem, Url};
use ropey::{Rope, RopeSlice};

use super::document::TextDocument;

#[derive(Debug)]
pub struct Session {
    documents: DashMap<String, TextDocument>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
        }
    }

    pub fn store_document(&self, document: &TextDocumentItem) -> Result<(), SessionError> {
        let text_document = TextDocument::new(document);
        let uri = document.uri.as_str().into();

        match self.documents.insert(uri, text_document) {
            None => Ok(()),
            _ => Err(SessionError::DocumentAlreadyOpened),
        }
    }

    pub fn update_document(
        &self,
        uri: Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), SessionError> {
        let uri = uri.as_str();
        let mut text_document = self.documents.get_mut(uri).unwrap();

        changes.iter().for_each(|change| {
            text_document.apply_change(change);
        });

        Ok(())
    }

    pub fn get_document(&self, key: &str, position: Position) -> Option<String> {
        if let Some(document) = self.documents.get(key) {
            Some(document.get_slice(position))
        } else {
            None
        }
    }

    pub fn get_document_text_as_string(&self, uri: &Url) -> Result<String, SessionError> {
        match self.documents.get(uri.as_str()) {
            Some(document) => Ok(document.get_text_as_string()),
            None => Err(SessionError::DocumentAlreadyClosed),
        }
    }

    pub fn remove_document(&self, uri: &Url) -> Result<TextDocument, SessionError> {
        match self.documents.remove(uri.as_str()) {
            Some((_, text_document)) => Ok(text_document),
            None => Err(SessionError::DocumentAlreadyClosed),
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    DocumentAlreadyOpened,
    DocumentAlreadyClosed,
}
