//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notifications.

use crate::{
    capabilities,
    core::sync,
    global_state::GlobalState,
};
use forc_pkg::PackageManifestFile;
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, FileChangeType,
};

pub(crate) async fn handle_did_open_text_document(
    state: &GlobalState,
    params: DidOpenTextDocumentParams,
) {
    match state
        .sessions
        .get_uri_and_session(&params.text_document.uri)
    {
        Ok((uri, session)) => {
            session.handle_open_file(&uri);
            state.parse_project(uri, params.text_document.uri, session.clone()).await;
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
}

pub(crate) async fn handle_did_change_text_document(
    state: &GlobalState,
    params: DidChangeTextDocumentParams,
) {
    let config = state.config.read().on_enter.clone();
    match state.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => {
            // handle on_enter capabilities if they are enabled
            capabilities::on_enter(&config, &state.client, &session, &uri.clone(), &params)
                .await;

            // update this file with the new changes and write to disk
            match session.write_changes_to_file(&uri, params.content_changes) {
                Ok(_) => {
                    state.parse_project(uri, params.text_document.uri.clone(), session)
                        .await;
                }
                Err(err) => tracing::error!("{}", err.to_string()),
            }
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
}

pub(crate) async fn handle_did_save_text_document(
    state: &GlobalState,
    params: DidSaveTextDocumentParams,
) {
    match state.sessions.get_uri_and_session(&params.text_document.uri) {
        Ok((uri, session)) => {
            // overwrite the contents of the tmp/folder with everything in
            // the current workspace. (resync)
            if let Err(err) = session.sync.clone_manifest_dir_to_temp() {
                tracing::error!("{}", err.to_string().as_str());
            }

            let _ = session
                .sync
                .manifest_path()
                .and_then(|manifest_path| PackageManifestFile::from_dir(&manifest_path).ok())
                .map(|manifest| {
                    if let Some(temp_manifest_path) = &session.sync.temp_manifest_path() {
                        sync::edit_manifest_dependency_paths(&manifest, temp_manifest_path)
                    }
                });
            state.parse_project(uri, params.text_document.uri, session)
                .await;
        }
        Err(err) => tracing::error!("{}", err.to_string()),
    }
}


pub(crate) async fn handle_did_change_watched_files(
    state: &GlobalState,
    params: DidChangeWatchedFilesParams,
) {
    for event in params.changes {
        if event.typ == FileChangeType::DELETED {
            match state.sessions.get_uri_and_session(&event.uri) {
                Ok((uri, session)) => {
                    let _ = session.remove_document(&uri);
                }
                Err(err) => tracing::error!("{}", err.to_string()),
            }
        }
    }
}