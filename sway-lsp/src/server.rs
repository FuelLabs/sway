pub use crate::error::DocumentError;
use crate::{
    capabilities::{self, diagnostic::Diagnostics},
    config::{Config, Warnings},
    core::{session::Session, sync},
    error::{DirectoryError, LanguageServerError},
    utils::{debug, keyword_docs::KeywordDocs},
};
use dashmap::DashMap;
use forc_pkg::manifest::PackageManifestFile;
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::*;
use tower_lsp::{jsonrpc, Client, LanguageServer};
use tracing::metadata::LevelFilter;

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub config: RwLock<Config>,
    pub keyword_docs: KeywordDocs,
    sessions: DashMap<PathBuf, Arc<Session>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let sessions = DashMap::new();
        let config = RwLock::new(Default::default());
        let keyword_docs = KeywordDocs::new();

        Backend {
            client,
            config,
            keyword_docs,
            sessions,
        }
    }

    fn init(&self, uri: &Url) -> Result<(), LanguageServerError> {
        let session = Arc::new(Session::new());
        let project_name = session.init(uri)?;
        self.sessions.insert(project_name, session);
        Ok(())
    }

    fn get_uri_and_session(
        &self,
        workspace_uri: &Url,
    ) -> Result<(Url, Arc<Session>), LanguageServerError> {
        let session = self.url_to_session(workspace_uri)?;
        let uri = session.sync.workspace_to_temp_url(workspace_uri)?;
        Ok((uri, session))
    }

    async fn parse_project(&self, uri: Url, workspace_uri: Url, session: Arc<Session>) {
        // pass in the temp Url into parse_project, we can now get the updated AST's back.
        let diagnostics = match session.parse_project(&uri) {
            Ok(diagnostics) => diagnostics,
            Err(err) => {
                tracing::error!("{}", err.to_string().as_str());
                if let LanguageServerError::FailedToParse { diagnostics } = err {
                    diagnostics
                } else {
                    Diagnostics {
                        warnings: vec![],
                        errors: vec![],
                    }
                }
            }
        };
        self.publish_diagnostics(&uri, &workspace_uri, session, diagnostics)
            .await;
    }
}

/// Returns the capabilities of the server to the client,
/// indicating its support for various language server protocol features.
pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: capabilities::semantic_tokens::SUPPORTED_TYPES.to_vec(),
                    token_modifiers: capabilities::semantic_tokens::SUPPORTED_MODIFIERS.to_vec(),
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: None,
                ..Default::default()
            }
            .into(),
        ),
        document_symbol_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: None,
            ..Default::default()
        }),
        document_formatting_provider: Some(OneOf::Left(true)),
        definition_provider: Some(OneOf::Left(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(false),
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..ServerCapabilities::default()
    }
}

impl Backend {
    fn url_to_session(&self, uri: &Url) -> Result<Arc<Session>, LanguageServerError> {
        let path = PathBuf::from(uri.path());
        let manifest = PackageManifestFile::from_dir(&path).map_err(|_| {
            DocumentError::ManifestFileNotFound {
                dir: path.to_string_lossy().to_string(),
            }
        })?;

        // strip Forc.toml from the path to get the manifest directory
        let manifest_dir = manifest
            .path()
            .parent()
            .ok_or(DirectoryError::ManifestDirNotFound)?
            .to_path_buf();

        let session = match self.sessions.try_get(&manifest_dir).try_unwrap() {
            Some(item) => item.value().clone(),
            None => {
                // If no session can be found, then we need to call init and inserst a new session into the map
                self.init(uri)?;
                self.sessions
                    .try_get(&manifest_dir)
                    .try_unwrap()
                    .map(|item| item.value().clone())
                    .expect("no session found even though it was just inserted into the map")
            }
        };

        Ok(session)
    }

