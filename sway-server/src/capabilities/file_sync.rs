use crate::core::session::Session;
use lspower::lsp::{FileChangeType, FileEvent};
use std::sync::Arc;

pub fn handle_watched_files(session: Arc<Session>, events: Vec<FileEvent>) {
    for event in events {
        if let FileChangeType::Deleted {} = event.typ {
            let _ = session.remove_document(&event.uri);
        }
    }
}
