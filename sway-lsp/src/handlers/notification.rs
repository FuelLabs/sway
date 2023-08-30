//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{
    error::LanguageServerError,
    server_state::{self, ServerState},
};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType,
};

pub async fn handle_did_open_text_document(
    state: &mut ServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_mut_session_from_workspace(&params.text_document.uri)?;
    session.handle_open_file(&uri);
    // If the token map is empty, then we need to parse the project.
    // Otherwise, don't recompile the project when a new file in the project is opened
    // as the workspace is already compiled.
    if session.token_map().is_empty() {
        let parse_result = server_state::parse_project(uri.clone(), &session).await?;
        session.write_parse_result(parse_result);
        server_state::publish_diagnostics(
            &state.config,
            &state.client,
            uri,
            params.text_document.uri,
            session,
        )
        .await;
    }
    Ok(())
}

pub async fn handle_did_change_text_document(
    state: &mut ServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_mut_session_from_workspace(&params.text_document.uri)?;
    session.write_changes_to_file(&uri, params.content_changes)?;
    let parse_result = server_state::parse_project(uri.clone(), &session).await?;
    session.write_parse_result(parse_result);
    server_state::publish_diagnostics(
        &state.config,
        &state.client,
        uri,
        params.text_document.uri,
        session,
    )
    .await;
    Ok(())
}

pub(crate) async fn handle_did_save_text_document(
    state: &mut ServerState,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_mut_session_from_workspace(&params.text_document.uri)?;
    session.sync.resync()?;
    let parse_result = server_state::parse_project(uri.clone(), &session).await?;
    session.write_parse_result(parse_result);
    server_state::publish_diagnostics(
        &state.config,
        &state.client,
        uri,
        params.text_document.uri,
        session,
    )
    .await;
    Ok(())
}

pub(crate) fn handle_did_change_watched_files(
    state: &mut ServerState,
    params: DidChangeWatchedFilesParams,
) {
    for event in params.changes {
        if event.typ == FileChangeType::DELETED {
            match state
                .sessions
                .uri_and_mut_session_from_workspace(&event.uri)
            {
                Ok((uri, session)) => {
                    let _ = session.remove_document(&uri);
                }
                Err(err) => tracing::error!("{}", err.to_string()),
            }
        }
    }
}
