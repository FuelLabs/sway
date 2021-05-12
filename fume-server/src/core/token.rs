use std::fmt;

use lspower::lsp::{Position, Range};
use parser::{Rule, Span};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range,
    pub token_type: TokenType,
    pub name: String,
    pub line_start: u32,
    is_multi_line: bool,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Library,
    Variable,
    Function,
    Enum,
    Trait,
    Struct,
}

impl Token {
    pub fn new(span: Span, name: String, token_type: TokenType) -> Self {
        let range = get_range(&span);

        Self {
            range,
            token_type,
            name,
            is_multi_line: range.start.line != range.end.line,
            line_start: range.start.line,
        }
    }

    pub fn contains_character(&self, character: u32) -> bool {
        let range = self.range;
        !self.is_multi_line
            && character >= range.start.character
            && character <= range.end.character
    }

    pub fn get_line_start(&self) -> u32 {
        self.line_start
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} - {} ~> {:?}",
            self.token_type, self.name, self.range
        )
    }
}

pub fn pair_rule_to_token(pair: &Pair<Rule>) -> Option<Token> {
    let span = pair.as_span();

    match pair.as_rule() {
        Rule::library_name => {
            let library_name = pair.as_str().into();
            Some(Token::new(span, library_name, TokenType::Library))
        }
        Rule::var_name => {
            let var_name = pair.as_str().into();
            Some(Token::new(span, var_name, TokenType::Variable))
        }
        Rule::fn_decl_name => {
            let func_name = pair.as_str().into();
            Some(Token::new(span, func_name, TokenType::Function))
        }
        Rule::enum_name => {
            let enum_name = pair.as_str().into();
            Some(Token::new(span, enum_name, TokenType::Enum))
        }
        Rule::trait_name => {
            let trait_name = pair.as_str().into();
            Some(Token::new(span, trait_name, TokenType::Trait))
        }
        Rule::struct_name => {
            let struct_name = pair.as_str().into();
            Some(Token::new(span, struct_name, TokenType::Struct))
        }
        _ => None,
    }
}

fn get_range(span: &Span) -> Range {
    let start = span.start_pos().line_col();
    let end = span.end_pos().line_col();

    let start_line = start.0 as u32 - 1;
    let start_character = start.1 as u32 - 1;

    let end_line = end.0 as u32 - 1;
    let end_character = end.1 as u32 - 2;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}
