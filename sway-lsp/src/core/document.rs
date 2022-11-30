#![allow(dead_code)]
use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use crate::error::DocumentError;

#[derive(Debug)]
pub(crate) struct TextDocument {
    #[allow(dead_code)]
    language_id: String,
    #[allow(dead_code)]
    version: i32,
    uri: String,
    content: Rope,
}

impl TextDocument {
    pub(crate) fn build_from_path(path: &str) -> Result<Self, DocumentError> {
        std::fs::read_to_string(path)
            .map(|content| Self {
                language_id: "sway".into(),
                version: 1,
                uri: path.into(),
                content: Rope::from_str(&content),
            })
            .map_err(|_| DocumentError::DocumentNotFound { path: path.into() })
    }

    pub(crate) fn get_uri(&self) -> &str {
        &self.uri
    }

    pub(crate) fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        let edit = self.build_edit(change);

        self.content.remove(edit.start_index..edit.end_index);
        self.content.insert(edit.start_index, edit.change_text);
    }

    pub(crate) fn get_text(&self) -> String {
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

        let range = match change.range {
            Some(range) => range,
            None => {
                let start = self.byte_to_position(0);
                let end = self.byte_to_position(text_end_byte_index);
                Range { start, end }
            }
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

#[derive(Debug)]
struct EditText<'text> {
    start_index: usize,
    end_index: usize,
    change_text: &'text str,
}

#[cfg(test)]
mod tests {
    use crate::utils::test::get_absolute_path;

    use super::*;

    #[test]
    fn build_from_path_returns_text_document() {
        let path = get_absolute_path("sway-lsp/test/fixtures/cats.txt");
        let result = TextDocument::build_from_path(&path);
        assert!(result.is_ok(), "result = {:?}", result);
    }

    #[test]
    fn build_from_path_returns_document_not_found_error() {
        let path = get_absolute_path("not/a/real/file/path");
        let result = TextDocument::build_from_path(&path).expect_err("expected DocumentNotFound");
        assert_eq!(result, DocumentError::DocumentNotFound { path });
    }
}
