use lspower::{jsonrpc, lsp, Client, LanguageServer};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use lsp::{
    CompletionParams, CompletionResponse, Hover, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, MessageType, OneOf,
};

use crate::core::{document::DocumentError, session::Session};
use crate::{capabilities, core::document::TextDocument};

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

    fn parse_and_store_sway_files(&self) -> Result<(), DocumentError> {
        let curr_dir = std::env::current_dir().unwrap();

        match find_manifest_dir(&curr_dir) {
            Some(path) => {
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
            _ => {}
        }

        Ok(())
    }
}

// todo: code-duplication from @forc, extract to separate lib
fn get_sway_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut dir_entries = vec![path];

    while let Some(next_dir) = dir_entries.pop() {
        if let Ok(read_dir) = fs::read_dir(next_dir) {
            for inner_entry in read_dir {
                if let Ok(entry) = inner_entry {
                    let path = entry.path();

                    if path.is_dir() {
                        dir_entries.push(path);
                    } else {
                        if is_sway_file(&path) {
                            files.push(path)
                        }
                    }
                }
            }
        }
    }

    files
}

// todo: code-duplication from @forc, extract to separate lib
fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new("sw")) == res
}

// todo: code-duplication from @forc, extract to separate lib
fn find_manifest_dir(starter_path: &PathBuf) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path.clone()).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push("Forc.toml");
        if path.exists() {
            path.pop();
            return Some(path);
        } else {
            path.pop();
            path.pop();
        }
    }
    None
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        self.client
            .log_message(MessageType::Info, "Initializing the Server")
            .await;

        // iterate over the project dir, parse all sway files
        let _ = self.parse_and_store_sway_files();

        Ok(lsp::InitializeResult {
            server_info: None,
            capabilities: lsp::ServerCapabilities {
                text_document_sync: Some(lsp::TextDocumentSyncCapability::Kind(
                    lsp::TextDocumentSyncKind::Incremental,
                )),
                definition_provider: Some(lsp::OneOf::Left(true)),
                semantic_tokens_provider: capabilities::semantic_tokens::get_semantic_tokens(),
                document_symbol_provider: Some(lsp::OneOf::Left(true)),
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
                document_formatting_provider: Some(OneOf::Left(true)),
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
        if let Some(diagnostics) =
            capabilities::text_sync::handle_open_file(self.session.clone(), &params)
        {
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        let _ = capabilities::text_sync::handle_change_file(self.session.clone(), params);
    }

    async fn did_save(&self, params: lsp::DidSaveTextDocumentParams) {
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
        params: lsp::DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<lsp::DocumentSymbolResponse>> {
        Ok(capabilities::document_symbol::document_symbol(
            self.session.clone(),
            params.text_document.uri,
        ))
    }

    async fn semantic_tokens_full(
        &self,
        params: lsp::SemanticTokensParams,
    ) -> jsonrpc::Result<Option<lsp::SemanticTokensResult>> {
        Ok(capabilities::semantic_tokens::get_semantic_tokens_full(
            self.session.clone(),
            params,
        ))
    }

    async fn document_highlight(
        &self,
        params: lsp::DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<lsp::DocumentHighlight>>> {
        Ok(capabilities::highlight::get_highlights(
            self.session.clone(),
            params,
        ))
    }

    async fn goto_definition(
        &self,
        params: lsp::GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<lsp::GotoDefinitionResponse>> {
        Ok(capabilities::go_to::go_to_definition(
            self.session.clone(),
            params,
        ))
    }

    async fn formatting(
        &self,
        params: lsp::DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<lsp::TextEdit>>> {
        Ok(capabilities::formatting::format_document(
            self.session.clone(),
            params,
        ))
    }
}
