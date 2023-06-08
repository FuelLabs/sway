#![recursion_limit = "256"]

mod capabilities;
pub mod config;
mod core;
pub mod error;
pub mod event_loop;
pub mod lsp_ext;
pub mod server;
mod handlers {
    pub(crate) mod notification;
    pub(crate) mod request;
}
mod traverse;
pub mod utils;

use crate::{
    config::Config,
    event_loop::{main_loop, Result},
};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use lsp_server::Connection;
use lsp_types::*;
use tracing::metadata::LevelFilter;

pub fn start() -> Result<()> {
    tracing::info!("Initializing the Sway Language Server");

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start()?;
    tracing::info!("InitializeParams: {}", initialize_params);
    let lsp_types::InitializeParams {
        initialization_options,
        client_info,
        ..
    } = event_loop::from_json::<lsp_types::InitializeParams>(
        "InitializeParams",
        &initialize_params,
    )?;

    let mut config: Config = Default::default();
    if let Some(initialization_options) = &initialization_options {
        config = serde_json::from_value(initialization_options.clone())
            .ok()
            .unwrap_or_default();
    }

    // Initalizing tracing library based on the user's config
    if config.logging.level != LevelFilter::OFF {
        let tracing_options = TracingSubscriberOptions {
            log_level: Some(config.logging.level),
            writer_mode: Some(TracingWriterMode::Stderr),
            ..Default::default()
        };
        init_tracing_subscriber(tracing_options);
    }

    let initialize_result = lsp_types::InitializeResult {
        server_info: None,
        capabilities: capabilities(),
        ..InitializeResult::default()
    };
    let initialize_result = serde_json::to_value(initialize_result).unwrap();
    connection.initialize_finish(initialize_id, initialize_result)?;
    tracing::info!("Sway Language Server Initialized");

    if let Some(client_info) = client_info {
        tracing::info!(
            "Client '{}' {}",
            client_info.name,
            client_info.version.unwrap_or_default()
        );
    }

    main_loop::run(config, connection)?;

    io_threads.join()?;
    tracing::info!("server did shut down");
    Ok(())
}

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
