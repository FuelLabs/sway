use super::token::Token;
use super::token_type::TokenType;
use crate::{capabilities, core::token::traverse_node};
use ropey::Rope;
use std::collections::HashMap;
use std::sync::Arc;
use sway_core::{parse, TreeType};
use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent};

#[derive(Debug)]
pub struct TextDocument {
    #[allow(dead_code)]
    language_id: String,
    #[allow(dead_code)]
    version: i32,
    uri: String,
    content: Rope,
    tokens: Vec<Token>,
    lines: HashMap<u32, Vec<usize>>,
    values: HashMap<String, Vec<usize>>,
}

impl TextDocument {
    pub fn build_from_path(path: &str) -> Result<Self, DocumentError> {
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(Self {
                language_id: "sway".into(),
                version: 1,
                uri: path.into(),
                content: Rope::from_str(&content),
                tokens: vec![],
                lines: HashMap::new(),
                values: HashMap::new(),
            }),
            Err(_) => Err(DocumentError::DocumentNotFound),
        }
    }

    pub fn get_token_at_position(&self, position: Position) -> Option<&Token> {
        let line = position.line;

        if let Some(indices) = self.lines.get(&line) {
            for index in indices {
                let token = &self.tokens[*index];
                if token.is_within_character_range(position.character) {
                    return Some(token);
                }
            }
        }

        None
    }

    pub fn get_all_tokens_by_single_name(&self, name: &str) -> Option<Vec<&Token>> {
        if let Some(indices) = self.values.get(name) {
            let tokens = indices.iter().map(|index| &self.tokens[*index]).collect();
            Some(tokens)
        } else {
            None
        }
    }

    pub fn get_declared_token(&self, name: &str) -> Option<&Token> {
        if let Some(indices) = self.values.get(name) {
            for index in indices {
                let token = &self.tokens[*index];
                if token.is_initial_declaration() {
                    return Some(token);
                }
            }
        }
        None
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn parse(&mut self) -> Result<Vec<Diagnostic>, DocumentError> {
        self.clear_tokens();
        self.clear_hash_maps();

        match self.parse_tokens_from_text() {
            Ok((tokens, diagnostics)) => {
                self.store_tokens(tokens);
                Ok(diagnostics)
            }
            Err(diagnostics) => Err(DocumentError::FailedToParse(diagnostics)),
        }
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
    fn parse_tokens_from_text(&self) -> Result<(Vec<Token>, Vec<Diagnostic>), Vec<Diagnostic>> {
        let text = Arc::from(self.get_text());
        let parsed_result = parse(text, None);
        match parsed_result.value {
            None => Err(capabilities::diagnostic::get_diagnostics(
                parsed_result.warnings,
                parsed_result.errors,
            )),
            Some(value) => {
                let mut tokens = vec![];

                if let TreeType::Library { name } = value.tree_type {
                    // TODO
                    // Is library name necessary to store for the LSP?
                    let token = Token::from_ident(&name, TokenType::Library);
                    tokens.push(token);
                };
                for node in value.tree.root_nodes {
                    traverse_node(node, &mut tokens);
                }

                Ok((
                    tokens,
                    capabilities::diagnostic::get_diagnostics(
                        parsed_result.warnings,
                        parsed_result.errors,
                    ),
                ))
            }
        }
    }

    fn store_tokens(&mut self, tokens: Vec<Token>) {
        self.tokens = Vec::with_capacity(tokens.len());

        for (index, token) in tokens.into_iter().enumerate() {
            let line = token.get_line_start();
            let token_name = token.name.clone();

            // insert to tokens
            self.tokens.push(token);

            // insert index into hashmap for lines
            match self.lines.get_mut(&line) {
                Some(v) => {
                    v.push(index);
                }
                None => {
                    self.lines.insert(line, vec![index]);
                }
            }

            // insert index into hashmap for names
            match self.values.get_mut(&token_name) {
                Some(v) => {
                    v.push(index);
                }
                None => {
                    self.values.insert(token_name, vec![index]);
                }
            }
        }
    }

    fn clear_hash_maps(&mut self) {
        self.lines = HashMap::new();
        self.values = HashMap::new();
    }

    fn clear_tokens(&mut self) {
        self.tokens = vec![];
    }

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

#[derive(Debug)]
pub enum DocumentError {
    FailedToParse(Vec<Diagnostic>),
    DocumentNotFound,
    DocumentAlreadyStored,
}
