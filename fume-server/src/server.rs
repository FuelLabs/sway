use lspower::{
    jsonrpc,
    lsp::{self, CompletionParams, CompletionResponse},
    Client, LanguageServer,
};
use std::sync::Arc;

use lsp::{Hover, HoverParams, InitializeParams, InitializeResult, MessageType};

use crate::{capabilities, session::Session};

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
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        self.client
            .log_message(MessageType::Info, "Initializing the Server")
            .await;

        Ok(lsp::InitializeResult {
            server_info: None,
            capabilities: lsp::ServerCapabilities {
                text_document_sync: Some(lsp::TextDocumentSyncCapability::Kind(
                    lsp::TextDocumentSyncKind::Incremental,
                )),
                completion_provider: Some(lsp::CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(lsp::ExecuteCommandOptions {
                    commands: vec![],
                    ..Default::default()
                }),
                workspace: Some(lsp::WorkspaceServerCapabilities {
                    workspace_folders: Some(lsp::WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(lsp::OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..lsp::ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: lsp::InitializedParams) {
        self.client
            .log_message(MessageType::Info, "Server initialized!")
            .await;
    }

    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "File opened!")
            .await;

        self.session.store_document(params.text_document).unwrap();

        self.client
            .log_message(MessageType::Info, "File stored!")
            .await;

        // 1. check the path of the document
        // 2. ignore if it does not exist 
        // 3. 
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        Ok(capabilities::completion::completion(
            self.session.clone(),
            params,
        ))
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "File changed!")
            .await;

        let session = self.session.clone();
        let document = session.get_document(&params.text_document.uri).unwrap();

        // self.client
        //     .log_message(MessageType::Info, format!("got the document {}", document))
        //     .await;
    }

    async fn did_save(&self, _params: lsp::DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::Info, "File saved!")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        todo!()
    }
}
