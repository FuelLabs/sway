use crate::{
    capabilities::{
        self,
        formatting::get_format_text_edits,
        runnable::{Runnable, RunnableType},
    },
    core::{
        document::{DocumentError, TextDocument},
        token::{Token, TokenMap, TypeDefinition},
        {traverse_parse_tree, traverse_typed_tree},
    },
    sway_config::SwayConfig,
    utils,
};
use dashmap::DashMap;
use forc_pkg::{self as pkg};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::{Arc, LockResult, RwLock},
};
use sway_core::{CompileAstResult, CompileResult, ParseProgram, TypeInfo, TypedProgramKind};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, GotoDefinitionParams, GotoDefinitionResponse, Location, Position,
    Range, SemanticToken, SymbolInformation, TextDocumentContentChangeEvent, TextEdit, Url,
};

pub type Documents = DashMap<String, TextDocument>;

#[derive(Debug)]
pub struct Session {
    pub documents: Documents,
    pub config: RwLock<SwayConfig>,
    pub token_map: TokenMap,
    pub runnables: DashMap<RunnableType, Runnable>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
            config: RwLock::new(SwayConfig::default()),
            token_map: DashMap::new(),
            runnables: DashMap::new(),
        }
    }

    /// Check if the code editor's cursor is currently over one of our collected tokens.
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(Ident, Token)> {
        let tokens = self.tokens_for_file(uri);
        match utils::common::ident_and_span_at_position(position, &tokens) {
            Some((ident, _)) => {
                self.token_map
                    .get(&utils::token::to_ident_key(&ident))
                    .map(|item| {
                        let ((ident, _), token) = item.pair();
                        (ident.clone(), token.clone())
                    })
            }
            None => None,
        }
    }

    pub fn all_references_of_token(&self, token: &Token) -> Vec<(Ident, Token)> {
        let current_type_id = utils::token::type_id(token);

        self.token_map
            .iter()
            .filter(|item| {
                let ((_, _), token) = item.pair();
                if token.typed.is_some() {
                    current_type_id == utils::token::type_id(token)
                } else {
                    false
                }
            })
            .map(|item| {
                let ((ident, _), token) = item.pair();
                (ident.clone(), token.clone())
            })
            .collect()
    }

    /// Return a TokenMap with tokens belonging to the provided file path
    pub fn tokens_for_file(&self, uri: &Url) -> TokenMap {
        self.token_map
            .iter()
            .filter(|item| {
                let (_, span) = item.key();
                match span.path() {
                    Some(path) => path.to_str() == Some(uri.path()),
                    None => false,
                }
            })
            .map(|item| {
                let (key, token) = item.pair();
                (key.clone(), token.clone())
            })
            .collect()
    }

    pub fn declared_token_ident(&self, token: &Token) -> Option<Ident> {
        // Look up the tokens TypeId
        match &token.type_def {
            Some(type_def) => {
                match type_def {
                    TypeDefinition::TypeId(type_id) => {
                        // Use the TypeId to look up the actual type
                        let type_info = sway_core::type_system::look_up_type_id(*type_id);
                        match type_info {
                            TypeInfo::UnknownGeneric { name }
                            | TypeInfo::Enum { name, .. }
                            | TypeInfo::Struct { name, .. }
                            | TypeInfo::Custom { name, .. } => Some(name),
                            _ => None,
                        }
                    }
                    TypeDefinition::Ident(ident) => Some(ident.clone()),
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

    pub fn parse_project(&self, uri: &Url) -> Result<Vec<Diagnostic>, DocumentError> {
        self.token_map.clear();
        self.runnables.clear();

        let manifest_dir = PathBuf::from(uri.path());
        let silent_mode = true;
        let locked = false;
        let offline = false;

        // TODO: match on any errors and report them back to the user in a future PR
        if let Ok(manifest) = pkg::ManifestFile::from_dir(&manifest_dir) {
            if let Ok(plan) = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline) {
                //we can then use them directly to convert them to a Vec<Diagnostic>
                if let Ok((parsed_res, ast_res)) = pkg::check(&plan, silent_mode) {
                    // First, populate our token_map with un-typed ast nodes
                    let _ = self.parse_ast_to_tokens(parsed_res);
                    // Next, populate our token_map with typed ast nodes
                    let res = self.parse_ast_to_typed_tokens(ast_res);
                    //self.test_typed_parse(ast_res);
                    return res;
                }
            }
        }
        Err(DocumentError::FailedToParse(vec![]))
    }

    fn parse_ast_to_tokens(
        &self,
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
                    traverse_parse_tree::traverse_node(node, &self.token_map);
                }

                for (_, submodule) in &parse_program.root.submodules {
                    for node in &submodule.module.tree.root_nodes {
                        traverse_parse_tree::traverse_node(node, &self.token_map);
                    }
                }

                Ok(capabilities::diagnostic::get_diagnostics(
                    parsed_result.warnings,
                    parsed_result.errors,
                ))
            }
        }
    }

    fn parse_ast_to_typed_tokens(
        &self,
        ast_res: CompileAstResult,
    ) -> Result<Vec<Diagnostic>, DocumentError> {
        match ast_res {
            CompileAstResult::Failure { warnings, errors } => {
                let diagnostics = capabilities::diagnostic::get_diagnostics(warnings, errors);
                Err(DocumentError::FailedToParse(diagnostics))
            }
            CompileAstResult::Success {
                typed_program,
                warnings,
            } => {
                if let TypedProgramKind::Script {
                    ref main_function, ..
                }
                | TypedProgramKind::Predicate {
                    ref main_function, ..
                } = typed_program.kind
                {
                    let main_fn_location =
                        utils::common::get_range_from_span(&main_function.name.span());
                    let runnable = Runnable::new(main_fn_location, typed_program.kind.tree_type());
                    self.runnables.insert(RunnableType::MainFn, runnable);
                }

                for node in &typed_program.root.all_nodes {
                    traverse_typed_tree::traverse_node(node, &self.token_map);
                }

                for (_, submodule) in &typed_program.root.submodules {
                    for node in &submodule.module.all_nodes {
                        traverse_typed_tree::traverse_node(node, &self.token_map);
                    }
                }

                Ok(capabilities::diagnostic::get_diagnostics(warnings, vec![]))
            }
        }
    }

    pub fn _test_typed_parse(&mut self, _ast_res: CompileAstResult, uri: &Url) {
        for item in self.token_map.iter() {
            let ((ident, _span), token) = item.pair();
            utils::debug::debug_print_ident_and_token(ident, token);
        }

        //let cursor_position = Position::new(25, 14); //Cursor's hovered over the position var decl in main()
        let cursor_position = Position::new(29, 18); //Cursor's hovered over the ~Particle in p = decl in main()

        if let Some((_, token)) = self.token_at_position(uri, cursor_position) {
            // Look up the tokens TypeId
            if let Some(type_id) = utils::token::type_id(&token) {
                tracing::info!("type_id = {:#?}", type_id);

                // Use the TypeId to look up the actual type
                let type_info = sway_core::type_system::look_up_type_id(type_id);
                tracing::info!("type_info = {:#?}", type_info);
            }

            // Find the ident / span on the returned type

            // Contruct a go_to LSP request from the declerations span
        }
    }

    pub fn contains_sway_file(&self, url: &Url) -> bool {
        self.documents.contains_key(url.path())
    }

    pub fn handle_open_file(&self, uri: &Url) {
        if !self.contains_sway_file(uri) {
            if let Ok(text_document) = TextDocument::build_from_path(uri.path()) {
                let _ = self.store_document(text_document);
            }
        }
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
        if let Some((_, token)) = self.token_at_position(url, position) {
            let token_ranges = self
                .all_references_of_token(&token)
                .iter()
                .map(|(ident, _)| utils::common::get_range_from_span(&ident.span()))
                .collect();

            return Some(token_ranges);
        }
        None
    }

    pub fn token_definition_response(
        &self,
        params: GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let url = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        self.token_at_position(&url, position)
            .and_then(|(_, token)| self.declared_token_ident(&token))
            .and_then(|decl_ident| {
                let range = utils::common::get_range_from_span(&decl_ident.span());
                match decl_ident.span().path() {
                    Some(path) => match Url::from_file_path(path.as_ref()) {
                        Ok(url) => Some(GotoDefinitionResponse::Scalar(Location::new(url, range))),
                        Err(_) => None,
                    },
                    None => None,
                }
            })
    }

    pub fn completion_items(&self) -> Option<Vec<CompletionItem>> {
        Some(capabilities::completion::to_completion_items(
            self.token_map(),
        ))
    }

    pub fn semantic_tokens(&self, url: &Url) -> Option<Vec<SemanticToken>> {
        let tokens = self.tokens_for_file(url);
        Some(capabilities::semantic_tokens::to_semantic_tokens(&tokens))
    }

    pub fn symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        let tokens = self.tokens_for_file(url);
        Some(capabilities::document_symbol::to_symbol_information(
            &tokens,
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
