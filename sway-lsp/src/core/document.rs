#![allow(dead_code)]

use super::token::{TokenMap, TokenType};
use super::{traverse_parse_tree, traverse_typed_tree};

use crate::{capabilities, utils};
use forc::utils::SWAY_GIT_TAG;
use forc_pkg::{self as pkg};
use ropey::Rope;
use std::{collections::HashMap, path::PathBuf};
use sway_core::{
    semantic_analysis::ast_node::TypedAstNode, CompileAstResult, CompileResult, ParseProgram,
    TypeInfo,
};
use sway_types::Ident;
use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent};

#[derive(Debug)]
pub struct TextDocument {
    #[allow(dead_code)]
    language_id: String,
    #[allow(dead_code)]
    version: i32,
    uri: String,
    content: Rope,
    token_map: TokenMap,
}

impl TextDocument {
    pub fn build_from_path(path: &str) -> Result<Self, DocumentError> {
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(Self {
                language_id: "sway".into(),
                version: 1,
                uri: path.into(),
                content: Rope::from_str(&content),
                token_map: HashMap::new(),
            }),
            Err(_) => Err(DocumentError::DocumentNotFound),
        }
    }

    /// Check if the code editor's cursor is currently over an of our collected tokens
    pub fn get_token_at_position(&self, position: Position) -> Option<(Ident, &TokenType)> {
        match utils::common::ident_and_span_at_position(position, &self.token_map) {
            Some((ident, _)) => {
                // Retrieve the TokenType from our HashMap
                self.token_map
                    .get(&utils::token::to_ident_key(&ident))
                    .map(|token| (ident.clone(), token))
            }
            None => None,
        }
    }

    pub fn get_all_references(&self, token: &TokenType) -> Vec<(&Ident, &TokenType)> {
        let current_type_id = utils::token::get_type_id(token);

        self.token_map
            .iter()
            .filter(|((_, _), token)| {
                if token.typed.is_some() {
                    current_type_id == utils::token::get_type_id(token)
                } else {
                    false
                }
            })
            .map(|((ident, _), token)| (ident, token))
            .collect()
    }

    pub fn get_declared_token_ident(&self, token: &TokenType) -> Option<Ident> {
        // Look up the tokens TypeId
        match utils::token::get_type_id(token) {
            Some(type_id) => {
                tracing::info!("type_id = {:#?}", type_id);

                // Use the TypeId to look up the actual type
                let type_info = sway_core::type_engine::look_up_type_id(type_id);
                tracing::info!("type_info = {:#?}", type_info);

                match type_info {
                    TypeInfo::UnknownGeneric { name }
                    | TypeInfo::Enum { name, .. }
                    | TypeInfo::Struct { name, .. }
                    | TypeInfo::Custom { name, .. } => Some(name),
                    _ => None,
                }
            }
            None => None,
        }
    }

    pub fn get_token_map(&self) -> &TokenMap {
        &self.token_map
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn parse(&mut self) -> Result<Vec<Diagnostic>, DocumentError> {
        self.clear_token_map();

        let manifest_dir = PathBuf::from(self.get_uri());
        let silent_mode = true;
        let locked = false;
        let offline = false;

        // TODO: match on any errors and report them back to the user in a future PR
        if let Ok(manifest) = pkg::ManifestFile::from_dir(&manifest_dir, SWAY_GIT_TAG) {
            if let Ok(plan) =
                pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline, SWAY_GIT_TAG)
            {
                if let Ok((parsed_res, _ast_res)) = pkg::check(&plan, silent_mode) {
                    let r = self.parse_tokens_from_text(parsed_res);
                    //self.test_typed_parse(ast_res);
                    return r;
                }
            }
        }

        Err(DocumentError::FailedToParse(vec![]))
    }

    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        let edit = self.build_edit(change);

        self.content.remove(edit.start_index..edit.end_index);
        self.content.insert(edit.start_index, edit.change_text);
    }

    pub fn get_text(&self) -> String {
        self.content.to_string()
    }

    pub fn test_typed_parse(&mut self, ast_res: CompileAstResult) {
        if let Some(all_nodes) = self.parse_typed_tokens_from_text(ast_res) {
            for node in &all_nodes {
                traverse_typed_tree::traverse_node(node, &mut self.token_map);
            }
        }

        for ((ident, _span), token) in &self.token_map {
            utils::debug::debug_print_ident_and_token(ident, token);
        }

        //let cursor_position = Position::new(25, 14); //Cursor's hovered over the position var decl in main()
        let cursor_position = Position::new(29, 18); //Cursor's hovered over the ~Particle in p = decl in main()

        if let Some((_, token)) = self.get_token_at_position(cursor_position) {
            // Look up the tokens TypeId
            if let Some(type_id) = utils::token::get_type_id(token) {
                tracing::info!("type_id = {:#?}", type_id);

                // Use the TypeId to look up the actual type
                let type_info = sway_core::type_engine::look_up_type_id(type_id);
                tracing::info!("type_info = {:#?}", type_info);
            }

            // Find the ident / span on the returned type

            // Contruct a go_to LSP request from the declerations span
        }
    }
}

// private methods
impl TextDocument {
    fn parse_typed_tokens_from_text(&self, ast_res: CompileAstResult) -> Option<Vec<TypedAstNode>> {
        match ast_res {
            CompileAstResult::Failure { .. } => None,
            CompileAstResult::Success { typed_program, .. } => Some(typed_program.root.all_nodes),
        }
    }

    fn parse_tokens_from_text(
        &mut self,
        parsed_result: CompileResult<ParseProgram>,
    ) -> Result<Vec<Diagnostic>, DocumentError> {
        match parsed_result.value {
            None => {
                let diagnostics = capabilities::diagnostic::get_diagnostics(
                    parsed_result.warnings,
                    parsed_result.errors,
                );
                Err(DocumentError::FailedToParse(diagnostics))
            }
            Some(parse_program) => {
                for node in &parse_program.root.tree.root_nodes {
                    traverse_parse_tree::traverse_node(node, &mut self.token_map);
                }

                Ok(capabilities::diagnostic::get_diagnostics(
                    parsed_result.warnings,
                    parsed_result.errors,
                ))
            }
        }
    }

    fn clear_token_map(&mut self) {
        self.token_map = HashMap::new();
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
