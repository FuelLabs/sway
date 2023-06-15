pub use crate::error::DocumentError;
use crate::{
    capabilities,
    config::{Config, Warnings},
    core::{session::Session, sync},
    error::{DirectoryError, LanguageServerError},
    handlers::{notification, request},
    utils::{debug, keyword_docs::KeywordDocs}, global_state::GlobalState,
};
use dashmap::DashMap;
use forc_pkg::manifest::PackageManifestFile;
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use lsp_types::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_types::{Ident, Spanned};
use tokio::task;
use tower_lsp::{jsonrpc, Client, LanguageServer};
use tracing::metadata::LevelFilter;


/// Returns the capabilities of the server to the client,
/// indicating its support for various language server protocol features.
pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(false),
        }),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_string()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![],
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
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
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        ..ServerCapabilities::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GlobalState {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        if let Some(initialization_options) = &params.initialization_options {
            let mut config = self.config.write();
            *config = serde_json::from_value(initialization_options.clone())
                .ok()
                .unwrap_or_default();
        }

        // Initalizing tracing library based on the user's config
        let config = self.config.read();
        if config.logging.level != LevelFilter::OFF {
            let tracing_options = TracingSubscriberOptions {
                log_level: Some(config.logging.level),
                writer_mode: Some(TracingWriterMode::Stderr),
                ..Default::default()
            };
            init_tracing_subscriber(tracing_options);
        }

        tracing::info!("Initializing the Sway Language Server");

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
        notification::handle_did_open_text_document(&self, params);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        notification::handle_did_change_text_document(&self, params);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        notification::handle_did_save_text_document(&self, params);
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        notification::handle_did_change_watched_files(&self, params);
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        request::handle_hover(self.snapshot(), params)
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        request::handle_code_action(self.snapshot(), params)
    }

    async fn code_lens(&self, params: CodeLensParams) -> jsonrpc::Result<Option<Vec<CodeLens>>> {
        request::handle_code_lens(self.snapshot(), params)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        request::handle_completion(self.snapshot(), params)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        request::handle_document_symbol(self.snapshot(), params)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> jsonrpc::Result<Option<SemanticTokensResult>> {
        request::handle_semantic_tokens_full(self.snapshot(), params)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        request::handle_document_highlight(self.snapshot(), params)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        request::handle_goto_definition(self.snapshot(), params)
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        request::handle_formatting(self.snapshot(), params)
    }

    async fn rename(&self, params: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        request::handle_rename(self.snapshot(), params)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> jsonrpc::Result<Option<PrepareRenameResponse>> {
        request::handle_prepare_rename(self.snapshot(), params)
    }
}
