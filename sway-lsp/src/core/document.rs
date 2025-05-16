use std::{path::PathBuf, sync::Arc};

use crate::{
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::document,
};
use dashmap::DashMap;
use forc_util::fs_locking::PidFileLocking;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use sway_utils::get_sway_files;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct TextDocument {
    version: i32,
    uri: String,
    content: String,
    line_offsets: Vec<usize>,
}

impl TextDocument {
    pub async fn build_from_path(path: &str) -> Result<Self, DocumentError> {
        tokio::fs::read_to_string(path)
            .await
            .map(|content| {
                let line_offsets = TextDocument::calculate_line_offsets(&content);
                Self {
                    version: 1,
                    uri: path.into(),
                    content,
                    line_offsets,
                }
            })
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    DocumentError::DocumentNotFound { path: path.into() }
                }
                std::io::ErrorKind::PermissionDenied => {
                    DocumentError::PermissionDenied { path: path.into() }
                }
                _ => DocumentError::IOError {
                    path: path.into(),
                    error: e.to_string(),
                },
            })
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn get_text(&self) -> &str {
        &self.content
    }

    pub fn get_line(&self, line: usize) -> &str {
        let start = self
            .line_offsets
            .get(line)
            .copied()
            .unwrap_or(self.content.len());
        let end = self
            .line_offsets
            .get(line + 1)
            .copied()
            .unwrap_or(self.content.len());
        &self.content[start..end]
    }

    pub fn apply_change(
        &mut self,
        change: &TextDocumentContentChangeEvent,
    ) -> Result<(), DocumentError> {
        if let Some(range) = change.range {
            self.validate_range(range)?;
            let start_index = self.position_to_index(range.start);
            let end_index = self.position_to_index(range.end);
            self.content
                .replace_range(start_index..end_index, &change.text);
        } else {
            self.content.clone_from(&change.text);
        }
        self.line_offsets = Self::calculate_line_offsets(&self.content);
        self.version += 1;
        Ok(())
    }

    fn validate_range(&self, range: Range) -> Result<(), DocumentError> {
        let start = self.position_to_index(range.start);
        let end = self.position_to_index(range.end);
        if start > end || end > self.content.len() {
            return Err(DocumentError::InvalidRange { range });
        }
        Ok(())
    }

    fn position_to_index(&self, position: Position) -> usize {
        let line_offset = self
            .line_offsets
            .get(position.line as usize)
            .copied()
            .unwrap_or(self.content.len());
        line_offset + position.character as usize
    }

    fn calculate_line_offsets(text: &str) -> Vec<usize> {
        let mut offsets = vec![0];
        for (i, c) in text.char_indices() {
            if c == '\n' {
                offsets.push(i + 1);
            }
        }
        offsets
    }
}

pub struct Documents(DashMap<String, TextDocument>);

impl Default for Documents {
    fn default() -> Self {
        Self::new()
    }
}

impl Documents {
    pub fn new() -> Self {
        Documents(DashMap::new())
    }

    pub async fn handle_open_file(&self, uri: &Url) {
        if !self.contains_key(uri.path()) {
            if let Ok(text_document) = TextDocument::build_from_path(uri.path()).await {
                let _ = self.store_document(text_document);
            }
        }
    }

    /// Asynchronously writes the changes to the file and updates the document.
    pub async fn write_changes_to_file(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<(), LanguageServerError> {
        let src = self.update_text_document(uri, changes)?;

        let mut file =
            File::create(uri.path())
                .await
                .map_err(|err| DocumentError::UnableToCreateFile {
                    path: uri.path().to_string(),
                    err: err.to_string(),
                })?;

        file.write_all(src.as_bytes())
            .await
            .map_err(|err| DocumentError::UnableToWriteFile {
                path: uri.path().to_string(),
                err: err.to_string(),
            })?;

        Ok(())
    }

    /// Update the document at the given [Url] with the Vec of changes returned by the client.
    pub fn update_text_document(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<String, DocumentError> {
        self.try_get_mut(uri.path())
            .try_unwrap()
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: uri.path().to_string(),
            })
            .and_then(|mut document| {
                for change in changes {
                    document.apply_change(change)?;
                }
                Ok(document.get_text().to_string())
            })
    }

    /// Get the document at the given [Url].
    pub fn get_text_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.try_get(url.path())
            .try_unwrap()
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|document| document.clone())
    }

    /// Remove the text document.
    pub fn remove_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.remove(url.path())
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|(_, text_document)| text_document)
    }

    /// Store the text document.
    pub fn store_document(&self, text_document: TextDocument) -> Result<(), DocumentError> {
        let uri = text_document.get_uri().to_string();
        self.insert(uri.clone(), text_document).map_or(Ok(()), |_| {
            Err(DocumentError::DocumentAlreadyStored { path: uri })
        })
    }

    /// Populate with sway files found in the workspace.
    pub async fn store_sway_files_from_temp(
        &self,
        temp_dir: PathBuf,
    ) -> Result<(), LanguageServerError> {
        for path_str in get_sway_files(temp_dir).iter().filter_map(|fp| fp.to_str()) {
            let text_doc = TextDocument::build_from_path(path_str).await?;
            self.store_document(text_doc)?;
        }
        Ok(())
    }
}