    async fn publish_diagnostics(
        &self,
        uri: &Url,
        workspace_uri: &Url,
        session: Arc<Session>,
        diagnostics: Diagnostics,
    ) {
        let diagnostics_res = {
            let mut diagnostics_to_publish = vec![];
            let config = &self.config.read();
            let tokens = session.token_map().tokens_for_file(uri);
            match config.debug.show_collected_tokens_as_warnings {
                // If collected_tokens_as_warnings is Parsed or Typed,
                // take over the normal error and warning display behavior
                // and instead show the either the parsed or typed tokens as warnings.
                // This is useful for debugging the lsp parser.
                Warnings::Parsed => diagnostics_to_publish
                    .extend(debug::generate_warnings_for_parsed_tokens(tokens)),
                Warnings::Typed => {
                    diagnostics_to_publish.extend(debug::generate_warnings_for_typed_tokens(tokens))
                }
                Warnings::Default => {}
            }
            if config.diagnostic.show_warnings {
                diagnostics_to_publish.extend(diagnostics.warnings);
            }
            if config.diagnostic.show_errors {
                diagnostics_to_publish.extend(diagnostics.errors);
            }
            diagnostics_to_publish
        };

        // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
        // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
        self.client
            .publish_diagnostics(workspace_uri.clone(), diagnostics_res, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        if let Some(initialization_options) = &params.initialization_options {
            let mut config = self.config.write();
            *config = serde_json::from_value(initialization_options.clone())
                .ok()
                .unwrap_or_default();
        }

        // Initalizing tracing library based on the user's config
        let config = self.config.read();
        if config.logging.level != LevelFilter::OFF {
            let tracing_options = TracingSubscriberOptions {
                log_level: Some(config.logging.level),
                writer_mode: Some(TracingWriterMode::Stderr),
                ..Default::default()
            };
            init_tracing_subscriber(tracing_options);
        }

        tracing::info!("Initializing the Sway Language Server");

        Ok(InitializeResult {
            server_info: None,
            capabilities: capabilities(),
            ..InitializeResult::default()
        })
    }

    // LSP-Server Lifecycle
    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("Sway Language Server Initialized");
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        tracing::info!("Shutting Down the Sway Language Server");

        let _ = self.sessions.iter().map(|item| {
            let session = item.value();
            session.shutdown();
        });

        Ok(())
    }

