//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles requests.

use crate::{
    capabilities, core::session::{self, build_plan, program_id_from_path}, lsp_ext, server_state::ServerState, utils::debug,
};
use forc_tracing::{tracing_subscriber, FmtSpan, TracingWriter};
use lsp_types::{
    CodeLens, CompletionResponse, DocumentFormattingParams, DocumentSymbolResponse,
    InitializeResult, InlayHint, InlayHintParams, PrepareRenameResponse, RenameParams,
    SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensRangeResult,
    SemanticTokensResult, TextDocumentIdentifier, Url, WorkspaceEdit,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf}, sync::Arc,
};
use sway_types::{Ident, Spanned};
use sway_utils::PerformanceData;
use tower_lsp::jsonrpc::Result;
use tracing::metadata::LevelFilter;

pub fn handle_initialize(
    state: &ServerState,
    params: &lsp_types::InitializeParams,
) -> Result<InitializeResult> {
    if let Some(initialization_options) = &params.initialization_options {
        let mut config = state.config.write();
        *config = serde_json::from_value(initialization_options.clone())
            .ok()
            .unwrap_or_default();
    }

    // Start a thread that will shutdown the server if the client process is no longer active.
    if let Some(client_pid) = params.process_id {
        state.spawn_client_heartbeat(client_pid as usize);
    }

    // Initializing tracing library based on the user's config
    let config = state.config.read();
    if config.logging.level != LevelFilter::OFF {
        tracing_subscriber::fmt::Subscriber::builder()
            .with_ansi(false)
            .with_max_level(config.logging.level)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(TracingWriter::Stderr)
            .init();
    }
    tracing::info!("Initializing the Sway Language Server");

    Ok(InitializeResult {
        server_info: None,
        capabilities: crate::server_capabilities(),
        ..InitializeResult::default()
    })
}

