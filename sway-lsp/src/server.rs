use crate::capabilities;
use crate::core::{
    document::{DocumentError, TextDocument},
    session::Session,
};
use std::sync::Arc;
use sway_utils::helpers::{find_manifest_dir, get_sway_files};
use tower_lsp::lsp_types::*;
use tower_lsp::{jsonrpc, Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    session: Arc<Session>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let session = Arc::new(Session::new());
        Backend { client, session }
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
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![],
            ..Default::default()
        }),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        if let Some(options) = params.initialization_options {
            self.session.update_config(options);
        }

        self.client
            .log_message(MessageType::INFO, "Initializing the Server")
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
        self.log_info_message("Server initialized").await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.log_info_message("Shutting the server").await;
        Ok(())
    }

    // Document Handlers
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let diagnostics = capabilities::text_sync::handle_open_file(self.session.clone(), &params);

        if !diagnostics.is_empty() {
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let _ = capabilities::text_sync::handle_change_file(self.session.clone(), params);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let url = params.text_document.uri.clone();
        self.client.publish_diagnostics(url, vec![], None).await;

        if let Some(diagnostics) =
            capabilities::text_sync::handle_save_file(self.session.clone(), &params)
        {
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
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
    use std::ops::Not;

    use serde_json::json;
    use tower::{Service, ServiceExt};

    use super::*;
    use futures::stream::StreamExt;
    use tower_lsp::jsonrpc::{self, Request, Response};
    use tower_lsp::LspService;

    fn initialize_request(id: i64) -> Request {
        Request::build("initialize")
            .params(json!({ "capabilities": capabilities() }))
            .id(id)
            .finish()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(Backend::new);

        let request = initialize_request(1);

        let response = service.ready().await.unwrap().call(request.clone()).await;
        let ok = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_eq!(response, Ok(Some(ok)));

        let response = service.ready().await.unwrap().call(request).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn refuses_requests_after_shutdown() {
        let (mut service, _) = LspService::new(Backend::new);

        let initialize = initialize_request(1);
        let response = service.ready().await.unwrap().call(initialize).await;
        let ok = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_eq!(response, Ok(Some(ok)));

        let shutdown = Request::build("shutdown").id(1).finish();
        let response = service.ready().await.unwrap().call(shutdown.clone()).await;
        let ok = Response::from_ok(1.into(), json!(null));
        assert_eq!(response, Ok(Some(ok)));

        let response = service.ready().await.unwrap().call(shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn did_open() {
        let (mut service, mut messages) = LspService::new(Backend::new);

        // send "initialize" request
        let initialize = initialize_request(1);
        let response = service.ready().await.unwrap().call(initialize).await;
        let ok = Response::from_ok(1.into(), json!({ "capabilities": capabilities() }));
        assert_eq!(response, Ok(Some(ok)));

        // send "initialized" notification
        let initialized = Request::build("initialized").finish();
        let response = service.ready().await.unwrap().call(initialized).await;
        assert_eq!(response, Ok(None));

        // ignore the "window/logMessage" notification: "Initializing the Server"
        println!("message = {:?}", messages.next().await.unwrap());

        // send "textDocument/didOpen" notification for `uri`
        let uri = Url::parse("inmemory:///test").unwrap();
        let language_id = "sway";
        let text = String::from("fn main{}");
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

        // send "shutdown" request
        let shutdown = Request::build("shutdown").id(1).finish();
        let response = service.ready().await.unwrap().call(shutdown.clone()).await;
        let ok = Response::from_ok(1.into(), json!(null));
        assert_eq!(response, Ok(Some(ok)));

        // send "exit" request
        let shutdown = Request::build("exit").finish();
        let response = service.ready().await.unwrap().call(shutdown.clone()).await;
        assert_eq!(response, Ok(None));
    }
}
