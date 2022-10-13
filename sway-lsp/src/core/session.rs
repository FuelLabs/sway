use crate::{
    capabilities::{
        self,
        formatting::get_format_text_edits,
        runnable::{Runnable, RunnableType},
    },
    core::{
        document::TextDocument,
        token::{Token, TokenMap, TypeDefinition},
        {traverse_parse_tree, traverse_typed_tree},
    },
    error::{DocumentError, LanguageServerError},
    utils,
};
use dashmap::DashMap;
use forc_pkg::{self as pkg};
use std::{
    path::PathBuf,
    sync::{Arc, LockResult, RwLock},
};
use sway_core::{
    language::{parsed::ParseProgram, ty},
    CompileResult,
};
use sway_types::{Ident, Spanned};
use swayfmt::Formatter;
use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, GotoDefinitionParams, GotoDefinitionResponse, Location, Position,
    Range, SymbolInformation, TextDocumentContentChangeEvent, TextEdit, Url,
};

pub type Documents = DashMap<String, TextDocument>;

#[derive(Default, Debug)]
pub struct CompiledProgram {
    pub parsed: Option<ParseProgram>,
    pub typed: Option<ty::TyProgram>,
}

#[derive(Debug)]
pub struct Session {
    pub documents: Documents,
    pub token_map: TokenMap,
    pub runnables: DashMap<RunnableType, Runnable>,
    pub compiled_program: RwLock<CompiledProgram>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            documents: DashMap::new(),
            token_map: DashMap::new(),
            runnables: DashMap::new(),
            compiled_program: RwLock::new(Default::default()),
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
            Some(type_def) => match type_def {
                TypeDefinition::TypeId(type_id) => utils::token::ident_of_type_id(type_id),
                TypeDefinition::Ident(ident) => Some(ident.clone()),
            },
            None => None,
        }
    }

    pub fn token_map(&self) -> &TokenMap {
        &self.token_map
    }

    // Document
    pub fn store_document(&self, text_document: TextDocument) -> Result<(), DocumentError> {
        let uri = text_document.get_uri().to_string();
        self.documents
            .insert(uri.clone(), text_document)
            .map_or(Ok(()), |_| {
                Err(DocumentError::DocumentAlreadyStored { path: uri })
            })
    }

    pub fn remove_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.documents
            .remove(url.path())
            .ok_or(DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|(_, text_document)| text_document)
    }

    pub fn parse_project(&self, uri: &Url) -> Result<Vec<Diagnostic>, LanguageServerError> {
        self.token_map.clear();
        self.runnables.clear();

        let manifest_dir = PathBuf::from(uri.path());
        let locked = false;
        let offline = false;

        let manifest = pkg::PackageManifestFile::from_dir(&manifest_dir).map_err(|_| {
            DocumentError::ManifestFileNotFound {
                dir: uri.path().into(),
            }
        })?;

        let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)
            .map_err(LanguageServerError::BuildPlanFailed)?;

        // We can convert these destructured elements to a Vec<Diagnostic> later on.
        let CompileResult {
            value,
            warnings,
            errors,
        } = pkg::check(&plan, true).map_err(LanguageServerError::FailedToCompile)?;

        // FIXME(Centril): Refactor parse_ast_to_tokens + parse_ast_to_typed_tokens
        // due to the new API.g
        let (parsed, typed) = match value {
            None => (None, None),
            Some((pp, tp)) => (Some(pp), tp),
        };

        // First, populate our token_map with un-typed ast nodes.
        let parsed_res = CompileResult::new(parsed, warnings.clone(), errors.clone());
        let _ = self.parse_ast_to_tokens(parsed_res);
        // Next, populate our token_map with typed ast nodes.
        let ast_res = CompileResult::new(typed, warnings, errors);

        self.parse_ast_to_typed_tokens(ast_res)
    }

    fn parse_ast_to_tokens(
        &self,
        parsed_result: CompileResult<ParseProgram>,
    ) -> Result<Vec<Diagnostic>, LanguageServerError> {
        let parse_program = parsed_result.value.ok_or_else(|| {
            let diagnostics = capabilities::diagnostic::get_diagnostics(
                parsed_result.warnings.clone(),
                parsed_result.errors.clone(),
            );
            LanguageServerError::FailedToParse { diagnostics }
        })?;

        for node in &parse_program.root.tree.root_nodes {
            traverse_parse_tree::traverse_node(node, &self.token_map);
        }

        for (_, submodule) in &parse_program.root.submodules {
            for node in &submodule.module.tree.root_nodes {
                traverse_parse_tree::traverse_node(node, &self.token_map);
            }
        }

        if let LockResult::Ok(mut program) = self.compiled_program.write() {
            program.parsed = Some(parse_program);
        }

        Ok(capabilities::diagnostic::get_diagnostics(
            parsed_result.warnings,
            parsed_result.errors,
        ))
    }

    fn parse_ast_to_typed_tokens(
        &self,
        ast_res: CompileResult<ty::TyProgram>,
    ) -> Result<Vec<Diagnostic>, LanguageServerError> {
        let typed_program = ast_res.value.ok_or(LanguageServerError::FailedToParse {
            diagnostics: capabilities::diagnostic::get_diagnostics(
                ast_res.warnings.clone(),
                ast_res.errors.clone(),
            ),
        })?;

        if let ty::TyProgramKind::Script {
            ref main_function, ..
        } = typed_program.kind
        {
            let main_fn_location = utils::common::get_range_from_span(&main_function.name.span());
            let runnable = Runnable::new(main_fn_location, typed_program.kind.tree_type());
            self.runnables.insert(RunnableType::MainFn, runnable);
        }

        let root_nodes = typed_program.root.all_nodes.iter();
        let sub_nodes = typed_program
            .root
            .submodules
            .iter()
            .flat_map(|(_, submodule)| &submodule.module.all_nodes);
        root_nodes
            .chain(sub_nodes)
            .for_each(|node| traverse_typed_tree::traverse_node(node, &self.token_map));

        if let LockResult::Ok(mut program) = self.compiled_program.write() {
            program.typed = Some(typed_program);
        }

        Ok(capabilities::diagnostic::get_diagnostics(
            ast_res.warnings,
            ast_res.errors,
        ))
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

    pub fn symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        let tokens = self.tokens_for_file(url);
        Some(capabilities::document_symbol::to_symbol_information(
            &tokens,
            url.clone(),
        ))
    }

    pub fn format_text(&self, url: &Url) -> Option<Vec<TextEdit>> {
        if let Some(document) = self.documents.get(url.path()) {
            let mut formatter = Formatter::default();
            get_format_text_edits(Arc::from(document.get_text()), &mut formatter)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils::{get_absolute_path, get_url};

    use super::*;

    #[test]
    fn store_document_returns_empty_tuple() {
        let session = Session::new();
        let path = get_absolute_path("sway-lsp/test/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).unwrap();
        let result = Session::store_document(&session, document);
        assert!(result.is_ok());
    }

    #[test]
    fn store_document_returns_document_already_stored_error() {
        let session = Session::new();
        let path = get_absolute_path("sway-lsp/test/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).unwrap();
        Session::store_document(&session, document).expect("expected successfully stored");
        let document = TextDocument::build_from_path(&path).unwrap();
        let result = Session::store_document(&session, document)
            .expect_err("expected DocumentAlreadyStored");
        assert_eq!(result, DocumentError::DocumentAlreadyStored { path });
    }

    #[test]
    fn parse_project_returns_manifest_file_not_found() {
        let session = Session::new();
        let dir = get_absolute_path("sway-lsp/test/fixtures");
        let uri = get_url(&dir);
        let result =
            Session::parse_project(&session, &uri).expect_err("expected ManifestFileNotFound");
        assert!(matches!(
            result,
            LanguageServerError::DocumentError(
                DocumentError::ManifestFileNotFound { dir: test_dir }
            )
            if test_dir == dir
        ));
    }
}
