//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use std::sync::{atomic::Ordering, Arc};

use crate::{core::{document, session::Session}, error::LanguageServerError, server_state::ServerState};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType, Url,
};

pub async fn handle_did_open_text_document(
    state: &ServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    eprintln!("did_open_text_document");
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    session.handle_open_file(&uri).await;
    // If the token map is empty, then we need to parse the project.
    // Otherwise, don't recompile the project when a new file in the project is opened
    // as the workspace is already compiled.
    if session.token_map().is_empty() {
        send_new_compilation_request(&state, session.clone(), &uri, None);

        eprintln!("did open - waiting for parsing to finish");
        state.wait_for_parsing().await;
        state.publish_diagnostics(uri, params.text_document.uri, session).await;
    }
    Ok(())
}

fn send_new_compilation_request(
    state: &ServerState,
    session: Arc<Session>,
    uri: &Url,
    version: Option<i32>,
) {
    if state.is_compiling.load(Ordering::Relaxed) {
        eprintln!("retrigger compilation!");
        state.retrigger_compilation.store(true, Ordering::SeqCst);
    }
    eprintln!("new compilation request - setting is_compiling to true");
    state.is_compiling.store(true, Ordering::Relaxed);

    // If channel is full, remove the old value so the compilation thread only
    // gets the latest value.
    if state.mpsc_tx.is_full() {
        eprintln!("channel is full!");
        let _ = state.mpsc_rx.try_recv();
    }

    eprintln!("sending new compilation request");
    let _ = state.mpsc_tx.send(crate::server_state::Shared {
        session: Some(session.clone()),
        uri: Some(uri.clone()),
        version,
    });
}

pub async fn handle_did_change_text_document(
    state: &ServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    eprintln!("did change text document: version: {:?}", params.text_document.version);
    document::mark_file_as_dirty(&params.text_document.uri).await?;
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    session
        .write_changes_to_file(&uri, params.content_changes)
        .await?;
    send_new_compilation_request(&state, session.clone(), &uri, Some(params.text_document.version));
    Ok(())
}

pub(crate) async fn handle_did_save_text_document(
    state: &ServerState,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    eprintln!("did save text document");
    document::remove_dirty_flag(&params.text_document.uri).await?;
    let (uri, session) = state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    session.sync.resync()?;
    eprintln!("resynced");
    send_new_compilation_request(&state, session.clone(), &uri, None);
    state.wait_for_parsing().await;
    state.publish_diagnostics(uri, params.text_document.uri, session).await;
    Ok(())
}

pub(crate) async fn handle_did_change_watched_files(
    state: &ServerState,
    params: DidChangeWatchedFilesParams,
) -> Result<(), LanguageServerError> {
    for event in params.changes {
        let (uri, session) = state
            .sessions
            .uri_and_session_from_workspace(&event.uri)
            .await?;
        if let FileChangeType::DELETED = event.typ {
            document::remove_dirty_flag(&event.uri).await?;
            let _ = session.remove_document(&uri);
        }
    }
    Ok(())
}
