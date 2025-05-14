//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles requests.

use crate::{
    capabilities,
    core::{session::build_plan, sync::SyncWorkspace},
    error::{DocumentError, LanguageServerError},
    lsp_ext,
    server_state::ServerState,
    utils::debug,
};
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
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
    path::{Path, PathBuf},
    sync::Arc,
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
    if let Some(uri) = &params.root_uri {
        tracing::info!("Client reported rootUri: {}", uri);
    }

    Ok(InitializeResult {
        server_info: None,
        capabilities: crate::server_capabilities(),
        ..InitializeResult::default()
    })

    // // Determine the workspace root path.
    // let workspace_root = params
    //     .root_uri
    //     .as_ref()
    //     .and_then(|uri| uri.to_file_path().ok())
    //     .ok_or(LanguageServerError::ClientNotInitialized)?;

    // // Regardless of whether initial_path_from_client is a file or directory,
    // // use ManifestFile::from_dir to find the true project/workspace root.
    // // ManifestFile::from_dir will search upwards from initial_path_from_client (or its parent if it's a file)
    // // to find a Forc.toml.
    // let search_path_for_manifest = if workspace_root.is_file() {
    //     workspace_root.parent().unwrap_or(&workspace_root)
    // } else {
    //     &workspace_root
    // };

    // let manifest_file = ManifestFile::from_dir(search_path_for_manifest)
    // .map_err(|_e| DocumentError::ManifestFileNotFound {
    //     dir: workspace_root.to_string_lossy().to_string(),
    // })?;

    // let actual_workspace_root = manifest_file.dir().to_path_buf(); // This will be sway/examples/
    // tracing::info!("Actual workspace root determined by ManifestFile::from_dir: {:?}", actual_workspace_root);

    // // Create and initialize the global SyncWorkspace.
    // let sw = Arc::new(SyncWorkspace::new());
    // sw.create_temp_dir_from_workspace(&actual_workspace_root)?;
    // sw.clone_manifest_dir_to_temp()?;
    // sw.sync_manifest();

    // // Initialize the OnceLock for sync_workspace
    // state
    //     .sync_workspace
    //     .set(sw)
    //     .map_err(|_| LanguageServerError::SyncWorkspaceAlreadyInitialized)?;

    // Ok(())
}

