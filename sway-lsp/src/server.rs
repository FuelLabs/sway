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
        if let Some(path) = find_manifest_dir(&std::env::current_dir().unwrap()) {
            // Store the documents.
            for path in get_sway_files(path).iter().filter_map(|fp| fp.to_str()) {
                self.session
                    .store_document(TextDocument::build_from_path(path)?)?;
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
        let uri = &params.text_document.uri;
        self.session.handle_open_file(uri);
        self.parse_project(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        self.session
            .update_text_document(uri, params.content_changes);
        self.parse_project(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.parse_project(&params.text_document.uri).await;
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
    use std::{borrow::Cow, env, fs, io::Read, path::PathBuf};
    use tower::{Service, ServiceExt};

    use super::*;
    use serial_test::serial;
    use tower_lsp::{
        jsonrpc::{self, Id, Request, Response},
        ExitedError, LspService,
    };

    fn sway_workspace_dir() -> PathBuf {
        env::current_dir().unwrap().parent().unwrap().to_path_buf()
    }

    fn e2e_language_dir() -> PathBuf {
        PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/language")
    }

    #[allow(dead_code)]
    fn e2e_test_dir() -> PathBuf {
        sway_workspace_dir()
            .join(e2e_language_dir())
            .join("struct_field_access")
    }

    #[allow(dead_code)]
    fn sway_example_dir() -> PathBuf {
        sway_workspace_dir().join("examples/storage_variables")
    }

    fn doc_comments_dir() -> PathBuf {
        sway_workspace_dir()
            .join(e2e_language_dir())
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

    async fn did_change_request(
        service: &mut LspService<Backend>,
        params: serde_json::value::Value,
    ) -> Request {
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
        let definition = build_request_with_id("textDocument/definition", params, 1);
        let response = call_request(service, definition.clone()).await;
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
        let hover = build_request_with_id("textDocument/hover", params, 1);
        let response = call_request(service, hover.clone()).await;
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

    fn config() -> DebugFlags {
        Default::default()
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
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let _ = initialize_request(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn initialized() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let initialize = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let response = call_request(&mut service, initialize).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    #[serial]
    async fn shutdown() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
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
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let _ = initialize_request(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    #[serial]
    async fn did_open() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn did_close() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let _ = init_and_open(&mut service, e2e_test_dir()).await;
        did_close_notification(&mut service).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn did_change() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
        let uri = init_and_open(&mut service, e2e_test_dir()).await;

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
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    #[serial]
    async fn show_ast() {
        let (mut service, _) = LspService::build(|client| Backend::new(client, config()))
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();

        let uri = init_and_open(&mut service, e2e_test_dir()).await;
        let _ = show_ast_request(&mut service, &uri).await;
        shutdown_and_exit(&mut service).await;
    }

    // This macro allows us to spin up a server / client for testing
    // It initializes and performs the necessary handshake and then loads
    // the sway example that was passed into `example_dir`.
    // It then runs the specific capability to test before gracefully shutting down.
    // The capability argument is an async function.
    macro_rules! test_lsp_capability {
        ($example_dir:expr, $capability:expr) => {{
            let (mut service, _) = LspService::new(|client| Backend::new(client, config()));
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
    lsp_capability_test!(go_to_definition, go_to_definition_request);
    lsp_capability_test!(format, format_request);
    lsp_capability_test!(hover, hover_request);
    lsp_capability_test!(highlight, highlight_request);
}
