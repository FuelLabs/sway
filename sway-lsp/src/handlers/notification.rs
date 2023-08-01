//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{core::sync, server_state::ServerState};
use forc_pkg::PackageManifestFile;
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType,
};

pub(crate) async fn handle_did_open_text_document(
    state: &ServerState,
    params: DidOpenTextDocumentParams,
) {
    match state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
    {
        Ok((uri, session)) => {
            session.handle_open_file(&uri);
            state
                .parse_project(uri, params.text_document.uri, session.clone())
                .await;
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
}

pub(crate) async fn handle_did_change_text_document(
    state: &ServerState,
    params: DidChangeTextDocumentParams,
) {
    match state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
    {
        Ok((uri, session)) => {
            // update this file with the new changes and write to disk
            match session.write_changes_to_file(&uri, params.content_changes) {
                Ok(_) => {
                    state
                        .parse_project(uri, params.text_document.uri.clone(), session)
                        .await;
                }
                Err(err) => tracing::error!("{}", err.to_string()),
            }
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
}

pub(crate) async fn handle_did_save_text_document(
    state: &ServerState,
    params: DidSaveTextDocumentParams,
) {
    match state
        .sessions
        .uri_and_session_from_workspace(&params.text_document.uri)
    {
        Ok((uri, session)) => {
            if let Err(err) = session.sync.resync() {
                tracing::error!("{}", err.to_string().as_str());
            }
            state
                .parse_project(uri, params.text_document.uri, session)
                .await;
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
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
