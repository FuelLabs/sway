use std::collections::HashMap;

use lspower::lsp::{Diagnostic, Position, Range, TextDocumentContentChangeEvent, TextDocumentItem};
use parser::{self, HllParser, Rule};
use pest::Parser;
use ropey::Rope;

use crate::capabilities;

use super::token::{pair_rule_to_token, ExpressionType, Token};

#[derive(Debug)]
pub struct TextDocument {
    language_id: String,
    version: i32,
    uri: String,
    content: Rope,
    text: String,
    tokens: Vec<Token>,
    lines: HashMap<u32, Vec<usize>>,
    values: HashMap<String, Vec<usize>>,
}

impl TextDocument {
    pub fn new(item: &TextDocumentItem) -> Self {
        Self {
            language_id: item.language_id.clone(),
            version: item.version,
            uri: item.uri.to_string(),
            content: Rope::from_str(&item.text),
            text: item.text.clone(),
            tokens: vec![],
            lines: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn get_token_at_position(&self, position: Position) -> Option<Token> {
        let line = position.line;

        if let Some(indices) = self.lines.get(&line) {
            for index in indices {
                let token = &self.tokens[*index];
                if token.is_within_character_range(position.character) {
                    return Some(token.clone());
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

    pub fn get_token_by_name_and_expression_type(
        &self,
        name: &str,
        expression_type: ExpressionType,
    ) -> Option<Token> {
        if let Some(indices) = self.values.get(name) {
            for index in indices {
                let token = &self.tokens[*index];
                if token.expression_type == expression_type {
                    return Some(self.tokens[*index].clone());
                }
            }
        }
        None
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }

    pub fn parse(&mut self) -> Result<(), DocumentError> {
        self.sync_text_with_content();
        self.clear_tokens();
        self.clear_lines();

        // TODO
        // improve parsing flow
        match HllParser::parse(Rule::program, &self.text) {
            Ok(pairs) => {
                for pair in pairs.flatten() {
                    if let Some(token) = pair_rule_to_token(&pair) {
                        let line = token.get_line_start();
                        let token_name = token.name.clone();

                        // insert to tokens
                        self.tokens.push(token);

                        let token_index = self.tokens.len() - 1;

                        // insert index into hashmap for lines
                        match self.lines.get_mut(&line) {
                            Some(v) => {
                                v.push(token_index);
                            }
                            None => {
                                self.lines.insert(line, vec![token_index]);
                            }
                        }

                        // insert index into hashmap for names
                        match self.values.get_mut(&token_name) {
                            Some(v) => {
                                v.push(token_index);
                            }
                            None => {
                                self.values.insert(token_name, vec![token_index]);
                            }
                        }
                    }
                }

                Ok(())
            }
            Err(_) => match parser::parse(&self.text) {
                parser::CompileResult::Err { warnings, errors } => {
                    Err(DocumentError::FailedToParse(
                        capabilities::diagnostic::perform_diagnostics(warnings, errors),
                    ))
                }
                _ => Ok(()),
            },
        }
    }

    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        let edit = self.build_edit(change);

        self.content.remove(edit.start_index..edit.end_index);
        self.content.insert(edit.start_index, edit.change_text);
    }
}

// private methods
impl TextDocument {
    fn sync_text_with_content(&mut self) {
        self.text = self.content.to_string();
    }

    fn clear_lines(&mut self) {
        self.lines = HashMap::new();
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
