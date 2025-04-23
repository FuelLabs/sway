//! This module is responsible for implementing handlers for Language Server
//! Protocol. This module specifically handles notification messages sent by the Client.

use crate::{
    core::{document::Documents, session::Session},
    error::{DocumentError, LanguageServerError},
    server_state::{CompilationContext, ServerState, TaskMessage},
};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, FileChangeType, Url,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{atomic::Ordering, Arc},
    time,
};

pub async fn handle_did_open_text_document(
    state: &ServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), LanguageServerError> {
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
                optimized_build: false,
                gc_options: state.config.read().garbage_collection.clone(),
                file_versions: BTreeMap::new(),
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
            optimized_build,
            gc_options: state.config.read().garbage_collection.clone(),
            file_versions,
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
    session.sync.resync()?;
    let file_versions = file_versions(&state.documents, &uri, None);
    send_new_compilation_request(state, session.clone(), &uri, false, file_versions);
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
    eprintln!(
        "Received did change watched files notification: {:?}",
        params
    );
    // eprintln!(
    //     "Received did change watched files notification: {:#?}",
    //     params
    // );
    for event in params.changes {
        let (uri, session) = state.uri_and_session_from_workspace(&event.uri).await?;
        if let FileChangeType::DELETED = event.typ {
            state.pid_locked_files.remove_dirty_flag(&event.uri)?;
            let _ = state.documents.remove_document(&uri);
        }

        // Check for Forc.toml changes
        if event.uri.path().ends_with("Forc.toml") && event.typ == FileChangeType::CHANGED {
            // find the time of the last modified Forc.toml
            let forc_toml_path = PathBuf::from(event.uri.path());
            let forc_toml_time = forc_toml_path.metadata().unwrap().modified().unwrap();

            // get the current time
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            eprintln!("Forc.toml time: {:?}", forc_toml_time);
            eprintln!("Current time: {:?}", current_time);

            //session.sync.resync()?;
            eprintln!("Forc.toml changed!!");
            let manifest_dir = session.sync.manifest_dir()?;

            eprintln!("Garbage collecting program...");
            let _ = session.garbage_collect_program(&mut session.engines.write());
            eprintln!("manifest_dir: {:?}", manifest_dir);
            // if let Ok(session) = state
            //     .sessions
            //     .reinitialize(&event.uri, &manifest_dir, &state.documents)
            //     .await
            // {

                eprintln!("uri: {:?}", uri);
                // let file_versions = file_versions(&state.documents, &uri, None);
                let file_versions = BTreeMap::new();
                eprintln!("File versions: {:#?}", file_versions);
                eprintln!("Sending new compilation request...");
                send_new_compilation_request(state, session.clone(), &uri, false, file_versions);
                eprintln!("Waiting for parsing...");
                state.wait_for_parsing().await;
                eprintln!("Parsing finished!!");

                eprintln!("Publishing diagnostics...");
                state.publish_diagnostics(uri, event.uri, session).await;
            // }
        }
    }
    Ok(())
}

/// Checks if any files in the given directory have been modified since the specified time.
///
/// # Arguments
/// * `dir_path` - The directory path to check for modifications
/// * `start_time` - The reference time to compare file modification times against
///
/// Returns true if any file was modified after start_time, false otherwise
fn check_modifications_since<P: AsRef<Path>>(
    dir_path: P,
    start_time: time::SystemTime,
) -> Result<bool, LanguageServerError> {
    for entry in walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let metadata = fs::metadata(entry.path()).map_err(|e| {
                LanguageServerError::DocumentError(DocumentError::IOError {
                    path: entry.path().to_string_lossy().to_string(),
                    error: e.to_string(),
                })
            })?;

            if let Ok(modified_time) = metadata.modified() {
                if modified_time > start_time {
                    // File was modified after start_time
                    tracing::debug!("Modified file found: {}", entry.path().display());
                    return Ok(true);
                }
            }
        }
    }
    // No modified files found
    Ok(false)
}

#[test]
fn test_check_modifications_since() {
    let dir_path = "/Users/josh/Documents/rust/fuel/test_projects/lsp-issue/test-contract";
    let one_hour_ago = time::SystemTime::now()
        .checked_sub(time::Duration::from_secs(3600))
        .expect("Time calculation error");
    let now = time::SystemTime::now();
    let modified = check_modifications_since(dir_path, one_hour_ago).unwrap();
    eprintln!("time: {:?}", now.elapsed());
    if modified {
        println!("At least one file was modified after the start time.");
    } else {
        println!("No files were modified after the start time.");
    }
}
