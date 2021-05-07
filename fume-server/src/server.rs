use lspower::{
    jsonrpc,
    lsp::{self},
    Client, LanguageServer,
};
use std::sync::Arc;

use lsp::{
    CompletionParams, CompletionResponse, Hover, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, MessageType, OneOf,
};

use crate::capabilities;
use crate::core::session::Session;

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
        self.client.log_message(MessageType::Info, message).await;
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(lsp::CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(lsp::ExecuteCommandOptions {
                    commands: vec![],
                    ..Default::default()
                }),
                document_highlight_provider: Some(OneOf::Left(true)),
                workspace: Some(lsp::WorkspaceServerCapabilities {
                    workspace_folders: Some(lsp::WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
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

    // Document Handlers
    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        self.log_info_message("File opened").await;

        match self.session.store_document(&params.text_document) {
            Ok(()) => {
                if let Some(diagnostics) =
                    capabilities::diagnostic::perform_diagnostics(&params.text_document.text)
                {
                    self.log_info_message(&format!("found {} error", diagnostics.len()))
                        .await;
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                } else {
                    self.log_info_message("no errors in sight").await;
                }

                self.log_info_message("File stored").await;
            }
            _ => {}
        }
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        self.log_info_message("File changed").await;
        self.session
            .update_document(params.text_document.uri, params.content_changes)
            .unwrap();
    }

    async fn did_save(&self, params: lsp::DidSaveTextDocumentParams) {
        self.log_info_message("File changed").await;

        let uri = params.text_document.uri.clone();
        self.client.publish_diagnostics(uri, vec![], None).await;

        match self
            .session
            .get_document_text_as_string(&params.text_document.uri)
        {
            Ok(document) => {
                if let Some(diagnostics) = capabilities::diagnostic::perform_diagnostics(&document)
                {
                    self.log_info_message(&format!("found {} error", diagnostics.len()))
                        .await;
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                } else {
                    self.log_info_message("no errors in sight").await;
                }
            }
            _ => {}
        }
    }

    async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
        self.log_info_message("Closing a document").await;

        match self.session.remove_document(&params.text_document.uri) {
            Ok(_) => self.log_info_message("Document closed").await,
            _ => self.log_info_message("Document previously closed").await,
        };
    }

    // Completion
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

    async fn hover(&self, _params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        // TODO
        // 0. on document open / save -> parse and store all the values and their metadata
        // 1. get the document
        // 2. find exact value of the hover
        // 3. return info of the hovered Value and it's metadata
        todo!()
    }

    async fn document_highlight(
        &self,
        _params: lsp::DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<lsp::DocumentHighlight>>> {
        // TODO
        // 1. find exact value of the highlight
        // 2. find it's matches in the document - convert to Range
        // 3. return the Vector of those ranges
        todo!()
    }

    async fn document_symbol(
        &self,
        _params: lsp::DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<lsp::DocumentSymbolResponse>> {
        // TODO
        // 0. on document open / save -> parse it and store all the values and their metada
        // 1. get the stored document
        // 2. get all symbols of the document that was previously stored
        // 3. return the Vector of symbols
        todo!()
    }

    async fn goto_declaration(
        &self,
        _params: lsp::request::GotoDeclarationParams,
    ) -> jsonrpc::Result<Option<lsp::request::GotoDeclarationResponse>> {
        todo!()
    }

    async fn goto_definition(
        &self,
        _params: lsp::GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<lsp::GotoDefinitionResponse>> {
        todo!()
    }

    async fn goto_type_definition(
        &self,
        _params: lsp::request::GotoTypeDefinitionParams,
    ) -> jsonrpc::Result<Option<lsp::request::GotoTypeDefinitionResponse>> {
        todo!()
    }
}
