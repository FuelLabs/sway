use crate::capabilities;
use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
};
use crate::utils::debug::{self, DebugFlags};
use forc_util::find_manifest_dir;
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

    fn parse_and_store_sway_files(&self) -> Result<(), DocumentError> {
        let curr_dir = std::env::current_dir().unwrap();

        if let Some(path) = find_manifest_dir(&curr_dir) {
            let files = get_sway_files(path);

            for file_path in files {
                if let Some(path) = file_path.to_str() {
                    // store the document
                    let text_document = TextDocument::build_from_path(path)?;
                    self.session.store_document(text_document)?;
                    // parse the document for tokens
                    let _ = self.session.parse_document(path);
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
        definition_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: capabilities::semantic_tokens::get_semantic_tokens(),
        document_symbol_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: None,
            ..Default::default()
        }),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![],
            ..Default::default()
        }),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    }
}

impl Backend {
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        // If parsed_tokens_as_warnings is true, take over the normal error and warning display behavior
        // and instead show the parsed tokens as warnings.
        // This is useful for debugging the lsp parser.
        if self.config.parsed_tokens_as_warnings {
            if let Some(document) = self.session.documents.get(uri.path()) {
                let diagnostics = debug::generate_warnings_for_parsed_tokens(document.get_tokens());
                self.client
                    .publish_diagnostics(uri, diagnostics, None)
                    .await;
            }
        } else if !diagnostics.is_empty() {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
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
        let _ = self.parse_and_store_sway_files();

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
        let diagnostics = capabilities::text_sync::handle_open_file(self.session.clone(), &params);
        self.publish_diagnostics(uri, diagnostics).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let diagnostics = capabilities::text_sync::handle_change_file(self.session.clone(), params);
        self.publish_diagnostics(uri, diagnostics).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let diagnostics = capabilities::text_sync::handle_save_file(self.session.clone(), &params);
        self.publish_diagnostics(uri, diagnostics).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let events = params.changes;
        capabilities::file_sync::handle_watched_files(self.session.clone(), events);
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        Ok(capabilities::hover::get_hover_data(
            self.session.clone(),
            params,
        ))
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        // TODO
        // here we would also need to provide a list of builtin methods not just the ones from the document
        Ok(capabilities::completion::get_completion(
            self.session.clone(),
            params,
        ))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        Ok(capabilities::document_symbol::document_symbol(
            self.session.clone(),
            params.text_document.uri,
        ))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> jsonrpc::Result<Option<SemanticTokensResult>> {
        Ok(capabilities::semantic_tokens::get_semantic_tokens_full(
            self.session.clone(),
            params,
        ))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        Ok(capabilities::highlight::get_highlights(
            self.session.clone(),
            params,
        ))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        Ok(capabilities::go_to::go_to_definition(
            self.session.clone(),
            params,
        ))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        Ok(capabilities::formatting::format_document(
            self.session.clone(),
            params,
        ))
    }

    async fn rename(&self, params: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        Ok(capabilities::rename::rename(self.session.clone(), params))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> jsonrpc::Result<Option<PrepareRenameResponse>> {
        Ok(capabilities::rename::prepare_rename(
            self.session.clone(),
            params,
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::{env, fs::File, io::Write};
    use tower::{Service, ServiceExt};

    use super::*;
    use futures::stream::StreamExt;
    use tower_lsp::jsonrpc::{self, Request, Response};
    use tower_lsp::LspService;

    // Simple sway script used for testing LSP capabilites
    const SWAY_PROGRAM: &str = r#"script;

use std::*;

/// A simple Particle struct
struct Particle {
    position: [u64; 3],
    velocity: [u64; 3],
    acceleration: [u64; 3],
    mass: u64,
}

impl Particle {
    /// Creates a new Particle with the given position, velocity, acceleration, and mass
    fn new(position: [u64; 3], velocity: [u64; 3], acceleration: [u64; 3], mass: u64) -> Particle {
        Particle {
            position: position,
            velocity: velocity,
            acceleration: acceleration,
            mass: mass,
        }
    }
}

fn main() {
    let position = [0, 0, 0];
    let velocity = [0, 1, 0];
    let acceleration = [1, 1, 0];
    let mass = 10;
    let p = ~Particle::new(position, velocity, acceleration, mass);
}
"#;

    fn load_test_sway_file(sway_file: &str) -> Url {
        let file_name = "tmp_sway_test_file.sw";
        let dir = env::temp_dir().join(file_name);
        let mut file = File::create(&dir).unwrap();
        file.write_all(sway_file.as_bytes()).unwrap();
        Url::from_file_path(dir.as_os_str().to_str().unwrap()).unwrap()
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

        let uri = load_test_sway_file(SWAY_PROGRAM);

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(&mut service, &uri, SWAY_PROGRAM).await;

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

        let uri = load_test_sway_file(SWAY_PROGRAM);

        // send "textDocument/didOpen" notification for `uri`
        did_open_notification(&mut service, &uri, SWAY_PROGRAM).await;

        // send "textDocument/didClose" notification for `uri`
        did_close_notification(&mut service).await;

        // send "shutdown" request
        let _ = shutdown_request(&mut service).await;

        // send "exit" request
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn did_change() {
        let (mut service, mut messages) = LspService::new(|client| Backend::new(client, config()));

        // send "initialize" request
        let _ = initialize_request(&mut service).await;

        // send "initialized" notification
        initialized_notification(&mut service).await;

        // ignore the "window/logMessage" notification: "Initializing the Sway Language Server"
        messages.next().await.unwrap();

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
