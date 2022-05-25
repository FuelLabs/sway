use super::token::Token;
use super::token_type::TokenType;
use crate::{capabilities, core::token::traverse_node};
use ropey::Rope;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use sway_core::{parse, TreeType};
use sway_core::{
    BuildConfig,
    CompileResult,
    TypedParseTree,
    semantic_analysis::{
        namespace, Namespace,
        ast_node::TypedAstNode,
    },
};
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
        
        self.test_typed_parse();

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

    pub fn test_typed_parse(&self) {
        use sway_types::ident::Ident;
        use sway_core::semantic_analysis::ast_node::TypedAstNodeContent;
        use super::traverse_typed_tree as ttt;

        let mut tokens: BTreeMap<Ident, TypedAstNodeContent> = BTreeMap::new();

        if let Some(all_nodes) = self.parse_typed_tokens_from_text() {
            for node in &all_nodes {
                ttt::traverse_node(node, &mut tokens);
            }
        }

        for (ident, token) in &tokens {
            ttt::debug_print_ident(ident, token);
        }

        //eprintln!("{:#?}", tokens.keys());
    
        let cursor_position = Position::new(25, 14); //Cursor's hovered over the position var decl in main()
        //let cursor_position = Position::new(29, 18); //Cursor's hovered over the ~Particle in p = decl in main()

        // Check if the code editor's cursor is currently over an of our collected tokens
        if let Some(ident) = ttt::ident_at_position(cursor_position, &tokens) {
            // Retrieve the typed_ast_node from our BTreeMap
            if let Some(token) = tokens.get(ident) {
                //eprintln!("token = {:#?}", token);

                // Look up the tokens TypeId
                let type_id = ttt::type_id(token);
                eprintln!("type_id = {:#?}", type_id);

                // Use the TypeId to look up the actual type (I think there is a method in the type_engine for this)
                let type_info = sway_core::type_engine::look_up_type_id(type_id);
                eprintln!("type_info = {:#?}", type_info);

                // Find the ident / span on the returned type
    
                // Contruct a go_to LSP request from the declerations span
            }
        }    
    }
}

// private methods
impl TextDocument {
    fn parse_typed_tokens_from_text(&self) -> Option<Vec<TypedAstNode>> {
        let text = Arc::from(self.get_text());
        let parse_tree = parse(text, None).value.unwrap();

        let program_type = &parse_tree.tree_type;
        let file_name = std::path::PathBuf::from(self.get_uri());
        let build_config = BuildConfig::root_from_file_name_and_manifest_path(
            file_name,
            Default::default(),
        );
        let mut dead_code_graph = Default::default();
        let mut namespace = Namespace::init_root(namespace::Module::default());

        let CompileResult {
            value: typed_parse_tree_result,
            warnings: _new_warnings,
            errors: _new_errors,
        } = TypedParseTree::type_check(
            parse_tree.tree,
            &mut namespace,
            program_type,
            &build_config,
            &mut dead_code_graph,
        );
        //eprintln!("typed_parse_tree_result: {:#?}", typed_parse_tree_result);

        if let Some(typed_parse_tree) = typed_parse_tree_result {
            match typed_parse_tree {
                TypedParseTree::Script{ main_function, namespace, declarations, all_nodes } => {
                    //eprintln!("main_function: {:#?}", main_function);
                    //eprintln!("namespace: {:#?}", namespace);
                    //eprintln!("declarations: {:#?}", declarations);
                    //eprintln!("all_nodes: {:#?}", all_nodes);
                    return Some(all_nodes);
                }
                _ => (),
            }
        }
        None
    }

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
