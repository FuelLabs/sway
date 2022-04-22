use super::document::{DocumentError, TextDocument};
use crate::{
    capabilities::{self, formatting::get_format_text_edits},
    sway_config::SwayConfig,
};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::{Arc, LockResult, RwLock};
use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, GotoDefinitionResponse, Position, Range, SemanticToken,
    SymbolInformation, TextDocumentContentChangeEvent, TextEdit, Url,
};

pub type Documents = DashMap<String, TextDocument>;

#[derive(Debug)]
pub struct Session {
    pub documents: Documents,
    pub config: RwLock<SwayConfig>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
            config: RwLock::new(SwayConfig::default()),
        }
    }

    // update sway config
    pub fn update_config(&self, options: Value) {
        if let LockResult::Ok(mut config) = self.config.write() {
            *config = SwayConfig::with_options(options);
        }
    }

    // Document
    pub fn store_document(&self, text_document: TextDocument) -> Result<(), DocumentError> {
        match self
            .documents
            .insert(text_document.get_uri().into(), text_document)
        {
            None => Ok(()),
            _ => Err(DocumentError::DocumentAlreadyStored),
        }
    }

    pub fn remove_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        match self.documents.remove(url.path()) {
            Some((_, text_document)) => Ok(text_document),
            None => Err(DocumentError::DocumentNotFound),
        }
    }

    pub fn parse_document(&self, path: &str) -> Result<Vec<Diagnostic>, DocumentError> {
        match self.documents.get_mut(path) {
            Some(ref mut document) => document.parse(),
            _ => Err(DocumentError::DocumentNotFound),
        }
    }

    pub fn contains_sway_file(&self, url: &Url) -> bool {
        self.documents.contains_key(url.path())
    }

    pub fn update_text_document(&self, url: &Url, changes: Vec<TextDocumentContentChangeEvent>) {
        if let Some(ref mut document) = self.documents.get_mut(url.path()) {
            changes.iter().for_each(|change| {
                document.apply_change(change);
            });
        }
    }

    // Token
    pub fn get_token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        if let Some(document) = self.documents.get(url.path()) {
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

    pub fn get_token_definition_response(
        &self,
        url: Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let key = url.path();

        if let Some(document) = self.documents.get(key) {
            if let Some(token) = document.get_token_at_position(position) {
                if token.is_initial_declaration() {
                    return Some(capabilities::go_to::to_definition_response(url, token));
                } else {
                    for document_ref in &self.documents {
                        if let Some(declared_token) = document_ref.get_declared_token(&token.name) {
                            return match Url::from_file_path(document_ref.key()) {
                                Ok(url) => Some(capabilities::go_to::to_definition_response(
                                    url,
                                    declared_token,
                                )),
                                Err(_) => None,
                            };
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_completion_items(&self, url: &Url) -> Option<Vec<CompletionItem>> {
        if let Some(document) = self.documents.get(url.path()) {
            return Some(capabilities::completion::to_completion_items(
                document.get_tokens(),
            ));
        }

        None
    }

    pub fn get_semantic_tokens(&self, url: &Url) -> Option<Vec<SemanticToken>> {
        if let Some(document) = self.documents.get(url.path()) {
            return Some(capabilities::semantic_tokens::to_semantic_tokes(
                document.get_tokens(),
            ));
        }

        None
    }

    pub fn get_symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        if let Some(document) = self.documents.get(url.path()) {
            return Some(capabilities::document_symbol::to_symbol_information(
                document.get_tokens(),
                url.clone(),
            ));
        }

        None
    }

    pub fn format_text(&self, url: &Url) -> Option<Vec<TextEdit>> {
        if let Some(document) = self.documents.get(url.path()) {
            match self.config.read() {
                std::sync::LockResult::Ok(config) => {
                    let config: SwayConfig = *config;
                    get_format_text_edits(Arc::from(document.get_text()), config.into())
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
