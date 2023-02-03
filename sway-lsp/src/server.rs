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

fn capabilities() -> ServerCapabilities {
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
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((uri, session)) => {
                // update this file with the new changes and write to disk
                match session.write_changes_to_file(&uri, params.content_changes) {
                    Ok(_) => {
                        self.parse_project(uri, params.text_document.uri, session.clone())
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
                self.parse_project(uri, params.text_document.uri, session.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::{
        assert_server_requests, dir_contains_forc_manifest, doc_comments_dir, e2e_language_dir,
        e2e_test_dir, get_fixture, runnables_test_dir, sway_workspace_dir, test_fixtures_dir,
    };
    use assert_json_diff::assert_json_eq;
    use serde_json::json;
    use std::{borrow::Cow, fs, io::Read, path::PathBuf};
    use tower::{Service, ServiceExt};
    use tower_lsp::{
        jsonrpc::{self, Id, Request, Response},
        ExitedError, LspService,
    };

    /// Holds the information needed to check the response of a goto definition request.
    struct GotoDefintion<'a> {
        req_uri: &'a Url,
        req_line: i32,
        req_char: i32,
        def_line: i32,
        def_start_char: i32,
        def_end_char: i32,
        def_path: &'a str,
    }

    fn load_sway_example(manifest_dir: PathBuf) -> (Url, String) {
        let src_path = manifest_dir.join("src/main.sw");
        let mut file = fs::File::open(&src_path).unwrap();
        let mut sway_program = String::new();
        file.read_to_string(&mut sway_program).unwrap();

        let uri = Url::from_file_path(src_path).unwrap();

        (uri, sway_program)
    }

    fn build_request_with_id(
        method: impl Into<Cow<'static, str>>,
        params: serde_json::Value,
        id: impl Into<Id>,
    ) -> Request {
        Request::build(method).params(params).id(id).finish()
    }

    async fn call_request(
        service: &mut LspService<Backend>,
        req: Request,
    ) -> Result<Option<Response>, ExitedError> {
        service.ready().await?.call(req).await
    }

    async fn initialize_request(service: &mut LspService<Backend>) -> Request {
        let params = json!({ "capabilities": capabilities() });
        let initialize = build_request_with_id("initialize", params, 1);
        let response = call_request(service, initialize.clone()).await;
        let expected = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_json_eq!(expected, response.ok().unwrap());
        initialize
    }

    async fn initialized_notification(service: &mut LspService<Backend>) {
        let initialized = Request::build("initialized").finish();
        let response = call_request(service, initialized).await;
        assert_eq!(response, Ok(None));
    }

    async fn shutdown_request(service: &mut LspService<Backend>) -> Request {
        let shutdown = Request::build("shutdown").id(1).finish();
        let response = call_request(service, shutdown.clone()).await;
        let expected = Response::from_ok(1.into(), json!(null));
        assert_json_eq!(expected, response.ok().unwrap());
        shutdown
    }

    async fn exit_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("exit").finish();
        let response = call_request(service, exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn did_open_notification(service: &mut LspService<Backend>, uri: &Url, text: &str) {
        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": "sway",
                "version": 1,
                "text": text,
            },
        });

        let did_open = Request::build("textDocument/didOpen")
            .params(params)
            .finish();
        let response = call_request(service, did_open).await;
        assert_eq!(response, Ok(None));
    }

    async fn did_change_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
                "version": 2
            },
            "contentChanges": [
                {
                    "range": {
                        "start": {
                            "line": 1,
                            "character": 0
                        },
                        "end": {
                            "line": 1,
                            "character": 0
                        }
                    },
                    "rangeLength": 0,
                    "text": "\n",
                }
            ]
        });
        let did_change = Request::build("textDocument/didChange")
            .params(params)
            .finish();
        let response = call_request(service, did_change.clone()).await;
        assert_eq!(response, Ok(None));
        did_change
    }

    async fn did_close_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("textDocument/didClose").finish();
        let response = call_request(service, exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn show_ast_request(
        service: &mut LspService<Backend>,
        uri: &Url,
        ast_kind: &str,
        save_path: Option<Url>,
    ) -> Request {
        // The path where the AST will be written to.
        // If no path is provided, the default path is "/tmp"
        let save_path = match save_path {
            Some(path) => path,
            None => Url::from_file_path(Path::new("/tmp")).unwrap(),
        };
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "astKind": ast_kind,
            "savePath": save_path,
        });
        let show_ast = build_request_with_id("sway/show_ast", params, 1);
        let response = call_request(service, show_ast.clone()).await;
        let expected = Response::from_ok(
            1.into(),
            json!({ "uri": format!("{save_path}/{ast_kind}.rs") }),
        );
        assert_json_eq!(expected, response.ok().unwrap());
        show_ast
    }

    async fn semantic_tokens_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let semantic_tokens = build_request_with_id("textDocument/semanticTokens/full", params, 1);
        let _response = call_request(service, semantic_tokens.clone()).await;
        semantic_tokens
    }

    async fn document_symbol_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let document_symbol = build_request_with_id("textDocument/documentSymbol", params, 1);
        let _response = call_request(service, document_symbol.clone()).await;
        document_symbol
    }

    fn definition_request(uri: &Url, token_line: i32, token_char: i32, id: i64) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": token_line,
                "character": token_char,
            }
        });
        build_request_with_id("textDocument/definition", params, id)
    }

    async fn definition_check<'a>(
        service: &mut LspService<Backend>,
        go_to: &'a GotoDefintion<'a>,
        id: i64,
    ) -> Request {
        let definition = definition_request(go_to.req_uri, go_to.req_line, go_to.req_char, id);
        let response = call_request(service, definition.clone())
            .await
            .unwrap()
            .unwrap();
        let value = response.result().unwrap().clone();
        if let GotoDefinitionResponse::Scalar(response) = serde_json::from_value(value).unwrap() {
            let uri = response.uri.as_str();
            let range = json!({
                "end": {
                    "character": go_to.def_end_char,
                    "line": go_to.def_line,
                },
                "start": {
                    "character": go_to.def_start_char,
                    "line": go_to.def_line,
                }
            });
            assert_json_eq!(response.range, range);
            assert!(
                uri.ends_with(go_to.def_path),
                "{} doesn't end with {}",
                uri,
                go_to.def_path,
            );
        } else {
            panic!("Expected GotoDefinitionResponse::Scalar");
        }
        definition
    }

    async fn hover_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": 44,
                "character": 24
            }
        });
        let hover = build_request_with_id("textDocument/hover", params, 1);
        let response = call_request(service, hover.clone()).await;
        let expected = Response::from_ok(
            1.into(),
            json!({
                "contents": {
                    "kind": "markdown",
                    "value": "```sway\nstruct Data\n```\n---\n Struct holding:\n\n 1. A `value` of type `NumberOrString`\n 2. An `address` of type `u64`"
                },
                "range": {
                    "end": {
                        "character": 27,
                        "line": 44
                    },
                    "start": {
                        "character": 23,
                        "line": 44
                    }
                }
            }),
        );
        assert_json_eq!(expected, response.ok().unwrap());
        hover
    }

    async fn format_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "options": {
                "tabSize": 4,
                "insertSpaces": true
            },
        });
        let formatting = build_request_with_id("textDocument/formatting", params, 1);
        let _response = call_request(service, formatting.clone()).await;
        formatting
    }

    async fn highlight_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": 45,
                "character": 37
            }
        });
        let highlight = build_request_with_id("textDocument/documentHighlight", params, 1);
        let response = call_request(service, highlight.clone()).await;
        let expected = Response::from_ok(
            1.into(),
            json!([{
                    "range": {
                        "end": {
                            "character": 41,
                            "line": 45
                        },
                        "start": {
                            "character": 35,
                            "line": 45
                        }
                    }
                }
            ]),
        );
        assert_json_eq!(expected, response.ok().unwrap());
        highlight
    }

    async fn code_action_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "range" : {
                "start":{
                    "line": 27,
                    "character": 4
                },
                "end":{
                    "line": 27,
                    "character": 9
                },
            },
            "context": {
                "diagnostics": [],
                "triggerKind": 2
            }
        });
        let code_action = build_request_with_id("textDocument/codeAction", params, 1);
        let response = call_request(service, code_action.clone()).await;
        let uri_string = uri.to_string();
        let expected = Response::from_ok(
            1.into(),
            json!([{
                "data": uri,
                "edit": {
                  "changes": {
                    uri_string: [
                      {
                        "newText": "\nimpl FooABI for Contract {\n    /// This is the `main` method on the `FooABI` abi\n    fn main() -> u64 {}\n}\n",
                        "range": {
                          "end": {
                            "character": 0,
                            "line": 31
                          },
                          "start": {
                            "character": 0,
                            "line": 31
                          }
                        }
                      }
                    ]
                  }
                },
                "kind": "refactor",
                "title": "Generate impl for contract"
            }]),
        );
        assert_json_eq!(expected, response.ok().unwrap());
        code_action
    }

    async fn code_lens_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let code_lens = build_request_with_id("textDocument/codeLens", params, 1);
        let response = call_request(service, code_lens.clone()).await;
        let actual_results = response
            .unwrap()
            .unwrap()
            .into_parts()
            .1
            .ok()
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        let expected_results = vec![
            json!({
              "command": {
                "arguments": [
                  {
                    "name": "test_bar"
                  }
                ],
                "command": "sway.runTests",
                "title": "▶︎ Run Test"
              },
              "range": {
                "end": {
                  "character": 7,
                  "line": 11
                },
                "start": {
                  "character": 0,
                  "line": 11
                }
              }
            }),
            json!({
              "command": {
                "arguments": [
                  {
                    "name": "test_foo"
                  }
                ],
                "command": "sway.runTests",
                "title": "▶︎ Run Test"
              },
              "range": {
                "end": {
                  "character": 7,
                  "line": 6
                },
                "start": {
                  "character": 0,
                  "line": 6
                }
              }
            }),
            json!({
              "command": {
                "command": "sway.runScript",
                "title": "▶︎ Run"
              },
              "range": {
                "end": {
                  "character": 7,
                  "line": 2
                },
                "start": {
                  "character": 3,
                  "line": 2
                }
              }
            }),
        ];

        assert_eq!(actual_results.len(), expected_results.len());
        for expected in expected_results.iter() {
            assert!(
                actual_results.contains(expected),
                "Expected {actual_results:?} to contain {expected:?}"
            );
        }
        code_lens
    }

    async fn init_and_open(service: &mut LspService<Backend>, manifest_dir: PathBuf) -> Url {
        let _ = initialize_request(service).await;
        initialized_notification(service).await;
        let (uri, sway_program) = load_sway_example(manifest_dir);
        did_open_notification(service, &uri, &sway_program).await;
        uri
    }

    async fn shutdown_and_exit(service: &mut LspService<Backend>) {
        let _ = shutdown_request(service).await;
        exit_notification(service).await;
    }

    // This method iterates over all of the examples in the e2e langauge should_pass dir
    // and saves the lexed, parsed, and typed ASTs to the users home directory.
    // This makes it easy to grep for certain compiler types to inspect their use cases,
    // providing necessary context when working on the traversal modules.
    #[allow(unused)]
    //#[tokio::test]
    async fn write_all_example_asts() {
        let (mut service, _) = LspService::build(Backend::new)
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;

        let ast_folder = dirs::home_dir()
            .expect("could not get users home directory")
            .join("sway_asts");
        let _ = fs::create_dir(&ast_folder);
        let e2e_dir = sway_workspace_dir().join(e2e_language_dir());
        let mut entries = fs::read_dir(&e2e_dir)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.
        entries.sort();

        for entry in entries {
            let manifest_dir = entry;
            let example_name = manifest_dir.file_name().unwrap();
            if manifest_dir.is_dir() {
                let example_dir = ast_folder.join(example_name);
                if !dir_contains_forc_manifest(manifest_dir.as_path()) {
                    continue;
                }
                match fs::create_dir(&example_dir) {
                    Ok(_) => (),
                    Err(_) => continue,
                }

                let example_dir = Some(Url::from_file_path(example_dir).unwrap());
                let (uri, sway_program) = load_sway_example(manifest_dir);
                did_open_notification(&mut service, &uri, &sway_program).await;
                let _ = show_ast_request(&mut service, &uri, "lexed", example_dir.clone()).await;
                let _ = show_ast_request(&mut service, &uri, "parsed", example_dir.clone()).await;
                let _ = show_ast_request(&mut service, &uri, "typed", example_dir).await;
            }
        }
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn initialize() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
    }

    #[tokio::test]
    async fn initialized() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
    }

    #[tokio::test]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(Backend::new);
        let initialize = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let response = call_request(&mut service, initialize).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn shutdown() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn refuses_requests_after_shutdown() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn did_open() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn did_close() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        did_close_notification(&mut service).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn did_change() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let _ = did_change_request(&mut service, &uri).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn lsp_syncs_with_workspace_edits() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            def_line: 19,
            def_start_char: 7,
            def_end_char: 11,
            def_path: uri.as_str(),
        };
        let _ = definition_check(&mut service, &go_to, 1).await;
        let _ = did_change_request(&mut service, &uri).await;
        go_to.def_line = 20;
        definition_check_with_req_offset(&mut service, &mut go_to, 45, 24, 2).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn show_ast() {
        let (mut service, _) = LspService::build(Backend::new)
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();

        let uri = init_and_open(&mut service, e2e_test_dir()).await;
        let _ = show_ast_request(&mut service, &uri, "typed", None).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            def_line: 19,
            def_start_char: 7,
            def_end_char: 11,
            def_path: uri.as_str(),
        };
        let _ = definition_check(&mut service, &go_to, 1).await;
        shutdown_and_exit(&mut service).await;
    }

    async fn definition_check_with_req_offset<'a>(
        service: &mut LspService<Backend>,
        go_to: &mut GotoDefintion<'a>,
        req_line: i32,
        req_char: i32,
        id: i64,
    ) {
        go_to.req_line = req_line;
        go_to.req_char = req_char;
        let _ = definition_check(service, go_to, id).await;
    }

    #[tokio::test]
    async fn go_to_definition_inside_turbofish() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens").join("turbofish"),
        )
        .await;

        let mut opt_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 15,
            req_char: 12,
            def_line: 80,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/option.sw",
        };
        // option.sw
        let _ = definition_check(&mut service, &opt_go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 16, 17, 2).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 17, 29, 3).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 18, 19, 4).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 20, 13, 5).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 21, 19, 6).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 22, 29, 7).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 23, 18, 8).await;

        let mut res_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 20,
            req_char: 19,
            def_line: 60,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/result.sw",
        };
        // result.sw
        let _ = definition_check(&mut service, &res_go_to, 9).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 21, 25, 10).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 22, 36, 11).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 23, 27, 12).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_paths() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens").join("paths"),
        )
        .await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 8,
            req_char: 13,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 11,
            def_path: "sway-lib-std/src/lib.sw",
        };
        // std
        let _ = definition_check(&mut service, &go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 10, 14, 2).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 16, 5, 3).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 22, 13, 4).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 8,
            req_char: 19,
            def_line: 74,
            def_start_char: 8,
            def_end_char: 14,
            def_path: "sway-lib-std/src/option.sw",
        };
        // option
        let _ = definition_check(&mut service, &go_to, 5).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 8,
            req_char: 27,
            def_line: 80,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/option.sw",
        };
        // Option
        let _ = definition_check(&mut service, &go_to, 6).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 9, 14, 7).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 17,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 10,
            def_path: "sway-lib-std/src/vm/mod.sw",
        };
        // vm
        let _ = definition_check(&mut service, &go_to, 8).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 22,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 11,
            def_path: "sway-lib-std/src/vm/evm/mod.sw",
        };
        // evm
        let _ = definition_check(&mut service, &go_to, 9).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 27,
            def_line: 1,
            def_start_char: 8,
            def_end_char: 19,
            def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
        };
        // evm_address
        let _ = definition_check(&mut service, &go_to, 10).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 42,
            def_line: 7,
            def_start_char: 11,
            def_end_char: 21,
            def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
        };
        // EvmAddress
        let _ = definition_check(&mut service, &go_to, 11).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 14,
            req_char: 6,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 16,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // test_mod
        let _ = definition_check(&mut service, &go_to, 12).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 20, 7, 13).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 14,
            req_char: 16,
            def_line: 2,
            def_start_char: 7,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // test_fun
        let _ = definition_check(&mut service, &go_to, 14).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 15,
            req_char: 8,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 16,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod.sw",
        };
        // deep_mod
        let _ = definition_check(&mut service, &go_to, 15).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 15,
            req_char: 18,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 18,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
        };
        // deeper_mod
        let _ = definition_check(&mut service, &go_to, 16).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 15,
            req_char: 29,
            def_line: 2,
            def_start_char: 7,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
        };
        // deep_fun
        let _ = definition_check(&mut service, &go_to, 17).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 16,
            req_char: 11,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 14,
            def_path: "sway-lib-std/src/assert.sw",
        };
        // assert
        let _ = definition_check(&mut service, &go_to, 18).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 13,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 12,
            def_path: "sway-lib-core/src/lib.sw",
        };
        // core
        let _ = definition_check(&mut service, &go_to, 19).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 21,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 18,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // primitives
        let _ = definition_check(&mut service, &go_to, 20).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 23, 20, 21).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 4,
            def_line: 6,
            def_start_char: 5,
            def_end_char: 6,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // A
        let _ = definition_check(&mut service, &go_to, 22).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 20, 14, 23).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 7,
            def_line: 7,
            def_start_char: 11,
            def_end_char: 14,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // fun
        let _ = definition_check(&mut service, &go_to, 24).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 20, 18, 25).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 22,
            req_char: 20,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 17,
            def_path: "sway-lib-std/src/constants.sw",
        };
        // constants
        let _ = definition_check(&mut service, &go_to, 26).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 22,
            req_char: 31,
            def_line: 5,
            def_start_char: 10,
            def_end_char: 19,
            def_path: "sway-lib-std/src/constants.sw",
        };
        // ZERO_B256
        let _ = definition_check(&mut service, &go_to, 27).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 31,
            def_line: 2,
            def_start_char: 5,
            def_end_char: 8,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // u64
        let _ = definition_check(&mut service, &go_to, 28).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 11,
            req_char: 17,
            def_line: 74,
            def_start_char: 5,
            def_end_char: 9,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // b256
        let _ = definition_check(&mut service, &go_to, 29).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 23, 31, 30).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_traits() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens").join("traits"),
        )
        .await;

        let mut trait_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 6,
            req_char: 10,
            def_line: 2,
            def_start_char: 10,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/traits/src/traits.sw",
        };

        let _ = definition_check(&mut service, &trait_go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut trait_go_to, 7, 10, 2).await;
        definition_check_with_req_offset(&mut service, &mut trait_go_to, 10, 6, 3).await;
        trait_go_to.req_line = 7;
        trait_go_to.req_char = 20;
        trait_go_to.def_line = 3;
        let _ = definition_check(&mut service, &trait_go_to, 3).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_variables() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens").join("variables"),
        )
        .await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 23,
            req_char: 26,
            def_line: 22,
            def_start_char: 8,
            def_end_char: 17,
            def_path: uri.as_str(),
        };
        // Variable expressions
        let _ = definition_check(&mut service, &go_to, 1).await;

        // Function arguments
        go_to.def_line = 23;
        definition_check_with_req_offset(&mut service, &mut go_to, 28, 35, 2).await;

        // Struct fields
        go_to.def_line = 22;
        definition_check_with_req_offset(&mut service, &mut go_to, 31, 45, 3).await;

        // Enum fields
        go_to.def_line = 22;
        definition_check_with_req_offset(&mut service, &mut go_to, 34, 39, 4).await;

        // Tuple elements
        go_to.def_line = 24;
        definition_check_with_req_offset(&mut service, &mut go_to, 37, 20, 5).await;

        // Array elements
        go_to.def_line = 25;
        definition_check_with_req_offset(&mut service, &mut go_to, 40, 20, 6).await;

        // Scoped declarations
        go_to.def_line = 44;
        go_to.def_start_char = 12;
        go_to.def_end_char = 21;
        definition_check_with_req_offset(&mut service, &mut go_to, 45, 13, 7).await;

        // If let scopes
        go_to.def_line = 50;
        go_to.def_start_char = 38;
        go_to.def_end_char = 39;
        definition_check_with_req_offset(&mut service, &mut go_to, 50, 47, 8).await;

        // Shadowing
        go_to.def_line = 50;
        go_to.def_start_char = 8;
        go_to.def_end_char = 17;
        definition_check_with_req_offset(&mut service, &mut go_to, 53, 29, 9).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn publish_diagnostics_dead_code_warning() {
        let (mut service, socket) = LspService::new(Backend::new);
        let fixture = get_fixture(test_fixtures_dir().join("diagnostics/dead_code/expected.json"));
        let expected_requests = vec![fixture];
        let socket_handle = assert_server_requests(socket, expected_requests, None).await;
        let _ = init_and_open(
            &mut service,
            test_fixtures_dir().join("diagnostics/dead_code"),
        )
        .await;
        socket_handle
            .await
            .unwrap_or_else(|e| panic!("Test failed: {e:?}"));
        shutdown_and_exit(&mut service).await;
    }

    // This macro allows us to spin up a server / client for testing
    // It initializes and performs the necessary handshake and then loads
    // the sway example that was passed into `example_dir`.
    // It then runs the specific capability to test before gracefully shutting down.
    // The capability argument is an async function.
    macro_rules! test_lsp_capability {
        ($example_dir:expr, $capability:expr) => {{
            let (mut service, _) = LspService::new(Backend::new);
            let uri = init_and_open(&mut service, $example_dir).await;
            // Call the specific LSP capability function that was passed in.
            let _ = $capability(&mut service, &uri).await;
            shutdown_and_exit(&mut service).await;
        }};
    }

    macro_rules! lsp_capability_test {
        ($test:ident, $capability:expr, $dir:expr) => {
            #[tokio::test]
            async fn $test() {
                test_lsp_capability!($dir(), $capability);
            }
        };
    }

    lsp_capability_test!(semantic_tokens, semantic_tokens_request, doc_comments_dir);
    lsp_capability_test!(document_symbol, document_symbol_request, doc_comments_dir);
    lsp_capability_test!(format, format_request, doc_comments_dir);
    lsp_capability_test!(hover, hover_request, doc_comments_dir);
    lsp_capability_test!(highlight, highlight_request, doc_comments_dir);
    lsp_capability_test!(code_action, code_action_request, doc_comments_dir);
    lsp_capability_test!(code_lens, code_lens_request, runnables_test_dir);
}
