//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles requests.

use crate::{
    capabilities,
    global_state::{GlobalState, GlobalStateSnapshot},
    lsp_ext,
    utils::debug,
};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use lsp_types::{
    CodeLens, CompletionResponse, DocumentFormattingParams, DocumentSymbolResponse,
    InitializeResult, InlayHint, InlayHintParams, PrepareRenameResponse, RenameParams,
    SemanticTokensParams, SemanticTokensResult, TextDocumentIdentifier, Url, WorkspaceEdit,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use sway_types::{Ident, Spanned};
use tower_lsp::jsonrpc::Result;
use tracing::metadata::LevelFilter;

pub(crate) fn handle_initialize(
    state: &GlobalState,
    params: lsp_types::InitializeParams,
) -> Result<InitializeResult> {
    if let Some(initialization_options) = &params.initialization_options {
        let mut config = state.config.write();
        *config = serde_json::from_value(initialization_options.clone())
            .ok()
            .unwrap_or_default();
    }
    // Initalizing tracing library based on the user's config
    let config = state.config.read();
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
        capabilities: crate::server_capabilities(),
        ..InitializeResult::default()
    })
}

pub(crate) fn handle_document_symbol(
    snap: GlobalStateSnapshot,
    params: lsp_types::DocumentSymbolParams,
) -> Result<Option<lsp_types::DocumentSymbolResponse>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => Ok(session
            .symbol_information(&uri)
            .map(DocumentSymbolResponse::Flat)),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_goto_definition(
    snap: GlobalStateSnapshot,
    params: lsp_types::GotoDefinitionParams,
) -> Result<Option<lsp_types::GotoDefinitionResponse>> {
    match snap
        .sessions
        .get_uri_and_session(&params.text_document_position_params.text_document.uri)
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            Ok(session.token_definition_response(uri, position))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_completion(
    snap: GlobalStateSnapshot,
    params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionResponse>> {
    let trigger_char = params
        .context
        .map(|ctx| ctx.trigger_character)
        .unwrap_or_default()
        .unwrap_or("".to_string());
    let position = params.text_document_position.position;
    match snap
        .sessions
        .get_uri_and_session(&params.text_document_position.text_document.uri)
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

pub(crate) fn handle_hover(
    snap: GlobalStateSnapshot,
    params: lsp_types::HoverParams,
) -> Result<Option<lsp_types::Hover>> {
    match snap
        .sessions
        .get_uri_and_session(&params.text_document_position_params.text_document.uri)
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            Ok(capabilities::hover::hover_data(
                session,
                &snap.keyword_docs,
                uri,
                position,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_prepare_rename(
    snap: GlobalStateSnapshot,
    params: lsp_types::TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => {
            match capabilities::rename::prepare_rename(session, uri, params.position) {
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

pub(crate) fn handle_rename(
    snap: GlobalStateSnapshot,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>> {
    match snap
        .sessions
        .get_uri_and_session(&params.text_document_position.text_document.uri)
    {
        Ok((uri, session)) => {
            let new_name = params.new_name;
            let position = params.text_document_position.position;
            match capabilities::rename::rename(session, new_name, uri, position) {
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

pub(crate) fn handle_document_highlight(
    snap: GlobalStateSnapshot,
    params: lsp_types::DocumentHighlightParams,
) -> Result<Option<Vec<lsp_types::DocumentHighlight>>> {
    match snap
        .sessions
        .get_uri_and_session(&params.text_document_position_params.text_document.uri)
    {
        Ok((uri, session)) => {
            let position = params.text_document_position_params.position;
            Ok(capabilities::highlight::get_highlights(
                session, uri, position,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_formatting(
    snap: GlobalStateSnapshot,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<lsp_types::TextEdit>>> {
    snap.sessions
        .get_uri_and_session(&params.text_document.uri)
        .and_then(|(uri, session)| session.format_text(&uri).map(Some))
        .or_else(|err| {
            tracing::error!("{}", err.to_string());
            Ok(None)
        })
}

pub(crate) fn handle_code_action(
    snap: GlobalStateSnapshot,
    params: lsp_types::CodeActionParams,
) -> Result<Option<lsp_types::CodeActionResponse>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((temp_uri, session)) => Ok(capabilities::code_actions(
            session,
            &params.range,
            params.text_document,
            &temp_uri,
        )),
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_code_lens(
    snap: GlobalStateSnapshot,
    params: lsp_types::CodeLensParams,
) -> Result<Option<Vec<CodeLens>>> {
    let mut result = vec![];
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((_, session)) => {
            // Construct code lenses for runnable functions
            session.runnables.iter().for_each(|item| {
                let runnable = item.value();
                result.push(CodeLens {
                    range: runnable.range(),
                    command: Some(runnable.command()),
                    data: None,
                });
            });
            Ok(Some(result))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_semantic_tokens_full(
    snap: GlobalStateSnapshot,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => {
            let _ = session.wait_for_parsing();
            Ok(capabilities::semantic_tokens::semantic_tokens_full(
                session, &uri,
            ))
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            Ok(None)
        }
    }
}

pub(crate) fn handle_inlay_hints(
    snap: GlobalStateSnapshot,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => {
            let _ = session.wait_for_parsing();
            let config = &snap.config.read().inlay_hints;
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
/// seperate side panel.
pub(crate) fn handle_show_ast(
    snap: GlobalStateSnapshot,
    params: lsp_ext::ShowAstParams,
) -> Result<Option<TextDocumentIdentifier>> {
    match snap.sessions.get_uri_and_session(&params.text_document.uri) {
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
                let engines = session.engines.read();
                ident.span().source_id().map(|p| engines.se().get_path(p)) == *path
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
                                &typed_program.root.all_nodes,
                                session.engines.read().de(),
                            );
                            for (ident, submodule) in &typed_program.root.submodules {
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
