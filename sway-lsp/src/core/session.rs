
use crate::{
    core::{
        document::{DocumentError, TextDocument},
        token::{TokenMap, TokenType},
        {traverse_parse_tree, traverse_typed_tree},
    },
    capabilities::{self, formatting::get_format_text_edits},
    sway_config::SwayConfig,
    utils,
};
use forc::utils::SWAY_GIT_TAG;
use forc_pkg::{self as pkg};
use dashmap::DashMap;
use serde_json::Value;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, LockResult, RwLock},
};
use sway_core::{
    semantic_analysis::ast_node::TypedAstNode, CompileAstResult, CompileResult, ParseProgram,
    TypeInfo,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, GotoDefinitionResponse, Position, Range, SemanticToken,
    SymbolInformation, TextDocumentContentChangeEvent, TextEdit, Url,
};

pub type Documents = DashMap<String, TextDocument>;

#[derive(Debug)]
pub struct Session {
    pub documents: Documents,
    pub config: RwLock<SwayConfig>,
    pub token_map: TokenMap,
    pub manifest: Option<pkg::ManifestFile>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
            config: RwLock::new(SwayConfig::default()),
            token_map: HashMap::new(),
            manifest: None,
        }
    }

    /// Check if the code editor's cursor is currently over an of our collected tokens
    pub fn token_at_position(&self, position: Position) -> Option<(Ident, &TokenType)> {
        match utils::common::ident_and_span_at_position(position, &self.token_map) {
            Some((ident, _)) => {
                // Retrieve the TokenType from our HashMap
                self.token_map
                    .get(&utils::token::to_ident_key(&ident))
                    .map(|token| (ident.clone(), token))
            }
            None => None,
        }
    }

    pub fn all_references_of_token(&self, token: &TokenType) -> Vec<(&Ident, &TokenType)> {
        let current_type_id = utils::token::type_id(token);

        self.token_map
            .iter()
            .filter(|((_, _), token)| {
                if token.typed.is_some() {
                    current_type_id == utils::token::type_id(token)
                } else {
                    false
                }
            })
            .map(|((ident, _), token)| (ident, token))
            .collect()
    }

    pub fn declared_token_ident(&self, token: &TokenType) -> Option<Ident> {
        // Look up the tokens TypeId
        match utils::token::type_id(token) {
            Some(type_id) => {
                tracing::info!("type_id = {:#?}", type_id);

                // Use the TypeId to look up the actual type
                let type_info = sway_core::type_engine::look_up_type_id(type_id);
                tracing::info!("type_info = {:#?}", type_info);

                match type_info {
                    TypeInfo::UnknownGeneric { name }
                    | TypeInfo::Enum { name, .. }
                    | TypeInfo::Struct { name, .. }
                    | TypeInfo::Custom { name, .. } => Some(name),
                    _ => None,
                }
            }
            None => None,
        }
    }

    pub fn token_map(&self) -> &TokenMap {
        &self.token_map
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

    pub fn parse_project(&mut self, uri: Url) -> Result<Vec<Diagnostic>, DocumentError> {
        self.token_map.clear();

        let manifest_dir = PathBuf::from(uri.path());
        let silent_mode = true;
        let locked = false;
        let offline = false;

        // TODO: match on any errors and report them back to the user in a future PR
        if let Ok(manifest) = pkg::ManifestFile::from_dir(&manifest_dir, SWAY_GIT_TAG) {
            if let Ok(plan) =
                pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline, SWAY_GIT_TAG)
            {
                //TODO, pkg::check should return the warnings and errors
                //we can then use them directly to convert them to a Vec<Diagnostic> 
                if let Ok((parsed_res, ast_res)) = pkg::check(&plan, silent_mode) {
                    // First, populate our token_map with un-typed ast nodes
                    let r = self.parse_ast_to_tokens(parsed_res);
                    // Next, populate our token_map with typed ast nodes
                    let rr = self.parse_ast_to_typed_tokens(ast_res);
                    //self.test_typed_parse(ast_res);
                    return r;
                }
            }
        }

        // First, populate our token_map with un-typed ast nodes
        if let Some(document) = self.documents.get(uri.path()) {
            let text = Arc::from(document.get_text());
            match self.parse_ast_to_tokens(text) {
                Ok(diagnostics) => self.diagnostics = diagnostics,
                Err(error) => {
                    tracing::warn!("{:#?}", error);
                }
            }
        }

        // Next, populate our token_map with typed ast nodes
        match self.parse_ast_to_typed_tokens() {
            Ok(diagnostics) => self.diagnostics = diagnostics,
            Err(error) => {
                match &error {
                    ServerError::FailedToParse(diagnostics) => {
                        self.diagnostics = diagnostics.clone();
                    }
                    _ => tracing::warn!("{:#?}", error), // change this to client.log instead of tracing
                }
                if let ServerError::FailedToParse(diagnostics) = error {
                    self.diagnostics = diagnostics;
                }
            }
        }
    }

    fn parse_ast_to_tokens(
        &mut self,
        parsed_result: CompileResult<ParseProgram>,
    ) -> Result<Vec<Diagnostic>, DocumentError> {
        match parsed_result.value {
            None => {
                let diagnostics = capabilities::diagnostic::get_diagnostics(
                    parsed_result.warnings,
                    parsed_result.errors,
                );
                Err(DocumentError::FailedToParse(diagnostics))
            }
            Some(parse_program) => {
                for node in &parse_program.root.tree.root_nodes {
                    traverse_parse_tree::traverse_node(node, &mut self.token_map);
                }

                Ok(capabilities::diagnostic::get_diagnostics(
                    parsed_result.warnings,
                    parsed_result.errors,
                ))
            }
        }
    }

    fn parse_ast_to_typed_tokens(&self, ast_res: CompileAstResult) -> Option<Vec<TypedAstNode>> {
        match ast_res {
            CompileAstResult::Failure { .. } => None,
            CompileAstResult::Success { typed_program, .. } => Some(typed_program.root.all_nodes),
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
    pub fn token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        if let Some((_, token)) = self.token_at_position(position) {
            let token_ranges = self
                .all_references_of_token(token)
                .iter()
                .map(|(ident, _)| utils::common::get_range_from_span(&ident.span()))
                .collect();

            return Some(token_ranges);
        }
        None
    }

    pub fn token_definition_response(
        &self,
        url: Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let key = url.path();
        if let Some((_, token)) = self.token_at_position(position) {
            if let Some(decl_ident) = self.declared_token_ident(token) {
                return Some(capabilities::go_to::to_definition_response(
                    url,
                    &decl_ident,
                ));
            }
        }
        None
    }

    pub fn completion_items(&self, url: &Url) -> Option<Vec<CompletionItem>> {
        Some(capabilities::completion::to_completion_items(
                self.token_map(),
        ))
    }

    pub fn semantic_tokens(&self, url: &Url) -> Option<Vec<SemanticToken>> {
        Some(capabilities::semantic_tokens::to_semantic_tokens(
                self.token_map(),
        ))
    }

    pub fn symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        Some(capabilities::document_symbol::to_symbol_information(
            self.token_map(),
            url.clone(),
        ))
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
