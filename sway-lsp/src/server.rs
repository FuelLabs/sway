//! This module implements the [LanguageServer] trait for [ServerState].
//! It provides an interface between the LSP protocol and the sway-lsp internals.

use crate::{
    handlers::{notification, request},
    lsp_ext::{OnEnterParams, ShowAstParams, VisualizeParams},
    server_state::ServerState,
};
use lsp_types::{
    CodeActionParams, CodeActionResponse, CodeLens, CodeLensParams, CompletionParams,
    CompletionResponse, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, DocumentHighlight, DocumentHighlightParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    InitializeParams, InitializeResult, InitializedParams, InlayHint, InlayHintParams, Location,
    PrepareRenameResponse, ReferenceParams, RenameParams, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult,
    TextDocumentIdentifier, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};
use sway_utils::PerformanceData;
use tower_lsp::{jsonrpc::Result, LanguageServer};

#[tower_lsp::async_trait]
impl LanguageServer for ServerState {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        request::handle_initialize(self, &params)
    }

    async fn initialized(&self, _: InitializedParams) {
        // Register a file system watcher for Forc.toml files with the client.
        if let Err(err) = self.register_forc_toml_watcher().await {
            tracing::error!("Failed to register Forc.toml file watcher: {}", err);
        }
        tracing::info!("Sway Language Server Initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown_server()
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(err) = notification::handle_did_open_text_document(self, params).await {
            tracing::error!("{}", err.to_string());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Err(err) = self
            .pid_locked_files
            .remove_dirty_flag(&params.text_document.uri)
        {
            tracing::error!("{}", err.to_string());
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(err) = notification::handle_did_change_text_document(self, params).await {
            tracing::error!("{}", err.to_string());
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Err(err) = notification::handle_did_save_text_document(self, params).await {
            tracing::error!("{}", err.to_string());
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        if let Err(err) = notification::handle_did_change_watched_files(self, params) {
            tracing::error!("{}", err.to_string());
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        request::handle_hover(self, params)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        request::handle_code_action(self, params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        request::handle_code_lens(self, params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        request::handle_completion(self, params)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        request::handle_document_symbol(self, params).await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        request::handle_semantic_tokens_full(self, params).await
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        request::handle_semantic_tokens_range(self, params).await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        request::handle_document_highlight(self, params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        request::handle_goto_definition(self, params)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        request::handle_formatting(self, params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        request::handle_rename(self, params)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        request::handle_prepare_rename(self, params)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        request::handle_inlay_hints(self, params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        request::handle_references(self, params).await
    }
}

// Custom LSP-Server Methods
impl ServerState {
    pub async fn show_ast(&self, params: ShowAstParams) -> Result<Option<TextDocumentIdentifier>> {
        request::handle_show_ast(self, &params)
    }

    pub async fn on_enter(&self, params: OnEnterParams) -> Result<Option<WorkspaceEdit>> {
        request::handle_on_enter(self, &params)
    }

    pub async fn visualize(&self, params: VisualizeParams) -> Result<Option<String>> {
        request::handle_visualize(self, &params)
    }

    pub async fn metrics(&self) -> Result<Option<Vec<(String, PerformanceData)>>> {
        request::metrics(self)
    }
}
