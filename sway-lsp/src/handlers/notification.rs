//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{
    core::{document::Documents, session::Session},
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
    tracing::info!("textDocument/didOpen: {:?}", file_uri);

    // Get or initialize the global SyncWorkspace.
    // get_or_try_init ensures the initialization closure runs only once.
    let sync_workspace_arc = match state.sync_workspace.get() {
        Some(sw_arc) => {
            tracing::debug!("SyncWorkspace already initialized.");
            sw_arc.clone()
        }
        None => {
            // Not initialized, attempt to initialize.
            // This closure will only run if the OnceLock is empty.
            // It needs to be an async block if initialize_global_sync_workspace is async.
            // However, get_or_try_init expects a FnOnce that returns Result, not async.
            // So, we must block_on or handle this differently if init is async.
            // For now, let's assume we can call an async helper and set it.
            // A robust way for async init with OnceLock might need a small mutex or a dedicated future.

            // Simpler approach: Check, then init if needed, then get. This might race if two didOpen arrive simultaneously.
            // Using a lock specifically for this initialization is safer if get_or_try_init can't take an async fn.
            // For now, let's assume a single-threaded context for this specific part or that races are unlikely
            // for the very first file open. A dedicated init future in ServerState would be more robust.

            // Let's use get_or_try_init by making the init fn sync or by adapting.
            // Alternative: a dedicated initialization state/mutex in ServerState.

            // For simplicity, let's call the async init and then try to set.
            // This is NOT fully robust against races for the *very first* init.
            // `OnceLock::get_or_init_async` would be ideal but is not in std yet.
            // We will call our async helper and then try to set.
            // If another thread sets it in between, .set() will fail, which is acceptable.
            match state.initialize_workspace_sync(file_uri).await {
                Ok(initialized_sw) => {
                    match state.sync_workspace.set(initialized_sw.clone()) {
                        Ok(()) => {
                            tracing::info!("Global SyncWorkspace successfully initialized and set.")
                        }
                        Err(_) => tracing::info!(
                            "Global SyncWorkspace was already set by another concurrent operation."
                        ),
                    }
                    // Regardless of who set it, get the reference now.
                    state.sync_workspace.get().unwrap().clone() // Should be Some now.
                }
                Err(e) => {
                    tracing::error!("Failed to initialize global SyncWorkspace: {:?}. LSP functions requiring it may fail.", e);
                    // Cannot proceed if SyncWorkspace failed to init.
                    return Err(e);
                }
            }
        }
    };

    // Convert the opened file's actual URI to its temporary URI
    let temp_uri_for_opened_file = sync_workspace_arc.workspace_to_temp_url(file_uri)?;

    // Ensure this specific document is loaded into the main document store using its temp URI
    state
        .documents
        .handle_open_file(&temp_uri_for_opened_file)
        .await;
    tracing::debug!("Handled open for temp file: {:?}", temp_uri_for_opened_file);

    // Get or create a session for the original file URI.
    let (uri, session) = state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    state.documents.handle_open_file(&uri).await;
    // If the token map is empty, then we need to parse the project.
    // Otherwise, don't recompile the project when a new file in the project is opened
    // as the workspace is already compiled.
    if session.token_map().is_empty() {
        let _ = state
            .cb_tx
            .send(TaskMessage::CompilationContext(CompilationContext {
                session: Some(session.clone()),
                uri: Some(uri.clone()),
                version: None,
                optimized_build: false,
                gc_options: state.config.read().garbage_collection.clone(),
                file_versions: BTreeMap::new(),
                sync: Some(state.sync_workspace.get().unwrap().clone()),
            }));
        state.is_compiling.store(true, Ordering::SeqCst);
        state.wait_for_parsing().await;
        state
            .publish_diagnostics(uri, params.text_document.uri, session)
            .await;
    }
    Ok(())
}

fn send_new_compilation_request(
    state: &ServerState,
    session: Arc<Session>,
    uri: &Url,
    version: Option<i32>,
    optimized_build: bool,
    file_versions: BTreeMap<PathBuf, Option<u64>>,
) {
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
            uri: Some(uri.clone()),
            version,
            optimized_build,
            gc_options: state.config.read().garbage_collection.clone(),
            file_versions,
            sync: Some(state.sync_workspace.get().unwrap().clone()),
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

    let (uri, session) = state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    state
        .documents
        .write_changes_to_file(&uri, &params.content_changes)
        .await?;

    let file_versions = file_versions(
        &state.documents,
        &uri,
        Some(params.text_document.version as u64),
    );
    send_new_compilation_request(
        state,
        session.clone(),
        &uri,
        Some(params.text_document.version),
        // TODO: Set this back to true once https://github.com/FuelLabs/sway/issues/6576 is fixed.
        false,
        file_versions,
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
    let (uri, session) = state
        .uri_and_session_from_workspace(&params.text_document.uri)
        .await?;
    let file_versions = file_versions(&state.documents, &uri, None);
    send_new_compilation_request(state, session.clone(), &uri, None, false, file_versions);
    state.wait_for_parsing().await;
    state
        .publish_diagnostics(uri, params.text_document.uri, session)
        .await;
    Ok(())
}

pub(crate) async fn handle_did_change_watched_files(
    state: &ServerState,
    params: DidChangeWatchedFilesParams,
) -> Result<(), LanguageServerError> {
    for event in params.changes {
        let (uri, session) = state.uri_and_session_from_workspace(&event.uri).await?;

        match event.typ {
            FileChangeType::CHANGED => {
                if event.uri.to_string().contains("Forc.toml") {
                    //session.sync.sync_manifest();
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
    Ok(())
}
