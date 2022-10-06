use crate::capabilities;
use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
};
use crate::utils::debug::{self, DebugFlags};
use forc_util::find_manifest_dir;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, ops::Deref, path::Path, sync::Arc};
use sway_types::Spanned;
use sway_utils::helpers::get_sway_files;
use tower_lsp::lsp_types::*;
use tower_lsp::{jsonrpc, Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    session: Arc<Session>,
    config: DebugFlags,
}

impl Backend {
    pub fn new(client: Client, config: DebugFlags) -> Self {
        let session = Arc::new(Session::new());
        Backend {
            client,
            session,
            config,
        }
    }

    async fn log_info_message(&self, message: &str) {
        self.client.log_message(MessageType::INFO, message).await;
    }

    async fn parse_and_store_sway_files(&self) -> Result<(), DocumentError> {
        let curr_dir = std::env::current_dir().unwrap();

        if let Some(path) = find_manifest_dir(&curr_dir) {
            let files = get_sway_files(path);
            for file_path in files {
                if let Some(path) = file_path.to_str() {
                    // store the document
                    let text_document = TextDocument::build_from_path(path)?;
                    self.session.store_document(text_document)?;
                }
            }
        }

        Ok(())
    }

    async fn parse_project(&self, uri: &Url) {
        let diagnostics = match self.session.parse_project(uri) {
            Ok(diagnostics) => diagnostics,
            Err(err) => {
                if let DocumentError::FailedToParse(diagnostics) = err {
                    diagnostics
                } else {
                    vec![]
                }
            }
        };
        self.publish_diagnostics(uri, diagnostics).await;
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
        ..ServerCapabilities::default()
    }
}

