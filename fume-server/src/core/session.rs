use dashmap::DashMap;
use lspower::lsp::{Position, Range, TextDocumentContentChangeEvent, TextDocumentItem, Url};

use super::{
    document::{DocumentError, TextDocument},
    token::{ExpressionType, Token},
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

    // Document
    pub fn store_document(&self, document: &TextDocumentItem) -> Result<(), DocumentError> {
        let text_document = TextDocument::new(document);
        let url = document.uri.clone();

        match self.documents.insert(url, text_document) {
            None => Ok(()),
            _ => Err(DocumentError::DocumentAlreadyStored),
        }
    }

    pub fn remove_document(&self, uri: &Url) -> Result<TextDocument, DocumentError> {
        match self.documents.remove(uri) {
            Some((_, text_document)) => Ok(text_document),
            None => Err(DocumentError::DocumentNotFound),
        }
    }

    pub fn parse_document(&self, url: &Url) -> Result<(), DocumentError> {
        match self.documents.get_mut(&url) {
            Some(ref mut document) => document.parse(),
            _ => Err(DocumentError::DocumentNotFound),
        }
    }

    pub fn update_text_document(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), DocumentError> {
        match self.documents.get_mut(&uri) {
            Some(ref mut document) => {
                changes.iter().for_each(|change| {
                    document.apply_change(change);
                });
                Ok(())
            }
            _ => Err(DocumentError::DocumentNotFound),
        }
    }

    // Token
    pub fn get_token_at_position(&self, url: &Url, position: Position) -> Option<Token> {
        match self.documents.get(url) {
            Some(document) => document.get_token_at_position(position),
            _ => None,
        }
    }

    pub fn get_token_ranges(&self, url: &Url, name: &str) -> Option<Vec<Range>> {
        match self.documents.get(url) {
            Some(document) => {
                let result = document
                    .get_all_tokens_by_single_name(name)
                    .unwrap()
                    .iter()
                    .map(|token| token.range)
                    .collect();
                Some(result)
            }
            _ => None,
        }
    }

    pub fn get_token_by_name_and_expression_type(
        &self,
        url: &Url,
        name: &str,
        definition_type: ExpressionType,
    ) -> Option<Token> {
        match self.documents.get(url) {
            Some(document) => document.get_token_by_name_and_expression_type(name, definition_type),
            _ => None,
        }
    }

    pub fn get_tokens_from_file(&self, url: &Url) -> Option<Vec<Token>> {
        match self.documents.get(url) {
            Some(document) => Some(document.get_tokens()),
            _ => None,
        }
    }
}
