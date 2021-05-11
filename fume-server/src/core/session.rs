use dashmap::DashMap;
use lspower::lsp::{Position, TextDocumentContentChangeEvent, TextDocumentItem, Url};

use super::{
    document::{DocumentError, TextDocument},
    token::Token,
};

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

    pub fn get_token_from_position(&self, url: &Url, position: Position) -> Option<Token> {
        match self.documents.get(url) {
            Some(document) => document.get_token_at_position(position),
            _ => None,
        }
    }

    pub fn remove_document(&self, uri: &Url) -> Result<TextDocument, SessionError> {
        match self.documents.remove(uri) {
            Some((_, text_document)) => Ok(text_document),
            None => Err(SessionError::DocumentAlreadyClosed),
        }
    }

    pub fn get_tokens_from_file(&self, url: &Url) -> Option<Vec<Token>> {
        match self.documents.get(url) {
            Some(document) => Some(document.get_tokens()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    DocumentAlreadyOpened,
    DocumentAlreadyClosed,
}
