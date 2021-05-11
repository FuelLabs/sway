use dashmap::DashMap;
use lspower::lsp::{TextDocumentContentChangeEvent, TextDocumentItem, Url};

use super::document::{DocumentError, TextDocument};

#[derive(Debug)]
pub struct Session {
    documents: DashMap<Url, TextDocument>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
        }
    }

    pub fn store_document(&self, document: &TextDocumentItem) -> Result<(), SessionError> {
        let text_document = TextDocument::new(document);
        let uri = document.uri.clone();

        match self.documents.insert(uri, text_document) {
            None => Ok(()),
            _ => Err(SessionError::DocumentAlreadyOpened),
        }
    }

    pub fn parse_document(&self, url: &Url) -> Result<(), DocumentError> {
        let mut text_document = self.documents.get_mut(&url).unwrap();
        text_document.parse()
    }

    pub fn update_text_document(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), SessionError> {
        let mut text_document = self.documents.get_mut(&uri).unwrap();

        changes.iter().for_each(|change| {
            text_document.apply_change(change);
        });

        Ok(())
    }

    pub fn get_document_text_as_string(&self, uri: &Url) -> Result<String, SessionError> {
        match self.documents.get(uri) {
            Some(document) => Ok(document.get_text_as_string()),
            None => Err(SessionError::DocumentAlreadyClosed),
        }
    }

    pub fn remove_document(&self, uri: &Url) -> Result<TextDocument, SessionError> {
        match self.documents.remove(uri) {
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
