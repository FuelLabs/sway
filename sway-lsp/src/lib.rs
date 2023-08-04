#![recursion_limit = "256"]

pub mod capabilities;
pub mod config;
pub mod core;
pub mod error;
#[cfg(feature = "custom-event-loop")]
pub mod event_loop;
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
    HoverProviderCapability, OneOf, RenameOptions, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    WorkDoneProgressOptions,
};
use server_state::ServerState;
use tower_lsp::{LspService, Server};

pub async fn start() {
    let (service, socket) = LspService::build(ServerState::new)
        .custom_method("sway/show_ast", ServerState::show_ast)
        .custom_method("sway/on_enter", ServerState::on_enter)
        .finish();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

/// Returns the capabilities of the server to the client,
/// indicating its support for various language server protocol features.
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        // code_lens_provider: Some(CodeLensOptions {
        //     resolve_provider: Some(false),
        // }),
        // completion_provider: Some(CompletionOptions {
        //     trigger_characters: Some(vec![".".to_string()]),
        //     ..Default::default()
        // }),
        // definition_provider: Some(OneOf::Left(true)),
        // document_formatting_provider: Some(OneOf::Left(true)),
        // document_highlight_provider: Some(OneOf::Left(true)),
        // document_symbol_provider: Some(OneOf::Left(true)),
        // execute_command_provider: Some(ExecuteCommandOptions {
        //     commands: vec![],
        //     ..Default::default()
        // }),
        // hover_provider: Some(HoverProviderCapability::Simple(true)),
        // inlay_hint_provider: Some(OneOf::Left(true)),
        // rename_provider: Some(OneOf::Right(RenameOptions {
        //     prepare_provider: Some(true),
        //     work_done_progress_options: WorkDoneProgressOptions {
        //         work_done_progress: Some(true),
        //     },
        // })),



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

use crate::{config::Config, event_loop::main_loop};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use lsp_server::Connection;
use tracing::metadata::LevelFilter;

pub fn start_custom_event_loop() {
    eprintln!("Custom event loop :)");
    
    tracing::info!("Initializing the Sway Language Server");

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start().unwrap_or_else(|err| {
        panic!("failed to initialize sway-lsp: {:?}", err);
    });

    tracing::info!("InitializeParams: {}", initialize_params);
    let lsp_types::InitializeParams {
        initialization_options,
        client_info,
        ..
    } = event_loop::from_json::<lsp_types::InitializeParams>(
        "InitializeParams",
        &initialize_params,
    )
    .unwrap_or_else(|err| {
        panic!("failed to destialize initialization params: {:?}", err);
    });

    let mut config: crate::config::Config = Default::default();
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
        capabilities: server_capabilities(),
        ..lsp_types::InitializeResult::default()
    };
    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    if let Err(err) = connection.initialize_finish(initialize_id, initialize_result) {
        tracing::error!("{}", err.to_string().as_str());
    } else {
        tracing::info!("Sway Language Server Initialized");
    }

    if let Some(client_info) = client_info {
        tracing::info!(
            "Client '{}' {}",
            client_info.name,
            client_info.version.unwrap_or_default()
        );
    }

    // Run the main loop
    if let Err(err) = main_loop::run(config, connection) {
        tracing::error!("{}", err.to_string().as_str());
    }

    if let Err(err) = io_threads.join() {
        tracing::error!("IoThreads join error {}", err.to_string().as_str());
    }

    tracing::info!("server did shut down");
}
