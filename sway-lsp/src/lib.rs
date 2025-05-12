#![recursion_limit = "256"]

pub mod capabilities;
pub mod config;
pub mod core;
pub mod error;
pub mod server_state;
pub mod handlers {
    pub mod notification;
    pub mod request;
}
pub mod lsp_ext;
pub mod server;
mod traverse;
pub mod utils;

use lsp_types::{
    CodeActionProviderCapability, CodeLensOptions, CompletionOptions, ExecuteCommandOptions,
    HoverProviderCapability, OneOf, RenameOptions, SemanticTokensLegend, SemanticTokensOptions,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use server_state::ServerState;
use tower_lsp::{LspService, Server};

pub async fn start() {
    let (service, socket) = LspService::build(ServerState::new)
        .custom_method("sway/show_ast", ServerState::show_ast)
        .custom_method("sway/visualize", ServerState::visualize)
        .custom_method("sway/on_enter", ServerState::on_enter)
        .custom_method("sway/metrics", ServerState::metrics)
        .finish();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

/// Returns the capabilities of the server to the client,
/// indicating its support for various language server protocol features.
pub fn server_capabilities() -> ServerCapabilities {
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
        references_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: capabilities::semantic_tokens::SUPPORTED_TYPES.to_vec(),
                    token_modifiers: capabilities::semantic_tokens::SUPPORTED_MODIFIERS.to_vec(),
                },
                range: Some(true),
                ..Default::default()
            }
            .into(),
        ),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            ..Default::default()
        }),
        ..ServerCapabilities::default()
    }
}
