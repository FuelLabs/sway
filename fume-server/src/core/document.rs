use lspower::lsp::{Position, Range, TextDocumentContentChangeEvent, TextDocumentItem};
use ropey::Rope;

#[derive(Debug)]
pub struct TextDocument {
    language_id: String,
    version: i32,
    uri: String,
    text: Rope,
}

impl TextDocument {
    pub fn new(item: &TextDocumentItem) -> Self {
        Self {
            language_id: item.language_id.clone(),
            version: item.version,
            uri: item.uri.to_string(),
            text: Rope::from_str(&item.text),
        }
    }

    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        let edit = self.build_edit(change);

        self.text.remove(edit.start_index..edit.end_index);
        self.text.insert(edit.start_index, edit.change_text);
    }

    pub fn get_text_as_string(&self) -> String {
        self.text.to_string()
    }
}

// private methods
impl TextDocument {
    fn build_edit<'change>(&self, change: &'change TextDocumentContentChangeEvent) -> EditText<'change> {
        let change_text = change.text.as_str();
        let text_bytes = change_text.as_bytes();
        let text_end_byte_index = text_bytes.len();

        let range = match change.range {
            Some(range ) => range,
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
        let line_index = self.text.byte_to_line(byte_index);

        let line_utf16_cu_index = {
            let char_index = self.text.line_to_char(line_index);
            self.text.char_to_utf16_cu(char_index)
        };

        let character_utf16_cu_index = {
            let char_index = self.text.byte_to_char(byte_index);
            self.text.char_to_utf16_cu(char_index)
        };

        let character = character_utf16_cu_index - line_utf16_cu_index;

        Position::new(line_index as u32, character as u32)
    }

    fn position_to_index(&self, position: Position) -> usize {
        let row_index = position.line as usize;
        let column_index = position.character as usize;

        let row_char_index = self.text.line_to_char(row_index);
        let column_char_index = self.text.utf16_cu_to_char(column_index);
        
        row_char_index + column_char_index
    }
}

#[derive(Debug)]
struct EditText<'text> {
    start_index: usize,
    end_index: usize,
    change_text: &'text str,
}