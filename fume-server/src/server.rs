use lspower::{
    jsonrpc,
    lsp::{self, CompletionParams, CompletionResponse},
    Client, LanguageServer,
};
use std::sync::Arc;

use lsp::{Hover, HoverParams, InitializeParams, InitializeResult, MessageType};

use crate::capabilities;
use crate::core::{session::Session};

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
        self.client
            .log_message(MessageType::Info, message)
            .await;
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

    // LSP-Server Lifecycle
    async fn initialized(&self, _: lsp::InitializedParams) {
        self.log_info_message("Server initialized").await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.log_info_message("Shutting the server").await;
        Ok(())
    }

    // Text Handlers
    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        self.log_info_message("File opened").await;

        match self.session.store_document(&params.text_document) {
            Ok(()) => {
                if let Some(diagnostics) = capabilities::diagnostic::perform_diagnostics(&params.text_document.text) {
                    self.log_info_message(&format!("found {} error", diagnostics.len())).await;
                    self.client.publish_diagnostics(params.text_document.uri, diagnostics, None).await;
                } else {
                    self.log_info_message("no errors in sight").await;    
                }

                self.log_info_message("File stored").await;
            },
            _ => {}
        }
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        self.log_info_message("File changed").await;
        self.session.update_document(params.text_document.uri, params.content_changes).unwrap();
    }

    async fn did_save(&self, params: lsp::DidSaveTextDocumentParams) {
        self.log_info_message("File changed").await;

        let uri = params.text_document.uri.clone();
        self.client.publish_diagnostics(uri, vec![], None).await;

        match self.session.get_document_text(&params.text_document.uri) {
            Ok(document) => {
                if let Some(diagnostics) = capabilities::diagnostic::perform_diagnostics(&document) {
                    self.log_info_message(&format!("found {} error", diagnostics.len())).await;
                    self.client.publish_diagnostics(params.text_document.uri, diagnostics, None).await;
                } else {
                    self.log_info_message("no errors in sight").await;    
                }
            },
            _ => {}
        }
    }

    async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
        self.log_info_message("Closing a document").await;
        
        match self.session.remove_document(&params.text_document.uri) {
            Ok(_) => self.log_info_message("Document closed").await,
            _ => self.log_info_message("Document previously closed").await
        };
    }

    // Completion
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        Ok(capabilities::completion::get_completion(
            self.session.clone(),
            params,
        ))
    }

    async fn completion_resolve(&self, _params: lsp::CompletionItem) -> jsonrpc::Result<lsp::CompletionItem> {
        todo!()
    }

    // OPTINALS
    async fn hover(&self, _params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        todo!()
    }

    async fn did_change_workspace_folders(&self, _params: lsp::DidChangeWorkspaceFoldersParams) {
        todo!()
    }

    async fn symbol(&self, _params: lsp::WorkspaceSymbolParams) -> jsonrpc::Result<Option<Vec<lsp::SymbolInformation>>> {
        todo!()
    }

    async fn document_highlight(&self, _params: lsp::DocumentHighlightParams) -> jsonrpc::Result<Option<Vec<lsp::DocumentHighlight>>> {
        todo!()
    }

    async fn execute_command(&self, _params: lsp::ExecuteCommandParams) -> jsonrpc::Result<Option<serde_json::Value>> {
        todo!()
    }

    async fn code_action(&self, _params: lsp::CodeActionParams) -> jsonrpc::Result<Option<lsp::CodeActionResponse>> {
        todo!()
    }

    async fn signature_help(&self, _params: lsp::SignatureHelpParams) -> jsonrpc::Result<Option<lsp::SignatureHelp>> {
        todo!()
    }

    async fn range_formatting(&self, _params: lsp::DocumentRangeFormattingParams) -> jsonrpc::Result<Option<Vec<lsp::TextEdit>>> {
        todo!()
    }

    async fn formatting(&self, _params: lsp::DocumentFormattingParams) -> jsonrpc::Result<Option<Vec<lsp::TextEdit>>> {
        todo!()
    }

    async fn references(&self, _params: lsp::ReferenceParams) -> jsonrpc::Result<Option<Vec<lsp::Location>>> {
        todo!()
    }

    async fn rename(&self, _params: lsp::RenameParams) -> jsonrpc::Result<Option<lsp::WorkspaceEdit>> {
        todo!()
    }

    async fn document_symbol(&self, _params: lsp::DocumentSymbolParams) -> jsonrpc::Result<Option<lsp::DocumentSymbolResponse>> {
        todo!()
    }
}
