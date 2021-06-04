use dashmap::DashMap;
use lspower::lsp::{
    CompletionItem, Diagnostic, FormattingOptions, GotoDefinitionResponse, Hover, Position, Range,
    SemanticToken, SymbolInformation, TextDocumentContentChangeEvent, TextDocumentItem, TextEdit,
    Url,
};

use crate::capabilities::{self, formatting::get_format_text_edits};

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

    pub fn parse_document(&self, url: &Url) -> Result<Vec<Diagnostic>, DocumentError> {
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
    pub fn get_token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        if let Some(document) = self.documents.get(url) {
            if let Some(token) = document.get_token_at_position(position) {
                let result = document
                    .get_all_tokens_by_single_name(&token.name)
                    .unwrap()
                    .iter()
                    .map(|token| token.range)
                    .collect();

                return Some(result);
            }
        }

        None
    }

    pub fn get_token_hover_content(&self, url: &Url, position: Position) -> Option<Hover> {
        if let Some(document) = self.documents.get(url) {
            if let Some(token) = document.get_token_at_position(position) {
                return Some(capabilities::hover::to_hover_content(token));
            }
        }

        None
    }

    pub fn get_token_definition_response(
        &self,
        url: Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        if let Some(document) = self.documents.get(&url) {
            if let Some(token) = document.get_token_at_position(position) {
                if token.is_initial_declaration() {
                    return Some(capabilities::go_to::to_definition_response(url, token));
                } else {
                    if let Some(other_token) = document.get_declared_token(&token.name) {
                        return Some(capabilities::go_to::to_definition_response(
                            url,
                            other_token,
                        ));
                    }
                }
            }
        }

        None
    }

    pub fn get_completion_items(&self, url: &Url) -> Option<Vec<CompletionItem>> {
        if let Some(document) = self.documents.get(url) {
            return Some(capabilities::completion::to_completion_items(
                document.get_tokens(),
            ));
        }

        None
    }

    pub fn get_semantic_tokens(&self, url: &Url) -> Option<Vec<SemanticToken>> {
        if let Some(document) = self.documents.get(url) {
            return Some(capabilities::semantic_tokens::to_semantic_tokes(
                document.get_tokens(),
            ));
        }

        None
    }

    pub fn get_symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        if let Some(document) = self.documents.get(url) {
            return Some(capabilities::document_symbol::to_symbol_information(
                document.get_tokens(),
                url.clone(),
            ));
        }

        None
    }

    pub fn format_text(&self, url: &Url, options: FormattingOptions) -> Option<Vec<TextEdit>> {
        if let Some(document) = self.documents.get(url) {
            get_format_text_edits(document.get_text(), options)
        } else {
            None
        }
    }
}
