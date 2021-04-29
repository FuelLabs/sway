use lspower::jsonrpc::Result;
use lspower::lsp::*;
use lspower::{Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.client.log_message(MessageType::Info, "Initializing the Server").await;

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Incremental)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![],
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "Server initialized!")
            .await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::Info, "File changed!").await;
    }

    
    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::Info, "File opened!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
