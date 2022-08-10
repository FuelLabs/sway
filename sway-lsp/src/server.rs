use crate::capabilities;
use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
    token::TokenMap,
};
use crate::utils::debug::{self, DebugFlags};
use forc_util::find_manifest_dir;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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

    async fn log_error_message(&self, message: &str) {
        self.client.log_message(MessageType::ERROR, message).await;
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
}

fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        semantic_tokens_provider: capabilities::semantic_tokens::semantic_tokens(),
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
    async fn publish_diagnostics(
        &self,
        uri: &Url,
        diagnostics: Vec<Diagnostic>,
        token_map: &TokenMap,
    ) {
        match &self.config.collected_tokens_as_warnings {
            Some(s) => {
                // If collected_tokens_as_warnings is Some, take over the normal error and warning display behavior
                // and instead show the either the parsed or typed tokens as warnings.
                // This is useful for debugging the lsp parser.
                let diagnostics = match s.as_str() {
                    "parsed" => Some(debug::generate_warnings_for_parsed_tokens(token_map)),
                    "typed" => Some(debug::generate_warnings_for_typed_tokens(token_map)),
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
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        if let Some(options) = params.initialization_options {
            self.session.update_config(options);
        }

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

        match self.session.parse_project(&uri) {
            Ok(diagnostics) => {
                let tokens = self.session.tokens_for_file(&uri);
                self.publish_diagnostics(&uri, diagnostics, &tokens).await
            }
            Err(_) => self.log_error_message("Unable to Parse Project!").await,
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.session
            .update_text_document(&uri, params.content_changes);
        match self.session.parse_project(&uri) {
            Ok(diagnostics) => {
                let tokens = self.session.tokens_for_file(&uri);
                self.publish_diagnostics(&uri, diagnostics, &tokens).await
            }
            Err(_) => self.log_error_message("Unable to Parse Project!").await,
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        match self.session.parse_project(&uri) {
            Ok(diagnostics) => {
                let tokens = self.session.tokens_for_file(&uri);
                self.publish_diagnostics(&uri, diagnostics, &tokens).await
            }
            Err(_) => self.log_error_message("Unable to Parse Project!").await,
        }
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
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::{env, fs, io::Read, path::PathBuf};
    use tower::{Service, ServiceExt};

    use super::*;
    use futures::stream::StreamExt;
    use tower_lsp::jsonrpc::{self, Request, Response};
    use tower_lsp::LspService;

    fn _e2e_test_dir() -> PathBuf {
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/src/e2e_vm_tests/test_programs/should_pass/language")
            .join("enum_if_let")
    }

    fn sway_example_dir() -> PathBuf {
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/signatures")
    }

    fn load_sway_example() -> (Url, String) {
        let manifest_dir = sway_example_dir();
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

    async fn did_close_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("textDocument/didClose").finish();
        let response = service.ready().await.unwrap().call(exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    fn config() -> DebugFlags {
        Default::default()
    }

    #[tokio::test]
    async fn initialize() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;
    }

    #[tokio::test]
    async fn initialized() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;
    }

    #[tokio::test]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let initialize = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        // send "initialize" request (again); should error
        let response = service.ready().await.unwrap().call(initialize).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn shutdown() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        // send "shutdown" request
        let shutdown = shutdown_request(&mut service).await;

        // send "shutdown" request (again); should error
        let response = service.ready().await.unwrap().call(shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));

        // send "exit" request
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn refuses_requests_after_shutdown() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "shutdown" request
        let shutdown = shutdown_request(&mut service).await;

        let response = service.ready().await.unwrap().call(shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn did_open() {
        let (mut service, mut messages) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
        messages.next().await.unwrap();

        let (uri, sway_program) = load_sway_example();

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(&mut service, &uri, &sway_program).await;

        // ignore the "textDocument/publishDiagnostics" notification
        messages.next().await.unwrap();

        // send "shutdown" request
        let _ = shutdown_request(&mut service).await;

        // send "exit" request
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn did_close() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        let (uri, sway_program) = load_sway_example();

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(&mut service, &uri, &sway_program).await;

        // send "textDocument/didClose" notification for `uri`
        did_close_notification(&mut service).await;

        // send "shutdown" request
        let _ = shutdown_request(&mut service).await;

        // send "exit" request
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn did_change() {
        let (mut service, _) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        let uri = Url::parse("inmemory:///test").unwrap();
        let text = r#"script;

        fn main() {
        
        }
        "#;

        // This just an example of the changes made
        // In reality, the only text that needs to be sent to the language server
        // is "let x = 0.0;"
        let _new_text = r#"script;

        fn main() {
            let x = 0.0;
        }
        "#;

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(&mut service, &uri, text).await;

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
        let did_change = Request::build("textDocument/didChange")
            .params(params)
            .finish();
        let response = service.ready().await.unwrap().call(did_change).await;
        assert_eq!(response, Ok(None));

        // send "shutdown" request
        let _ = shutdown_request(&mut service).await;

        // send "exit" request
        exit_notification(&mut service).await;
    }
}
