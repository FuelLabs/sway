//! This module implements the [LanguageServer] trait for [GlobalState].
//! It provides an interface between the LSP protocol and the sway-lsp internals.

use crate::{
    global_state::GlobalState,
    handlers::{notification, request},
    lsp_ext::ShowAstParams,
};
use lsp_types::{
    CodeActionParams, CodeActionResponse, CodeLens, CodeLensParams, CompletionParams,
    CompletionResponse, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentFormattingParams,
    DocumentHighlight, DocumentHighlightParams, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, InitializeParams,
    InitializeResult, InitializedParams, InlayHint, InlayHintParams, PrepareRenameResponse,
    RenameParams, SemanticTokensParams, SemanticTokensResult, TextDocumentIdentifier,
    TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};
use tower_lsp::{jsonrpc::Result, LanguageServer};

#[tower_lsp::async_trait]
impl LanguageServer for GlobalState {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        request::handle_initialize(self, params)
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("Sway Language Server Initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown_server()
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        notification::handle_did_open_text_document(self, params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        notification::handle_did_change_text_document(self, params).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        notification::handle_did_save_text_document(self, params).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        notification::handle_did_change_watched_files(self, params).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        request::handle_hover(self.snapshot(), params)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        request::handle_code_action(self.snapshot(), params)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        request::handle_code_lens(self.snapshot(), params)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        request::handle_completion(self.snapshot(), params)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        request::handle_document_symbol(self.snapshot(), params)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        request::handle_semantic_tokens_full(self.snapshot(), params)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        request::handle_document_highlight(self.snapshot(), params)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        request::handle_goto_definition(self.snapshot(), params)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        request::handle_formatting(self.snapshot(), params)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        request::handle_rename(self.snapshot(), params)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        request::handle_prepare_rename(self.snapshot(), params)
    }
}

// Custom LSP-Server Methods
impl GlobalState {
    pub async fn inlay_hints(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        request::handle_inlay_hints(self.snapshot(), params)
    }

    pub async fn show_ast(&self, params: ShowAstParams) -> Result<Option<TextDocumentIdentifier>> {
        request::handle_show_ast(self.snapshot(), params)
    }
}