    // Document Handlers
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                session.handle_open_file(&uri);
                self.parse_project(uri, params.text_document.uri, session.clone())
                    .await;
            }
            Err(err) => tracing::error!("{}", err.to_string()),
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let config = self.config.read().on_enter.clone();
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                // handle on_enter capabilities if they are enabled
                capabilities::on_enter(&config, &self.client, &session, &uri.clone(), &params)
                    .await;

                // update this file with the new changes and write to disk
                match session.write_changes_to_file(&uri, params.content_changes) {
                    Ok(_) => {
                        self.parse_project(uri, params.text_document.uri.clone(), session.clone())
                            .await;
                    }
                    Err(err) => tracing::error!("{}", err.to_string()),
                }
            }
            Err(err) => tracing::error!("{}", err.to_string()),
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                // overwrite the contents of the tmp/folder with everything in
                // the current workspace. (resync)
                if let Err(err) = session.sync.clone_manifest_dir_to_temp() {
                    tracing::error!("{}", err.to_string().as_str());
                }

                let _ = session
                    .sync
                    .manifest_path()
                    .and_then(|manifest_path| PackageManifestFile::from_dir(&manifest_path).ok())
                    .map(|manifest| {
                        if let Some(temp_manifest_path) = &session.sync.temp_manifest_path() {
                            sync::edit_manifest_dependency_paths(&manifest, temp_manifest_path)
                        }
                    });
                self.parse_project(uri, params.text_document.uri, session)
                    .await;
            }
            Err(err) => tracing::error!("{}", err.to_string()),
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for event in params.changes {
            if event.typ == FileChangeType::DELETED {
                match self.get_uri_and_session(&event.uri) {
                    Ok((uri, session)) => {
                        let _ = session.remove_document(&uri);
                    }
                    Err(err) => tracing::error!("{}", err.to_string()),
                }
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        match self.get_uri_and_session(&params.text_document_position_params.text_document.uri) {
            Ok((uri, session)) => {
                let position = params.text_document_position_params.position;
                Ok(capabilities::hover::hover_data(
                    session,
                    &self.keyword_docs,
                    uri,
                    position,
                ))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((temp_uri, session)) => Ok(capabilities::code_actions(
                session,
                &params.range,
                params.text_document,
                &temp_uri,
            )),
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn code_lens(&self, params: CodeLensParams) -> jsonrpc::Result<Option<Vec<CodeLens>>> {
        let mut result = vec![];
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((_, session)) => {
                // Construct code lenses for runnable functions
                session.runnables.iter().for_each(|item| {
                    let runnable = item.value();
                    result.push(CodeLens {
                        range: runnable.range(),
                        command: Some(runnable.command()),
                        data: None,
                    });
                });
                Ok(Some(result))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        match self.get_uri_and_session(&params.text_document_position.text_document.uri) {
            Ok((_, session)) => {
                // TODO
                // here we would also need to provide a list of builtin methods not just the ones from the document
                Ok(session.completion_items().map(CompletionResponse::Array))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => Ok(session
                .symbol_information(&uri)
                .map(DocumentSymbolResponse::Flat)),
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> jsonrpc::Result<Option<SemanticTokensResult>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => Ok(capabilities::semantic_tokens::semantic_tokens_full(
                session, &uri,
            )),
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        match self.get_uri_and_session(&params.text_document_position_params.text_document.uri) {
            Ok((uri, session)) => {
                let position = params.text_document_position_params.position;
                Ok(capabilities::highlight::get_highlights(
                    session, uri, position,
                ))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        match self.get_uri_and_session(&params.text_document_position_params.text_document.uri) {
            Ok((uri, session)) => {
                let position = params.text_document_position_params.position;
                Ok(session.token_definition_response(uri, position))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        self.get_uri_and_session(&params.text_document.uri)
            .and_then(|(uri, session)| session.format_text(&uri).map(Some))
            .or_else(|err| {
                tracing::error!("{}", err.to_string());
                Ok(None)
            })
    }

    async fn rename(&self, params: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        match self.get_uri_and_session(&params.text_document_position.text_document.uri) {
            Ok((uri, session)) => {
                let new_name = params.new_name;
                let position = params.text_document_position.position;
                Ok(capabilities::rename::rename(
                    session, new_name, uri, position,
                ))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> jsonrpc::Result<Option<PrepareRenameResponse>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                let position = params.position;
                Ok(capabilities::rename::prepare_rename(session, uri, position))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
    pub save_path: Url,
}

// Custom LSP-Server Methods
impl Backend {
    pub async fn inlay_hints(
        &self,
        params: InlayHintParams,
    ) -> jsonrpc::Result<Option<Vec<InlayHint>>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                let config = &self.config.read().inlay_hints;
                Ok(capabilities::inlay_hints::inlay_hints(
                    session,
                    &uri,
                    &params.range,
                    config,
                ))
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    /// This method is triggered by a command palette request in VScode
    /// The 3 commands are: "show lexed ast", "show parsed ast" or "show typed ast"
    ///
    /// If any of these commands are executed, the client requests this method
    /// by calling the "sway/show_ast".
    ///
    /// The function expects the URI of the current open file where the
    /// request was made, and if the "lexed", "parsed" or "typed" ast was requested.
    ///
    /// A formatted AST is written to a temporary file and the URI is
    /// returned to the client so it can be opened and displayed in a
    /// seperate side panel.
    pub async fn show_ast(
        &self,
        params: ShowAstParams,
    ) -> jsonrpc::Result<Option<TextDocumentIdentifier>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((_, session)) => {
                let current_open_file = params.text_document.uri;
                // Convert the Uri to a PathBuf
                let path = current_open_file.to_file_path().ok();

                let write_ast_to_file =
                    |path: &Path, ast_string: &String| -> Option<TextDocumentIdentifier> {
                        if let Ok(mut file) = File::create(path) {
                            let _ = writeln!(&mut file, "{ast_string}");
                            if let Ok(uri) = Url::from_file_path(path) {
                                // Return the tmp file path where the AST has been written to.
                                return Some(TextDocumentIdentifier::new(uri));
                            }
                        }
                        None
                    };

                // Returns true if the current path matches the path of a submodule
                let path_is_submodule = |ident: &Ident, path: &Option<PathBuf>| -> bool {
                    ident.span().path().map(|a| a.deref()) == path.as_ref()
                };

                let ast_path = PathBuf::from(params.save_path.path());
                {
                    let program = session.compiled_program.read();
                    match params.ast_kind.as_str() {
                        "lexed" => {
                            Ok(program.lexed.as_ref().and_then(|lexed_program| {
                                let mut formatted_ast = format!("{:#?}", program.lexed);
                                for (ident, submodule) in &lexed_program.root.submodules {
                                    if path_is_submodule(ident, &path) {
                                        // overwrite the root AST with the submodule AST
                                        formatted_ast = format!("{:#?}", submodule.module.tree);
                                    }
                                }
                                write_ast_to_file(
                                    ast_path.join("lexed.rs").as_path(),
                                    &formatted_ast,
                                )
                            }))
                        }
                        "parsed" => {
                            Ok(program.parsed.as_ref().and_then(|parsed_program| {
                                // Initialize the string with the AST from the root
                                let mut formatted_ast =
                                    format!("{:#?}", parsed_program.root.tree.root_nodes);
                                for (ident, submodule) in &parsed_program.root.submodules {
                                    if path_is_submodule(ident, &path) {
                                        // overwrite the root AST with the submodule AST
                                        formatted_ast =
                                            format!("{:#?}", submodule.module.tree.root_nodes);
                                    }
                                }
                                write_ast_to_file(
                                    ast_path.join("parsed.rs").as_path(),
                                    &formatted_ast,
                                )
                            }))
                        }
                        "typed" => {
                            Ok(program.typed.as_ref().and_then(|typed_program| {
                                // Initialize the string with the AST from the root
                                let mut formatted_ast = debug::print_decl_engine_types(
                                    &typed_program.root.all_nodes,
                                    &session.decl_engine.read(),
                                );
                                for (ident, submodule) in &typed_program.root.submodules {
                                    if path_is_submodule(ident, &path) {
                                        // overwrite the root AST with the submodule AST
                                        formatted_ast = debug::print_decl_engine_types(
                                            &submodule.module.all_nodes,
                                            &session.decl_engine.read(),
                                        );
                                    }
                                }
                                write_ast_to_file(
                                    ast_path.join("typed.rs").as_path(),
                                    &formatted_ast,
                                )
                            }))
                        }
                        _ => Ok(None),
                    }
                }
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }
}