impl std::ops::Deref for Documents {
    type Target = DashMap<String, TextDocument>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Manages process-based file locking for multiple files.
pub struct PidLockedFiles {
    locks: DashMap<Url, Arc<PidFileLocking>>,
}

impl Default for PidLockedFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl PidLockedFiles {
    pub fn new() -> Self {
        Self {
            locks: DashMap::new(),
        }
    }

    /// Marks the specified file as "dirty" by creating a corresponding flag file.
    ///
    /// This function ensures the necessary directory structure exists before creating the flag file.
    /// If the file is already locked, this function will do nothing. This is to reduce the number of
    /// unnecessary file IO operations.
    pub fn mark_file_as_dirty(&self, uri: &Url) -> Result<(), LanguageServerError> {
        if !self.locks.contains_key(uri) {
            let path = document::get_path_from_url(uri)?;
            let file_lock = Arc::new(PidFileLocking::lsp(path));
            file_lock
                .lock()
                .map_err(|e| DirectoryError::LspLocksDirFailed(e.to_string()))?;
            self.locks.insert(uri.clone(), file_lock);
        }
        Ok(())
    }

    /// Removes the corresponding flag file for the specified Url.
    ///
    /// If the flag file does not exist, this function will do nothing.
    pub fn remove_dirty_flag(&self, uri: &Url) -> Result<(), LanguageServerError> {
        if let Some((uri, file_lock)) = self.locks.remove(uri) {
            file_lock
                .release()
                .map_err(|err| DocumentError::UnableToRemoveFile {
                    path: uri.path().to_string(),
                    err: err.to_string(),
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_lsp_test_utils::get_absolute_path;

    #[tokio::test]
    async fn build_from_path_returns_text_document() {
        let path = get_absolute_path("sway-lsp/tests/fixtures/cats.txt");
        let result = TextDocument::build_from_path(&path).await;
        assert!(result.is_ok(), "result = {result:?}");
        let document = result.unwrap();
        assert_eq!(document.version, 1);
        assert_eq!(document.uri, path);
        assert!(!document.content.is_empty());
        assert!(!document.line_offsets.is_empty());
    }

    #[tokio::test]
    async fn build_from_path_returns_document_not_found_error() {
        let path = get_absolute_path("not/a/real/file/path");
        let result = TextDocument::build_from_path(&path)
            .await
            .expect_err("expected DocumentNotFound");
        assert_eq!(result, DocumentError::DocumentNotFound { path });
    }

    #[tokio::test]
    async fn store_document_returns_empty_tuple() {
        let documents = Documents::new();
        let path = get_absolute_path("sway-lsp/tests/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).await.unwrap();
        let result = documents.store_document(document);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn store_document_returns_document_already_stored_error() {
        let documents = Documents::new();
        let path = get_absolute_path("sway-lsp/tests/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).await.unwrap();
        documents
            .store_document(document)
            .expect("expected successfully stored");
        let document = TextDocument::build_from_path(&path).await.unwrap();
        let result = documents
            .store_document(document)
            .expect_err("expected DocumentAlreadyStored");
        assert_eq!(result, DocumentError::DocumentAlreadyStored { path });
    }

    #[test]
    fn get_line_returns_correct_line() {
        let content = "line1\nline2\nline3".to_string();
        let line_offsets = TextDocument::calculate_line_offsets(&content);
        let document = TextDocument {
            version: 1,
            uri: "test.sw".into(),
            content,
            line_offsets,
        };
        assert_eq!(document.get_line(0), "line1\n");
        assert_eq!(document.get_line(1), "line2\n");
        assert_eq!(document.get_line(2), "line3");
    }

    #[test]
    fn apply_change_updates_content_correctly() {
        let content = "Hello, world!".to_string();
        let line_offsets = TextDocument::calculate_line_offsets(&content);
        let mut document = TextDocument {
            version: 1,
            uri: "test.sw".into(),
            content,
            line_offsets,
        };
        let change = TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 7), Position::new(0, 12))),
            range_length: None,
            text: "Rust".into(),
        };
        document.apply_change(&change).unwrap();
        assert_eq!(document.get_text(), "Hello, Rust!");
    }

    #[test]
    fn position_to_index_works_correctly() {
        let content = "line1\nline2\nline3".to_string();
        let line_offsets = TextDocument::calculate_line_offsets(&content);
        let document = TextDocument {
            version: 1,
            uri: "test.sw".into(),
            content,
            line_offsets,
        };
        assert_eq!(document.position_to_index(Position::new(1, 2)), 8);
    }
}
