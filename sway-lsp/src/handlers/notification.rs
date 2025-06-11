//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{
    core::{document::Documents, session::Session, sync::SyncWorkspace},
    error::LanguageServerError,
    server_state::{CompilationContext, ServerState, TaskMessage},
};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType, Url,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

pub async fn handle_did_open_text_document(
    state: &ServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
    let file_uri = &params.text_document.uri;
    // Initialize the SyncWorkspace for this file if it doesn't exist.
    let sync_workspace = state.get_or_init_sync_workspace(file_uri).await?;

    // Get or create a session for the original file URI.
    let (uri, session) = state.uri_and_session_from_workspace(&params.text_document.uri)?;
    state.documents.handle_open_file(&uri).await;

    send_new_compilation_request(state, session.clone(), &uri, None, false, sync_workspace);
    state.is_compiling.store(true, Ordering::SeqCst);
    state.wait_for_parsing().await;
    state
        .publish_diagnostics(uri, params.text_document.uri, session)
        .await;

    Ok(())
}

fn send_new_compilation_request(
    state: &ServerState,
    session: Arc<Session>,
    uri: &Url,
    version: Option<i32>,
    optimized_build: bool,
    sync_workspace: Arc<SyncWorkspace>,
) {
    let file_versions = file_versions(&state.documents, uri, version.map(|v| v as u64));

    if state.is_compiling.load(Ordering::SeqCst) {
        // If we are already compiling, then we need to retrigger compilation
        state.retrigger_compilation.store(true, Ordering::SeqCst);
    }

    // Check if the channel is full. If it is, we want to ensure that the compilation
    // thread receives only the most recent value.
    if state.cb_tx.is_full() {
        while let Ok(TaskMessage::CompilationContext(_)) = state.cb_rx.try_recv() {
            // Loop will continue to remove `CompilationContext` messages
            // until the channel has no more of them.
        }
    }

    let _ = state
        .cb_tx
        .send(TaskMessage::CompilationContext(CompilationContext {
            session: Some(session.clone()),
            engines: state.engines.clone(),
            token_map: state.token_map.clone(),
            uri: Some(uri.clone()),
            version,
            optimized_build,
            gc_options: state.config.read().garbage_collection.clone(),
            file_versions,
            sync: Some(sync_workspace),
        }));
}

pub async fn handle_did_change_text_document(
    state: &ServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), LanguageServerError> {
    if let Err(err) = state
        .pid_locked_files
        .mark_file_as_dirty(&params.text_document.uri)
    {
        tracing::warn!("Failed to mark file as dirty: {}", err);
    }

    let (uri, session) = state.uri_and_session_from_workspace(&params.text_document.uri)?;
    let sync_workspace = state.get_sync_workspace_for_uri(&params.text_document.uri)?;
    state
        .documents
        .write_changes_to_file(&uri, &params.content_changes)
        .await?;

    send_new_compilation_request(
        state,
        session.clone(),
        &uri,
        Some(params.text_document.version),
        // TODO: Set this back to true once https://github.com/FuelLabs/sway/issues/6576 is fixed.
        false,
        sync_workspace,
    );
    Ok(())
}

fn file_versions(
    documents: &Documents,
    uri: &Url,
    version: Option<u64>,
) -> BTreeMap<PathBuf, Option<u64>> {
    let mut file_versions = BTreeMap::new();
    for item in documents.iter() {
        let path = PathBuf::from(item.key());
        if path == uri.to_file_path().unwrap() {
            file_versions.insert(path, version);
        } else {
            file_versions.insert(path, None);
        }
    }
    file_versions
}

pub(crate) async fn handle_did_save_text_document(
    state: &ServerState,
    params: DidSaveTextDocumentParams,
) -> Result<(), LanguageServerError> {
    state
        .pid_locked_files
        .remove_dirty_flag(&params.text_document.uri)?;
    let (uri, session) = state.uri_and_session_from_workspace(&params.text_document.uri)?;
    let sync_workspace = state.get_sync_workspace_for_uri(&params.text_document.uri)?;
    send_new_compilation_request(state, session.clone(), &uri, None, false, sync_workspace);
    state.wait_for_parsing().await;
    state
        .publish_diagnostics(uri, params.text_document.uri, session)
        .await;
    Ok(())
}

pub(crate) fn handle_did_change_watched_files(
    state: &ServerState,
    params: DidChangeWatchedFilesParams,
) -> Result<(), LanguageServerError> {
    for event in params.changes {
        match state.get_sync_workspace_for_uri(&event.uri) {
            Ok(sync_workspace) => {
                let uri = sync_workspace.workspace_to_temp_url(&event.uri)?;

                match event.typ {
                    FileChangeType::CHANGED => {
                        if event.uri.to_string().contains("Forc.toml") {
                            sync_workspace.sync_manifest()?;
                            // TODO: Recompile the project | see https://github.com/FuelLabs/sway/issues/7103
                        }
                    }
                    FileChangeType::DELETED => {
                        state.pid_locked_files.remove_dirty_flag(&event.uri)?;
                        let _ = state.documents.remove_document(&uri);
                    }
                    FileChangeType::CREATED => {
                        // TODO: handle this case
                    }
                    _ => {}
                }
            }
            Err(err) => {
                tracing::error!("Failed to get sync workspace for {}: {}", event.uri, err);
            }
        }
    }
    Ok(())
}
