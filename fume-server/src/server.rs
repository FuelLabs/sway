use lspower::{
    jsonrpc,
    lsp::{
        self, SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions,
        SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    },
    Client, LanguageServer,
};
use std::sync::Arc;

use lsp::{
    CompletionParams, CompletionResponse, Hover, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, MessageType, OneOf,
};

use crate::core::session::Session;
use crate::{capabilities, core::document::DocumentError};

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

    fn get_semantic_tokens() -> Option<SemanticTokensServerCapabilities> {
        let token_types = vec![
            SemanticTokenType::CLASS,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::STRING,
            SemanticTokenType::TYPE,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::VARIABLE,
        ];

        let token_modifiers: Vec<SemanticTokenModifier> = vec![
            // declaration of symbols
            SemanticTokenModifier::DECLARATION,
            // definition of symbols as in header files
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
            // for variable references where the variable is assigned to
            SemanticTokenModifier::MODIFICATION,
            SemanticTokenModifier::DOCUMENTATION,
            // for symbols that are part of stdlib
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ];

        let legend = SemanticTokensLegend {
            token_types,
            token_modifiers,
        };

        let options = SemanticTokensOptions {
            legend,
            range: None,
            full: Some(SemanticTokensFullOptions::Bool(true)),
            ..Default::default()
        };

        Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            options,
        ))
    }
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        self.client
            .log_message(MessageType::Info, "Initializing the Server")
            .await;

        Ok(lsp::InitializeResult {
            server_info: None,
            capabilities: lsp::ServerCapabilities {
                text_document_sync: Some(lsp::TextDocumentSyncCapability::Kind(
                    lsp::TextDocumentSyncKind::Incremental,
                )),
                semantic_tokens_provider: Backend::get_semantic_tokens(),
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

        let url = params.text_document.uri.clone();

        if let Ok(_) = self.session.store_document(&params.text_document) {
            if let Err(DocumentError::FailedToParse(diagnostics)) =
                self.session.parse_document(&url)
            {
                self.client
                    .publish_diagnostics(params.text_document.uri, diagnostics, None)
                    .await;
            }
        };
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        self.log_info_message("File changed").await;
        self.session
            .update_text_document(&params.text_document.uri, params.content_changes)
            .unwrap();
    }

    async fn did_save(&self, params: lsp::DidSaveTextDocumentParams) {
        self.log_info_message("File saved").await;

        let url = params.text_document.uri.clone();
        self.client.publish_diagnostics(url, vec![], None).await;

        if let Err(DocumentError::FailedToParse(diagnostics)) =
            self.session.parse_document(&params.text_document.uri)
        {
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: lsp::DidCloseTextDocumentParams) {
        self.log_info_message("Closing a document").await;

        match self.session.remove_document(&params.text_document.uri) {
            Ok(_) => self.log_info_message("Document closed").await,
            _ => self.log_info_message("Document previously closed").await,
        };
    }

    // refer to this
    // https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
    async fn semantic_tokens_full(
        &self,
        _params: lsp::SemanticTokensParams,
    ) -> jsonrpc::Result<Option<lsp::SemanticTokensResult>> {
        Ok(None)
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

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let position = params.text_document_position_params.position;
        let url = &params.text_document_position_params.text_document.uri;

        self.log_info_message(&format!("position is {:?}", position))
            .await;

        match self.session.get_token_from_position(url, position) {
            Some(token) => {
                self.log_info_message(&format!("token found is at {:?}", token.range))
                    .await;
                Ok(capabilities::hover::get_hover_data(token))
            }
            _ => Ok(None),
        }
    }

    async fn document_highlight(
        &self,
        _params: lsp::DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<lsp::DocumentHighlight>>> {
        // TODO
        // 1. find exact value of the highlight
        // 2. find it's matches in the document - convert to Range
        // 3. return the Vector of those ranges
        Ok(None)
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
        Ok(None)
    }

    async fn goto_declaration(
        &self,
        _params: lsp::request::GotoDeclarationParams,
    ) -> jsonrpc::Result<Option<lsp::request::GotoDeclarationResponse>> {
        // TODO
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: lsp::GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<lsp::GotoDefinitionResponse>> {
        // TODO
        Ok(None)
    }

    async fn goto_type_definition(
        &self,
        _params: lsp::request::GotoTypeDefinitionParams,
    ) -> jsonrpc::Result<Option<lsp::request::GotoTypeDefinitionResponse>> {
        // TODO
        Ok(None)
    }
}
