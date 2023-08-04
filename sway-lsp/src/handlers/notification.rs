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
    tracing::info!("did_open async begin");

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
    tracing::info!("did_open async end");
    Ok(())
}

pub async fn handle_did_change_text_document_async(
    state: &ServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    tracing::info!("did_change async begin");

    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.write_changes_to_file(&uri, params.content_changes)?;
    state
        .parse_project(uri, params.text_document.uri, session.clone())
        .await;

    tracing::info!("did_change async end");
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
        .parse_project(uri, params.text_document.uri, session.clone())
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
    ext: &mut ServerStateExt,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    tracing::info!("did_open begin");

    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.handle_open_file(&uri);
    if session.parse_project(&uri)? {
        ext.publish_diagnostics(
            params.text_document.uri,
            ext.state.diagnostics(&uri, session),
        );
    }

    tracing::info!("did_open end");
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_did_change_text_document(
    ext: &mut ServerStateExt,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    tracing::info!("did_change begin");
    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.write_changes_to_file(&uri, params.content_changes)?;
    if session.parse_project(&uri)? {
        ext.publish_diagnostics(
            params.text_document.uri,
            ext.state.diagnostics(&uri, session),
        );
    }
    tracing::info!("did_change end");
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_did_save_text_document(
    ext: &mut ServerStateExt,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let (uri, session) = ext
        .state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)?;
    session.sync.resync()?;
    if session.parse_project(&uri)? {
        ext.publish_diagnostics(
            params.text_document.uri,
            ext.state.diagnostics(&uri, session),
        );
    }
    Ok(())
}

#[cfg(feature = "custom-event-loop")]
pub(crate) fn handle_cancel(state: &mut ServerStateExt, params: lsp_types::CancelParams) -> Result<(), LanguageServerError> {
    let id: lsp_server::RequestId = match params.id {
        lsp_types::NumberOrString::Number(id) => id.into(),
        lsp_types::NumberOrString::String(id) => id.into(),
    };
    state.cancel(id);
    Ok(())
}
