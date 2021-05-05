use dashmap::DashMap;
use lspower::lsp::{TextDocumentItem, Url};
#[derive(Debug)]
pub struct Session {
    documents: DashMap<String, String>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
        }
    }

    pub fn store_document(&self, document: TextDocumentItem) -> Result<(), SessionError> {
        let uri = document.uri.to_string();
        match self.documents.insert(uri, document.text) {
            None => Ok(()),
            _ => Err(SessionError::DocumentAlreadyOpened),
        }
    }

    pub fn get_document(&self, uri: &Url) -> Result<String, SessionError> {
        match self.documents.get(uri.as_str()) {
            Some(document) => {
                let document = document.clone();
                Ok(document)
            }
            None => Err(SessionError::DocumentAlreadyClosed),
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    DocumentAlreadyOpened,
    DocumentAlreadyClosed,
}
