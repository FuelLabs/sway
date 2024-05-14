#![allow(dead_code)]
use crate::{
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::document,
};
use dashmap::DashMap;
use forc_util::fs_locking::PidFileLocking;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use ropey::Rope;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct TextDocument {
    #[allow(dead_code)]
    language_id: String,
    #[allow(dead_code)]
    version: i32,
    uri: String,
    content: Rope,
}

impl TextDocument {
    pub async fn build_from_path(path: &str) -> Result<Self, DocumentError> {
        tokio::fs::read_to_string(path)
            .await
            .map(|content| Self {
                language_id: "sway".into(),
                version: 1,
                uri: path.into(),
                content: Rope::from_str(&content),
            })
            .map_err(|_| DocumentError::DocumentNotFound { path: path.into() })
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn get_line(&self, line: usize) -> String {
        self.content.line(line).to_string()
    }

    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        let edit = self.build_edit(change);

        self.content.remove(edit.start_index..edit.end_index);
        self.content.insert(edit.start_index, edit.change_text);
    }

    pub fn get_text(&self) -> String {
        self.content.to_string()
    }
}

// private methods
impl TextDocument {
    fn build_edit<'change>(
        &self,
        change: &'change TextDocumentContentChangeEvent,
    ) -> EditText<'change> {
        let change_text = change.text.as_str();
        let text_bytes = change_text.as_bytes();
        let text_end_byte_index = text_bytes.len();

        let range = if let Some(range) = change.range {
            range
        } else {
            let start = self.byte_to_position(0);
            let end = self.byte_to_position(text_end_byte_index);
            Range { start, end }
        };

        let start_index = self.position_to_index(range.start);
        let end_index = self.position_to_index(range.end);

        EditText {
            start_index,
            end_index,
            change_text,
        }
    }

    fn byte_to_position(&self, byte_index: usize) -> Position {
        let line_index = self.content.byte_to_line(byte_index);

        let line_utf16_cu_index = {
            let char_index = self.content.line_to_char(line_index);
            self.content.char_to_utf16_cu(char_index)
        };

        let character_utf16_cu_index = {
            let char_index = self.content.byte_to_char(byte_index);
            self.content.char_to_utf16_cu(char_index)
        };

        let character = character_utf16_cu_index - line_utf16_cu_index;

        Position::new(line_index as u32, character as u32)
    }

    fn position_to_index(&self, position: Position) -> usize {
        let row_index = position.line as usize;
        let column_index = position.character as usize;

        let row_char_index = self.content.line_to_char(row_index);
        let column_char_index = self.content.utf16_cu_to_char(column_index);

        row_char_index + column_char_index
    }
}

/// Marks the specified file as "dirty" by creating a corresponding flag file.
///
/// This function ensures the necessary directory structure exists before creating the flag file.
pub fn mark_file_as_dirty(uri: &Url) -> Result<(), LanguageServerError> {
    let path = document::get_path_from_url(uri)?;
    Ok(PidFileLocking::lsp(path)
        .lock()
        .map_err(|e| DirectoryError::LspLocksDirFailed(e.to_string()))?)
}

/// Removes the corresponding flag file for the specified Url.
///
/// If the flag file does not exist, this function will do nothing.
pub fn remove_dirty_flag(uri: &Url) -> Result<(), LanguageServerError> {
    let path = document::get_path_from_url(uri)?;
    let uri = uri.clone();
    Ok(PidFileLocking::lsp(path)
        .release()
        .map_err(|err| DocumentError::UnableToRemoveFile {
            path: uri.path().to_string(),
            err: err.to_string(),
        })?)
}

#[derive(Debug)]
struct EditText<'text> {
    start_index: usize,
    end_index: usize,
    change_text: &'text str,
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
        let src = self.update_text_document(uri, changes).ok_or_else(|| {
            DocumentError::DocumentNotFound {
                path: uri.path().to_string(),
            }
        })?;

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

    /// Get the document at the given [Url].
    pub fn get_text_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.try_get(url.path())
            .try_unwrap()
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|document| document.clone())
    }

    /// Update the document at the given [Url] with the Vec of changes returned by the client.
    pub fn update_text_document(
        &self,
        url: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Option<String> {
        self.try_get_mut(url.path())
            .try_unwrap()
            .map(|mut document| {
                for change in changes {
                    document.apply_change(change);
                }
                document.get_text()
            })
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
}

impl std::ops::Deref for Documents {
    type Target = DashMap<String, TextDocument>;
    fn deref(&self) -> &Self::Target {
        &self.0
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
}
