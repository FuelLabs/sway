use crate::core::session::Session;
use std::sync::Arc;
use tower_lsp::lsp_types::{FileChangeType, FileEvent};

pub fn handle_watched_files(session: Arc<Session>, events: Vec<FileEvent>) {
    for event in events {
        if event.typ == FileChangeType::DELETED {
            let _ = session.remove_document(&event.uri);
        }
    }
}