pub async fn handle_document_symbol(
    state: &ServerState,
    params: lsp_types::DocumentSymbolParams,
) -> Result<Option<lsp_types::DocumentSymbolResponse>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((uri, session)) => Ok(session::document_symbols(&uri, &state.token_map, &state.engines.read(), &state.compiled_programs)
            .map(DocumentSymbolResponse::Nested)),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub fn handle_goto_definition(
    state: &ServerState,
    params: lsp_types::GotoDefinitionParams,
) -> Result<Option<lsp_types::GotoDefinitionResponse>> {
    match state.sync_and_uri_from_workspace(
        &params.text_document_position_params.text_document.uri,
    ) {
        Ok((sync, uri)) => {
            let position = params.text_document_position_params.position;
            Ok(session::token_definition_response(
                &uri,
                position,
                &state.engines.read(),
                &state.token_map,
                &sync,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub fn handle_completion(
    state: &ServerState,
    params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionResponse>> {
    let trigger_char = params
        .context
        .as_ref()
        .and_then(|ctx| ctx.trigger_character.as_deref())
        .unwrap_or("");
    let position = params.text_document_position.position;
    match state.uri_and_session_from_workspace(&params.text_document_position.text_document.uri) {
        Ok((uri, session)) => Ok(session::completion_items(
                &uri,
                position,
                trigger_char,
                &state.token_map,
                &state.engines.read(),
                &state.compiled_programs,
            )
            .map(CompletionResponse::Array)),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub fn handle_hover(
    state: &ServerState,
    params: lsp_types::HoverParams,
) -> Result<Option<lsp_types::Hover>> {
    match state.sync_and_uri_from_workspace(
        &params.text_document_position_params.text_document.uri,
    ) {
        Ok((sync, uri)) => {
            let position = params.text_document_position_params.position;
            Ok(capabilities::hover::hover_data(
                state,
                sync,
                &state.engines.read(),
                &uri,
                position,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub fn handle_prepare_rename(
    state: &ServerState,
    params: lsp_types::TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>> {
    match state.sync_and_uri_from_workspace(&params.text_document.uri) {
        Ok((sync, uri)) => capabilities::rename::prepare_rename(
            &state.engines.read(),
            &state.token_map,
            &uri,
            params.position,
            &sync,
        )
        .map(Some)
        .or_else(|e| {
            tracing::error!("{}", e);
            Ok(None)
        }),
        Err(e) => {
            tracing::error!("{}", e);
            Ok(None)
        }
    }
}

pub fn handle_rename(state: &ServerState, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
    match state.sync_and_uri_from_workspace(&params.text_document_position.text_document.uri) {
        Ok((sync, uri)) => {
            let new_name = params.new_name;
            let position = params.text_document_position.position;
            capabilities::rename::rename(
                &state.engines.read(),
                &state.token_map,
                new_name,
                &uri,
                position,
                &sync,
            )
            .map(Some)
            .or_else(|e| {
                tracing::error!("{}", e);
                Ok(None)
            })
        }
        Err(e) => {
            tracing::error!("{}", e);
            Ok(None)
        }
    }
}

pub async fn handle_document_highlight(
    state: &ServerState,
    params: lsp_types::DocumentHighlightParams,
) -> Result<Option<Vec<lsp_types::DocumentHighlight>>> {
    let _ = state.wait_for_parsing().await;
    match state
        .uri_and_session_from_workspace(&params.text_document_position_params.text_document.uri)
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            Ok(capabilities::highlight::get_highlights(
                &state.engines.read(),
                &state.token_map,
                &uri,
                position,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_references(
    state: &ServerState,
    params: lsp_types::ReferenceParams,
) -> Result<Option<Vec<lsp_types::Location>>> {
    let _ = state.wait_for_parsing().await;
    match state
        .sync_and_uri_from_workspace(&params.text_document_position.text_document.uri)
    {
        Ok((sync, uri)) => {
            let position = params.text_document_position.position;
            Ok(session::token_references(
                &uri,
                position,
                &state.token_map,
                &state.engines.read(),
                &sync,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_formatting(
    state: &ServerState,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<lsp_types::TextEdit>>> {
    let _ = state.wait_for_parsing().await;
    state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .and_then(|(uri, _)| {
            capabilities::formatting::format_text(&state.documents, &uri).map(Some)
        })
        .or_else(|err| {
            tracing::error!("{}", err.to_string());
            Ok(None)
        })
}

pub async fn handle_code_action(
    state: &ServerState,
    params: lsp_types::CodeActionParams,
) -> Result<Option<lsp_types::CodeActionResponse>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((temp_uri, session)) => Ok(capabilities::code_actions(
            &state.engines.read(),
            &state.token_map,
            &params.range,
            &params.text_document.uri,
            &temp_uri,
            &params.context.diagnostics,
            &state.compiled_programs,
        )),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_code_lens(
    state: &ServerState,
    params: lsp_types::CodeLensParams,
) -> Result<Option<Vec<CodeLens>>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((url, session)) => Ok(Some(capabilities::code_lens::code_lens(&state.runnables, &url))),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_semantic_tokens_range(
    state: &ServerState,
    params: SemanticTokensRangeParams,
) -> Result<Option<SemanticTokensRangeResult>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((uri, session)) => Ok(capabilities::semantic_tokens::semantic_tokens_range(
            &state.token_map,
            &uri,
            &params.range,
        )),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_semantic_tokens_full(
    state: &ServerState,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((uri, session)) => Ok(capabilities::semantic_tokens::semantic_tokens_full(
            &state.token_map,
            &uri,
        )),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_inlay_hints(
    state: &ServerState,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>> {
    let _ = state.wait_for_parsing().await;
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((uri, _)) => {
            let config = &state.config.read().inlay_hints;
            Ok(capabilities::inlay_hints::inlay_hints(
                &state.engines.read(),
                &state.token_map,
                &uri,
                &params.range,
                config,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

/// This method is triggered by a command palette request in VScode
/// The 3 commands are: "show lexed ast", "show parsed ast" or "show typed ast"
///
/// If any of these commands are executed, the client requests this method
/// by calling the "sway/show_ast".
///
/// The function expects the URI of the current open file where the
/// request was made, and if the "lexed", "parsed" or "typed" ast was requested.
///
/// A formatted AST is written to a temporary file and the URI is
/// returned to the client so it can be opened and displayed in a
/// separate side panel.
pub fn handle_show_ast(
    state: &ServerState,
    params: &lsp_ext::ShowAstParams,
) -> Result<Option<TextDocumentIdentifier>> {
    match state.uri_and_session_from_workspace(&params.text_document.uri) {
        Ok((_, session)) => {
            let current_open_file = &params.text_document.uri;
            // Convert the Uri to a PathBuf
            let path = current_open_file.to_file_path().unwrap();
            let program_id = program_id_from_path(&path, &state.engines.read()).unwrap();

            let write_ast_to_file =
                |path: &Path, ast_string: &String| -> Option<TextDocumentIdentifier> {
                    if let Ok(mut file) = File::create(path) {
                        let _ = writeln!(&mut file, "{ast_string}");
                        if let Ok(uri) = Url::from_file_path(path) {
                            // Return the tmp file path where the AST has been written to.
                            return Some(TextDocumentIdentifier::new(uri));
                        }
                    }
                    None
                };

            // Returns true if the current path matches the path of a submodule
            let path_is_submodule = |ident: &Ident, path: &PathBuf| -> bool {
                ident
                    .span()
                    .source_id()
                    .map(|p| state.engines.read().se().get_path(p))
                    == Some(path.clone())
            };

            let ast_path = PathBuf::from(params.save_path.path());
            {
                let program = state.compiled_programs.get(&program_id).unwrap();
                match params.ast_kind.as_str() {
                    "lexed" => {
                        let lexed_program = program.value().lexed.clone();
                        Ok({
                            let mut formatted_ast = format!("{:#?}", program.lexed);
                            for (ident, submodule) in &lexed_program.root.submodules {
                                if path_is_submodule(ident, &path) {
                                    // overwrite the root AST with the submodule AST
                                    formatted_ast = format!("{:#?}", submodule.module.tree);
                                }
                            }
                            write_ast_to_file(ast_path.join("lexed.rs").as_path(), &formatted_ast)
                        })
                    }
                    "parsed" => {
                        let parsed_program = program.value().parsed.clone();
                        Ok({
                            // Initialize the string with the AST from the root
                            let mut formatted_ast =
                                format!("{:#?}", parsed_program.root.tree.root_nodes);
                            for (ident, submodule) in &parsed_program.root.submodules {
                                if path_is_submodule(ident, &path) {
                                    // overwrite the root AST with the submodule AST
                                    formatted_ast =
                                        format!("{:#?}", submodule.module.tree.root_nodes);
                                }
                            }
                            write_ast_to_file(ast_path.join("parsed.rs").as_path(), &formatted_ast)
                        })
                    }
                    "typed" => {
                        let typed_program = program.value().typed.as_ref().unwrap();
                        Ok({
                            // Initialize the string with the AST from the root
                            let mut formatted_ast = debug::print_decl_engine_types(
                                &typed_program.root_module.all_nodes,
                                state.engines.read().de(),
                            );
                            for (ident, submodule) in &typed_program.root_module.submodules {
                                if path_is_submodule(ident, &path) {
                                    // overwrite the root AST with the submodule AST
                                    formatted_ast = debug::print_decl_engine_types(
                                        &submodule.module.all_nodes,
                                        state.engines.read().de(),
                                    );
                                }
                            }
                            write_ast_to_file(ast_path.join("typed.rs").as_path(), &formatted_ast)
                        })
                    }
                    _ => Ok(None),
                }
            }
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

/// This method is triggered when the use hits enter or pastes a newline in the editor.
pub fn handle_on_enter(
    state: &ServerState,
    params: &lsp_ext::OnEnterParams,
) -> Result<Option<WorkspaceEdit>> {
    match state.sync_and_uri_from_workspace(&params.text_document.uri) {
        Ok((_, uri)) => {
            // handle on_enter capabilities if they are enabled
            Ok(capabilities::on_enter(
                &state.config.read().on_enter,
                &state.documents,
                &uri,
                params,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

/// Returns a [String] of the GraphViz DOT representation of a graph.
pub fn handle_visualize(
    _state: &ServerState,
    params: &lsp_ext::VisualizeParams,
) -> Result<Option<String>> {
    match params.graph_kind.as_str() {
        "build_plan" => match build_plan(&params.text_document.uri) {
            Ok(build_plan) => Ok(Some(
                build_plan.visualize(Some("vscode://file".to_string())),
            )),
            Err(err) => {
                tracing::error!("{}", err.to_string());
                Ok(None)
            }
        },
        _ => Ok(None),
    }
}

/// This method is triggered by the test suite to request the latest compilation metrics.
pub(crate) fn metrics(
    state: &ServerState,
    // TODO: this seems wrong. why aren't we using the params?
    params: &lsp_ext::MetricsParams,
) -> Result<Option<Vec<(String, PerformanceData)>>> {
    let mut metrics = vec![];
    for item in state.compiled_programs.iter() {
        let path = state
            .engines
            .read()
            .se()
            .get_manifest_path_from_program_id(item.key())
            .unwrap()
            .to_string_lossy()
            .to_string();
        metrics.push((path, item.value().metrics.clone()));
    }
    Ok(Some(metrics))
}