impl Backend {
    async fn publish_diagnostics(&self, uri: &Url, diagnostics: Vec<Diagnostic>) {
        match &self.config.collected_tokens_as_warnings {
            Some(s) => {
                let token_map = self.session.tokens_for_file(uri);

                // If collected_tokens_as_warnings is Some, take over the normal error and warning display behavior
                // and instead show the either the parsed or typed tokens as warnings.
                // This is useful for debugging the lsp parser.
                let diagnostics = match s.as_str() {
                    "parsed" => Some(debug::generate_warnings_for_parsed_tokens(&token_map)),
                    "typed" => Some(debug::generate_warnings_for_typed_tokens(&token_map)),
                    _ => None,
                };
                if let Some(diagnostics) = diagnostics {
                    self.client
                        .publish_diagnostics(uri.clone(), diagnostics, None)
                        .await;
                }
            }
            None => {
                // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
                // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, None)
                    .await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "Initializing the Sway Language Server")
            .await;

        // iterate over the project dir, parse all sway files
        let _ = self.parse_and_store_sway_files().await;

        Ok(InitializeResult {
            server_info: None,
            capabilities: capabilities(),
        })
    }

    // LSP-Server Lifecycle
    async fn initialized(&self, _: InitializedParams) {
        self.log_info_message("Sway Language Server Initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.log_info_message("Shutting Down the Sway Language Server")
            .await;
        Ok(())
    }

    // Document Handlers
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.session.handle_open_file(&uri);
        self.parse_project(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.session
            .update_text_document(&uri, params.content_changes);
        self.parse_project(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.parse_project(&uri).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for event in params.changes {
            if event.typ == FileChangeType::DELETED {
                let _ = self.session.remove_document(&event.uri);
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        Ok(capabilities::hover::hover_data(&self.session, params))
    }

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        // TODO
        // here we would also need to provide a list of builtin methods not just the ones from the document
        Ok(self
            .session
            .completion_items()
            .map(CompletionResponse::Array))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        Ok(self
            .session
            .symbol_information(&params.text_document.uri)
            .map(DocumentSymbolResponse::Flat))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> jsonrpc::Result<Option<SemanticTokensResult>> {
        let url = params.text_document.uri;
        Ok(capabilities::semantic_tokens::semantic_tokens_full(
            &self.session,
            &url,
        ))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        Ok(capabilities::highlight::get_highlights(
            &self.session,
            params,
        ))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        Ok(self.session.token_definition_response(params))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        Ok(self.session.format_text(&params.text_document.uri))
    }

    async fn rename(&self, params: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        Ok(capabilities::rename::rename(&self.session, params))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> jsonrpc::Result<Option<PrepareRenameResponse>> {
        Ok(capabilities::rename::prepare_rename(&self.session, params))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnableParams {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
}

// Custom LSP-Server Methods
impl Backend {
    pub async fn runnables(
        &self,
        _params: RunnableParams,
    ) -> jsonrpc::Result<Option<Vec<(Range, String)>>> {
        let ranges = self
            .session
            .runnables
            .get(&capabilities::runnable::RunnableType::MainFn)
            .map(|item| {
                let runnable = item.value();
                vec![(runnable.range, format!("{}", runnable.tree_type))]
            });

        Ok(ranges)
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

        match self.session.compiled_program.read() {
            std::sync::LockResult::Ok(program) => {
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
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::{env, fs, io::Read, path::PathBuf};
    use tower::{Service, ServiceExt};

    use super::*;
    use tower_lsp::{
        jsonrpc::{self, Request, Response},
        {ClientSocket, LspService},
    };

    #[allow(dead_code)]
    fn e2e_test_dir() -> PathBuf {
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/src/e2e_vm_tests/test_programs/should_pass/language")
            .join("struct_field_access")
    }

    #[allow(dead_code)]
    fn sway_example_dir() -> PathBuf {
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/storage_variables")
    }

    #[allow(dead_code)]
    fn doc_comments() -> PathBuf {
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/src/e2e_vm_tests/test_programs/should_pass/language")
            .join("doc_comments")
    }

    fn load_sway_example(manifest_dir: PathBuf) -> (Url, String) {
        let src_path = manifest_dir.join("src/main.sw");
        let mut file = fs::File::open(&src_path).unwrap();
        let mut sway_program = String::new();
        file.read_to_string(&mut sway_program).unwrap();

        let uri = Url::from_file_path(src_path).unwrap();

        (uri, sway_program)
    }

    async fn initialize_request(service: &mut LspService<Backend>) -> Request {
        let initialize = Request::build("initialize")
            .params(json!({ "capabilities": capabilities() }))
            .id(1)
            .finish();
        let response = service
            .ready()
            .await
            .unwrap()
            .call(initialize.clone())
            .await;
        let ok = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_eq!(response, Ok(Some(ok)));
        initialize
    }

    async fn initialized_notification(service: &mut LspService<Backend>) {
        let initialized = Request::build("initialized").finish();
        let response = service.ready().await.unwrap().call(initialized).await;
        assert_eq!(response, Ok(None));
    }

    async fn shutdown_request(service: &mut LspService<Backend>) -> Request {
        let shutdown = Request::build("shutdown").id(1).finish();
        let response = service.ready().await.unwrap().call(shutdown.clone()).await;
        let ok = Response::from_ok(1.into(), json!(null));
        assert_eq!(response, Ok(Some(ok)));
        shutdown
    }

    async fn exit_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("exit").finish();
        let response = service.ready().await.unwrap().call(exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn did_open_notification(service: &mut LspService<Backend>, uri: &Url, text: &str) {
        let language_id = "sway";
        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": 1,
                "text": text,
            },
        });
        let did_open = Request::build("textDocument/didOpen")
            .params(params)
            .finish();
        let response = service.ready().await.unwrap().call(did_open).await;
        assert_eq!(response, Ok(None));
    }

    async fn did_change_request(
        service: &mut LspService<Backend>,
        params: serde_json::value::Value,
    ) -> Request {
        let did_change = Request::build("textDocument/didChange")
            .params(params)
            .finish();
        let response = service
            .ready()
            .await
            .unwrap()
            .call(did_change.clone())
            .await;
        assert_eq!(response, Ok(None));
        did_change
    }

    async fn did_close_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("textDocument/didClose").finish();
        let response = service.ready().await.unwrap().call(exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn show_ast_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "astKind": "typed",
        });
        let show_ast = Request::build("sway/show_ast")
            .params(params)
            .id(1)
            .finish();
        let response = service.ready().await.unwrap().call(show_ast.clone()).await;
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
        let semantic_tokens = Request::build("textDocument/semanticTokens/full")
            .params(params)
            .id(1)
            .finish();
        let _response = service
            .ready()
            .await
            .unwrap()
            .call(semantic_tokens.clone())
            .await;
        semantic_tokens
    }

    async fn document_symbol_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let document_symbol = Request::build("textDocument/documentSymbol")
            .params(params)
            .id(1)
            .finish();
        let _response = service
            .ready()
            .await
            .unwrap()
            .call(document_symbol.clone())
            .await;
        document_symbol
    }

    async fn go_to_definition_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": 44,
                "character": 24
            }
        });
        let definition = Request::build("textDocument/definition")
            .params(params)
            .id(1)
            .finish();
        let response = service
            .ready()
            .await
            .unwrap()
            .call(definition.clone())
            .await;
        let ok = Response::from_ok(
            1.into(),
            json!({
                "range": {
                    "end": {
                        "character": 11,
                        "line": 19
                    },
                    "start": {
                        "character": 7,
                        "line": 19
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
        let hover = Request::build("textDocument/hover")
            .params(params)
            .id(1)
            .finish();
        let response = service.ready().await.unwrap().call(hover.clone()).await;
        let ok = Response::from_ok(
            1.into(),
            json!({
                "contents": {
                    "kind": "markdown",
                    "value": "```sway\nstruct Data\n```"
                },
                "range": {
                    "end": {
                        "character": 11,
                        "line": 19
                    },
                    "start": {
                        "character": 7,
                        "line": 19
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
        let formatting = Request::build("textDocument/formatting")
            .params(params)
            .id(1)
            .finish();
        let _response = service
            .ready()
            .await
            .unwrap()
            .call(formatting.clone())
            .await;
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
        let highlight = Request::build("textDocument/documentHighlight")
            .params(params)
            .id(1)
            .finish();
        let response = service.ready().await.unwrap().call(highlight.clone()).await;
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

    fn config() -> DebugFlags {
        Default::default()
    }

    async fn init_and_open(
        service: &mut LspService<Backend>,
        messages: &mut ClientSocket,
        manifest_dir: PathBuf,
    ) -> Url {
        // send "initialize" request
        let _ = initialize_request(service).await;

        // send "initialized" notification
        initialized_notification(service).await;

        // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
        //let _ = messages.next().await;

        let (uri, sway_program) = load_sway_example(manifest_dir);

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(service, &uri, &sway_program).await;

        // ignore the "textDocument/publishDiagnostics" notification
        //let _ = messages.next().await;

        uri
    }

    async fn shutdown_and_exit(service: &mut LspService<Backend>) {
        // send "shutdown" request
        let _ = shutdown_request(service).await;

        // send "exit" request
        exit_notification(service).await;
    }

    #[tokio::test]
    async fn initialize() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

                // send "initialize" request
                let _ = initialize_request(&mut service).await;
            })
        });
    }

    #[tokio::test]
    async fn initialized() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));

                // send "initialize" request
                let _ = initialize_request(&mut service).await;

                // send "initialized" notification
                initialized_notification(&mut service).await;

                // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
                //let _ = messages.next().await;
            })
        });
    }

    #[tokio::test]
    async fn initializes_only_once() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));

                // send "initialize" request
                let initialize = initialize_request(&mut service).await;

                // send "initialized" notification
                initialized_notification(&mut service).await;

                // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
                //let _ = messages.next().await;

                // send "initialize" request (again); should error
                let response = service.ready().await.unwrap().call(initialize).await;
                let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
                assert_eq!(response, Ok(Some(err)));
            })
        });
    }

    #[tokio::test]
    async fn shutdown() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));

                // send "initialize" request
                let _ = initialize_request(&mut service).await;

                // send "initialized" notification
                initialized_notification(&mut service).await;

                // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
                //let _ = messages.next().await;

                // send "shutdown" request
                let shutdown = shutdown_request(&mut service).await;

                // send "shutdown" request (again); should error
                let response = service.ready().await.unwrap().call(shutdown).await;
                let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
                assert_eq!(response, Ok(Some(err)));

                // send "exit" request
                exit_notification(&mut service).await;
            })
        });
    }

    #[tokio::test]
    async fn refuses_requests_after_shutdown() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));

                // send "initialize" request
                let _ = initialize_request(&mut service).await;

                // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
                //let _ = messages.next().await;

                // send "shutdown" request
                let shutdown = shutdown_request(&mut service).await;

                let response = service.ready().await.unwrap().call(shutdown).await;
                let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
                assert_eq!(response, Ok(Some(err)));
            })
        });
    }

    #[tokio::test]
    async fn did_open() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));
                let _ = init_and_open(&mut service, &mut messages, e2e_test_dir()).await;
                shutdown_and_exit(&mut service).await;
            })
        });
    }

    #[tokio::test]
    async fn did_close() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));
                let _ = init_and_open(&mut service, &mut messages, e2e_test_dir()).await;

                // send "textDocument/didClose" notification for `uri`
                did_close_notification(&mut service).await;

                shutdown_and_exit(&mut service).await;
            })
        });
    }

    #[tokio::test]
    async fn did_change() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::new(|client| Backend::new(client, config()));
                let uri = init_and_open(&mut service, &mut messages, e2e_test_dir()).await;

                // send "textDocument/didChange" notification for `uri`
                let params = json!({
                    "textDocument": {
                        "uri": uri,
                        "version": 1
                    },
                    "contentChanges": [
                        {
                            "range": {
                                "start": {
                                    "line": 3,
                                    "character": 4
                                },
                                "end": {
                                    "line": 3,
                                    "character": 4
                                }
                            },
                            "rangeLength": 0,
                            "text": "let x = 0.0;",
                        }
                    ]
                });

                let _ = did_change_request(&mut service, params).await;

                // ignore the "textDocument/publishDiagnostics" notification
                //let _ = messages.next().await;

                shutdown_and_exit(&mut service).await;
            })
        });
    }

    #[tokio::test]
    async fn show_ast() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (mut service, mut messages) =
                    LspService::build(|client| Backend::new(client, config()))
                        .custom_method("sway/show_ast", Backend::show_ast)
                        .finish();

                let uri = init_and_open(&mut service, &mut messages, e2e_test_dir()).await;

                // send "sway/show_typed_ast" request
                let _ = show_ast_request(&mut service, &uri).await;

                shutdown_and_exit(&mut service).await;
            })
        });
    }

    // This macro allows us to spin up a server / client for testing
    // It initializes and performs the necessary handshake and then loads
    // the sway example that was passed into `example_dir`.
    // It then runs the specific capability to test before gracefully shutting down.
    // The capability argument is an async function.
    macro_rules! test_lsp_capability {
        ($example_dir:expr, $capability:expr) => {{
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let (mut service, mut messages) =
                        LspService::new(|client| Backend::new(client, config()));
                    let uri = init_and_open(&mut service, &mut messages, $example_dir).await;
                    // Call the specific LSP capability function that was passed in.
                    let _ = $capability(&mut service, &uri).await;

                    shutdown_and_exit(&mut service).await;
                })
            });
        }};
    }

    #[tokio::test]
    async fn semantic_tokens() {
        // send "textDocument/semanticTokens/full" request
        test_lsp_capability!(doc_comments(), semantic_tokens_request);
    }

    #[tokio::test]
    async fn document_symbol() {
        // send "textDocument/documentSymbol" request
        test_lsp_capability!(doc_comments(), document_symbol_request);
    }

    #[tokio::test]
    async fn go_to_definition() {
        // send "textDocument/definition" request
        test_lsp_capability!(doc_comments(), go_to_definition_request);
    }

    #[tokio::test]
    async fn format() {
        // send "textDocument/formatting" request
        test_lsp_capability!(doc_comments(), format_request);
    }

    #[tokio::test]
    async fn hover() {
        // send "textDocument/hover" request
        test_lsp_capability!(doc_comments(), hover_request);
    }

    #[tokio::test]
    async fn highlight() {
        // send "textDocument/documentHighlight" request
        test_lsp_capability!(doc_comments(), highlight_request);
    }
}
