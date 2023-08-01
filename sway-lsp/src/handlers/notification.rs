//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{
    error::LanguageServerError, event_loop::server_state_ext::ServerStateExt,
    server_state::ServerState,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType,
};

//--------------- Async versions --------------//

pub async fn handle_did_open_text_document_async(
    state: &ServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.handle_open_file(&uri);
    // If the token map is empty, then we need to parse the project.
    // Otherwise, don't recompile the project when a new file in the project is opened
    // as the workspace is already compiled.
    if session.token_map().is_empty() {
        state
            .parse_project(uri, params.text_document.uri, session.clone())
            .await;
    }
    Ok(())
}

pub async fn handle_did_change_text_document_async(
    state: &ServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.write_changes_to_file(&uri, params.content_changes)?;
    state
        .parse_project_async(uri, params.text_document.uri, session.clone())
        .await;
    Ok(())
}

pub(crate) async fn handle_did_save_text_document_async(
    state: &ServerState,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.sync.resync()?;
    state
        .parse_project_async(uri, params.text_document.uri, session.clone())
        .await;
    Ok(())
}

pub(crate) fn handle_did_change_watched_files(
    state: &ServerState,
    params: DidChangeWatchedFilesParams,
) {
    for event in params.changes {
        if event.typ == FileChangeType::DELETED {
            match state.sessions.uri_and_session_from_workspace(&event.uri) {
                Ok((uri, session)) => {
                    let _ = session.remove_document(&uri);
                }
                Err(err) => tracing::error!("{}", err.to_string()),
            }
        }
    }
}

//--------------- Sync versions --------------//

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_did_open_text_document(
    ext: &ServerStateExt,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.handle_open_file(&uri);
    ext.state
        .parse_project(uri, params.text_document.uri, session.clone());
    ext.publish_diagnostics(
        params.text_document.uri,
        ext.state.diagnostics(&uri, session),
    );
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_did_change_text_document(
    ext: &ServerStateExt,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.write_changes_to_file(&uri, params.content_changes)?;
    ext.state
        .parse_project(uri, params.text_document.uri, session.clone());
    ext.publish_diagnostics(
        params.text_document.uri,
        ext.state.diagnostics(&uri, session),
    );
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_did_save_text_document(
    ext: &ServerStateExt,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.sync.resync()?;
    ext.state
        .parse_project(uri, params.text_document.uri, session.clone());
    ext.publish_diagnostics(
        params.text_document.uri,
        ext.state.diagnostics(&uri, session),
    );
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_cancel(state: &mut ServerStateExt, params: lsp_types::CancelParams) {
    let id: lsp_server::RequestId = match params.id {
        lsp_types::NumberOrString::Number(id) => id.into(),
        lsp_types::NumberOrString::String(id) => id.into(),
    };
    state.cancel(id);
}