pub async fn handle_document_symbol(
    state: &ServerState,
    params: lsp_types::DocumentSymbolParams,
) -> Result<Option<lsp_types::DocumentSymbolResponse>> {
    let _ = state.wait_for_parsing().await;
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, session)) => Ok(session
            .document_symbols(&uri)
            .map(DocumentSymbolResponse::Nested)),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_goto_definition(
    state: &ServerState,
    params: lsp_types::GotoDefinitionParams,
) -> Result<Option<lsp_types::GotoDefinitionResponse>> {
    match state
        .uri_and_session_from_workspace(&params.text_document_position_params.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let sync = state.sync_workspace.get().unwrap();
            let position = params.text_document_position_params.position;
            Ok(session.token_definition_response(&uri, position, sync))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_completion(
    state: &ServerState,
    params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionResponse>> {
    let trigger_char = params
        .context
        .as_ref()
        .and_then(|ctx| ctx.trigger_character.as_deref())
        .unwrap_or("");
    let position = params.text_document_position.position;
    match state
        .uri_and_session_from_workspace(&params.text_document_position.text_document.uri)
        .await
    {
        Ok((uri, session)) => Ok(session
            .completion_items(&uri, position, trigger_char)
            .map(CompletionResponse::Array)),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_hover(
    state: &ServerState,
    params: lsp_types::HoverParams,
) -> Result<Option<lsp_types::Hover>> {
    match state
        .uri_and_session_from_workspace(&params.text_document_position_params.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            let sync = state.sync_workspace.get().unwrap();
            Ok(capabilities::hover::hover_data(
                session,
                &state.keyword_docs,
                &uri,
                position,
                state.config.read().client.clone(),
                sync,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_prepare_rename(
    state: &ServerState,
    params: lsp_types::TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>> {
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let sync = state.sync_workspace.get().unwrap();
            match capabilities::rename::prepare_rename(session, &uri, params.position, sync) {
                Ok(res) => Ok(Some(res)),
                Err(err) => {
                    tracing::error!("{}", err.to_string());
                    Ok(None)
                }
            }
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub async fn handle_rename(
    state: &ServerState,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>> {
    match state
        .uri_and_session_from_workspace(&params.text_document_position.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let new_name = params.new_name;
            let position = params.text_document_position.position;
            let sync = state.sync_workspace.get().unwrap();
            match capabilities::rename::rename(session, new_name, &uri, position, sync) {
                Ok(res) => Ok(Some(res)),
                Err(err) => {
                    tracing::error!("{}", err.to_string());
                    Ok(None)
                }
            }
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
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
        .await
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            Ok(capabilities::highlight::get_highlights(
                session, &uri, position,
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
        .uri_and_session_from_workspace(&params.text_document_position.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let position = params.text_document_position.position;
            let sync = state.sync_workspace.get().unwrap();
            Ok(session.token_references(&uri, position, sync))
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
        .await
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
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((temp_uri, session)) => Ok(capabilities::code_actions(
            session,
            &params.range,
            &params.text_document.uri,
            &temp_uri,
            &params.context.diagnostics,
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
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((url, session)) => Ok(Some(capabilities::code_lens::code_lens(&session, &url))),
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
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, session)) => Ok(capabilities::semantic_tokens::semantic_tokens_range(
            session,
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
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, session)) => Ok(capabilities::semantic_tokens::semantic_tokens_full(
            session, &uri,
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
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, session)) => {
            let config = &state.config.read().inlay_hints;
            Ok(capabilities::inlay_hints::inlay_hints(
                session,
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
pub async fn handle_show_ast(
    state: &ServerState,
    params: lsp_ext::ShowAstParams,
) -> Result<Option<TextDocumentIdentifier>> {
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((_, session)) => {
            let current_open_file = params.text_document.uri;
            // Convert the Uri to a PathBuf
            let path = current_open_file.to_file_path().ok();

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
            let path_is_submodule = |ident: &Ident, path: &Option<PathBuf>| -> bool {
                ident
                    .span()
                    .source_id()
                    .map(|p| session.engines.read().se().get_path(p))
                    == *path
            };

            let ast_path = PathBuf::from(params.save_path.path());
            {
                let program = session.compiled_program.read();
                match params.ast_kind.as_str() {
                    "lexed" => {
                        Ok(program.lexed.as_ref().and_then(|lexed_program| {
                            let mut formatted_ast = format!("{:#?}", program.lexed);
                            for (ident, submodule) in &lexed_program.root.submodules {
                                if path_is_submodule(ident, &path) {
                                    // overwrite the root AST with the submodule AST
                                    formatted_ast = format!("{:#?}", submodule.module.tree);
                                }
                            }
                            write_ast_to_file(ast_path.join("lexed.rs").as_path(), &formatted_ast)
                        }))
                    }
                    "parsed" => {
                        Ok(program.parsed.as_ref().and_then(|parsed_program| {
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
                        }))
                    }
                    "typed" => {
                        Ok(program.typed.as_ref().and_then(|typed_program| {
                            // Initialize the string with the AST from the root
                            let mut formatted_ast = debug::print_decl_engine_types(
                                &typed_program.root_module.all_nodes,
                                session.engines.read().de(),
                            );
                            for (ident, submodule) in &typed_program.root_module.submodules {
                                if path_is_submodule(ident, &path) {
                                    // overwrite the root AST with the submodule AST
                                    formatted_ast = debug::print_decl_engine_types(
                                        &submodule.module.all_nodes,
                                        session.engines.read().de(),
                                    );
                                }
                            }
                            write_ast_to_file(ast_path.join("typed.rs").as_path(), &formatted_ast)
                        }))
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
pub async fn handle_on_enter(
    state: &ServerState,
    params: lsp_ext::OnEnterParams,
) -> Result<Option<WorkspaceEdit>> {
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((uri, _)) => {
            // handle on_enter capabilities if they are enabled
            Ok(capabilities::on_enter(
                &state.config.read().on_enter,
                &state.documents,
                &uri,
                &params,
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
    params: lsp_ext::VisualizeParams,
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
pub(crate) async fn metrics(
    state: &ServerState,
    params: lsp_ext::MetricsParams,
) -> Result<Option<Vec<(String, PerformanceData)>>> {
    match state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await
    {
        Ok((_, session)) => {
            let mut metrics = vec![];
            for kv in session.metrics.iter() {
                let path = session
                    .engines
                    .read()
                    .se()
                    .get_manifest_path_from_program_id(kv.key())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                metrics.push((path, kv.value().clone()));
            }
            Ok(Some(metrics))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}
