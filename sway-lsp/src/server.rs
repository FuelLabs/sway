pub use crate::error::DocumentError;
use crate::{
    capabilities,
    core::{
        config::{Config, Warnings},
        session::Session,
    },
    error::{DirectoryError, LanguageServerError},
    utils::{debug, sync},
};
use dashmap::DashMap;
use forc_pkg::manifest::PackageManifestFile;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_types::Spanned;
use tower_lsp::lsp_types::*;
use tower_lsp::{jsonrpc, Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub config: RwLock<Config>,
    sessions: DashMap<PathBuf, Arc<Session>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let sessions = DashMap::new();
        let config = RwLock::new(Default::default());

        Backend {
            client,
            config,
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
                    vec![]
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

        let session = match self.sessions.get(&manifest_dir) {
            Some(item) => item.value().clone(),
            None => {
                // TODO, remove this once the type engine no longer uses global memory: https://github.com/FuelLabs/sway/issues/2063
                // Until then, we clear the current project and init with the new project.
                // At least allows the user to switch between projects within a workspace but only if they
                // have one pane open.
                let _ = self.sessions.iter().map(|item| {
                    let session = item.value();
                    session.shutdown();
                });
                self.sessions.clear();

                // If no session can be found, then we need to call init and inserst a new session into the map
                self.init(uri)?;
                self.sessions
                    .get(&manifest_dir)
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
        diagnostics: Vec<Diagnostic>,
    ) {
        let diagnostics_res = {
            let debug = &self.config.read().debug;
            let tokens = session.tokens_for_file(uri);
            match debug.show_collected_tokens_as_warnings {
                Warnings::Default => diagnostics,
                // If collected_tokens_as_warnings is Parsed or Typed,
                // take over the normal error and warning display behavior
                // and instead show the either the parsed or typed tokens as warnings.
                // This is useful for debugging the lsp parser.
                Warnings::Parsed => debug::generate_warnings_for_parsed_tokens(tokens),
                Warnings::Typed => debug::generate_warnings_for_typed_tokens(tokens),
            }
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
        tracing::info!("Initializing the Sway Language Server");

        if let Some(initialization_options) = &params.initialization_options {
            let mut config = self.config.write();
            *config = serde_json::from_value(initialization_options.clone())
                .ok()
                .unwrap_or_default();
        }

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
                if let Some(src) = session.update_text_document(&uri, params.content_changes) {
                    if let Ok(mut file) = File::create(uri.path()) {
                        let _ = writeln!(&mut file, "{}", src);
                    }
                }
                self.parse_project(uri, params.text_document.uri, session.clone())
                    .await;
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
                Ok(capabilities::hover::hover_data(session, uri, position))
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
pub struct RunnableParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
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

    pub async fn runnables(
        &self,
        params: RunnableParams,
    ) -> jsonrpc::Result<Option<Vec<(Range, String)>>> {
        match self.get_uri_and_session(&params.text_document.uri) {
            Ok((_, session)) => {
                let ranges = session
                    .runnables
                    .get(&capabilities::runnable::RunnableType::MainFn)
                    .map(|item| {
                        let runnable = item.value();
                        vec![(runnable.range, format!("{}", runnable.tree_type))]
                    });

                Ok(ranges)
            }
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        }
    }

    /// This method is triggered by a command palette request in VScode
    /// The 2 commands are: "show parsed ast" or "show typed ast"
    ///
    /// If either command is executed, the client requests this method
    /// by calling the "sway/show_ast".
    ///
    /// The function expects the URI of the current open file where the
    /// request was made, and if the "parsed" or "typed" ast was requested.
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
                            let _ = writeln!(&mut file, "{}", ast_string);
                            if let Ok(uri) = Url::from_file_path(path) {
                                // Return the tmp file path where the AST has been written to.
                                return Some(TextDocumentIdentifier::new(uri));
                            }
                        }
                        None
                    };

                {
                    let program = session.compiled_program.read();
                    match params.ast_kind.as_str() {
                        "parsed" => {
                            match program.parsed {
                                Some(ref parsed_program) => {
                                    // Initialize the string with the AST from the root
                                    let mut formatted_ast: String =
                                        format!("{:#?}", parsed_program.root.tree.root_nodes);

                                    for (ident, submodule) in &parsed_program.root.submodules {
                                        // if the current path matches the path of a submodule
                                        // overwrite the root AST with the submodule AST
                                        if ident.span().path().map(|a| a.deref()) == path.as_ref() {
                                            formatted_ast =
                                                format!("{:#?}", submodule.module.tree.root_nodes);
                                        }
                                    }

                                    let tmp_ast_path = Path::new("/tmp/parsed_ast.rs");
                                    Ok(write_ast_to_file(tmp_ast_path, &formatted_ast))
                                }
                                _ => Ok(None),
                            }
                        }
                        "typed" => {
                            match program.typed {
                                Some(ref typed_program) => {
                                    // Initialize the string with the AST from the root
                                    let mut formatted_ast: String =
                                        format!("{:#?}", typed_program.root.all_nodes);

                                    for (ident, submodule) in &typed_program.root.submodules {
                                        // if the current path matches the path of a submodule
                                        // overwrite the root AST with the submodule AST
                                        if ident.span().path().map(|a| a.deref()) == path.as_ref() {
                                            formatted_ast =
                                                format!("{:#?}", submodule.module.all_nodes);
                                        }
                                    }

                                    let tmp_ast_path = Path::new("/tmp/typed_ast.rs");
                                    Ok(write_ast_to_file(tmp_ast_path, &formatted_ast))
                                }
                                _ => Ok(None),
                            }
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
    use crate::test_utils::{doc_comments_dir, e2e_test_dir};
    use serde_json::json;
    use serial_test::serial;
    use std::{borrow::Cow, fs, io::Read, path::PathBuf};
    use tower::{Service, ServiceExt};
    use tower_lsp::{
        jsonrpc::{self, Id, Request, Response},
        ExitedError, LspService,
    };

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
        let ok = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_eq!(response, Ok(Some(ok)));
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
        let ok = Response::from_ok(1.into(), json!(null));
        assert_eq!(response, Ok(Some(ok)));
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

    async fn show_ast_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "astKind": "typed",
        });
        let show_ast = build_request_with_id("sway/show_ast", params, 1);
        let response = call_request(service, show_ast.clone()).await;
        let ok = Response::from_ok(1.into(), json!({"uri": "file:///tmp/typed_ast.rs"}));
        assert_eq!(response, Ok(Some(ok)));
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

    async fn go_to_definition_request(
        service: &mut LspService<Backend>,
        uri: &Url,
        token_req_line: i32,
        token_def_line: i32,
        id: i64,
    ) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": token_req_line,
                "character": 24,
            }
        });
        let definition = build_request_with_id("textDocument/definition", params, id);
        let response = call_request(service, definition.clone()).await;
        let ok = Response::from_ok(
            id.into(),
            json!({
                "range": {
                    "end": {
                        "character": 11,
                        "line": token_def_line,
                    },
                    "start": {
                        "character": 7,
                        "line": token_def_line,
                    }
                },
                "uri": uri,
            }),
        );
        assert_eq!(response, Ok(Some(ok)));
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
        let ok = Response::from_ok(
            1.into(),
            json!({
                "contents": {
                    "kind": "markdown",
                    "value": " Struct holding:\n\n 1. A `value` of type `NumberOrString`\n 2. An `address` of type `u64`"
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
        assert_eq!(response, Ok(Some(ok)));
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
                "line": 44,
                "character": 27
            }
        });
        let highlight = build_request_with_id("textDocument/documentHighlight", params, 1);
        let response = call_request(service, highlight.clone()).await;
        let ok = Response::from_ok(
            1.into(),
            json!([{
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
            }]),
        );
        assert_eq!(response, Ok(Some(ok)));
        highlight
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

    #[tokio::test]
    #[serial]
    async fn initialize() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn initialized() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(Backend::new);
        let initialize = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let response = call_request(&mut service, initialize).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    #[serial]
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
    #[serial]
    async fn refuses_requests_after_shutdown() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    #[serial]
    async fn did_open() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn did_close() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        did_close_notification(&mut service).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn did_change() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let _ = did_change_request(&mut service, &uri).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn lsp_syncs_with_workspace_edits() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let _ = go_to_definition_request(&mut service, &uri, 44, 19, 1).await;
        let _ = did_change_request(&mut service, &uri).await;
        let _ = go_to_definition_request(&mut service, &uri, 45, 20, 2).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn show_ast() {
        let (mut service, _) = LspService::build(Backend::new)
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();

        let uri = init_and_open(&mut service, e2e_test_dir()).await;
        let _ = show_ast_request(&mut service, &uri).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn go_to_definition() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir()).await;
        let _ = go_to_definition_request(&mut service, &uri, 44, 19, 1).await;
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
        ($test:ident, $capability:expr) => {
            #[tokio::test]
            #[serial]
            async fn $test() {
                test_lsp_capability!(doc_comments_dir(), $capability);
            }
        };
    }

    lsp_capability_test!(semantic_tokens, semantic_tokens_request);
    lsp_capability_test!(document_symbol, document_symbol_request);
    lsp_capability_test!(format, format_request);
    lsp_capability_test!(hover, hover_request);
    lsp_capability_test!(highlight, highlight_request);
}
