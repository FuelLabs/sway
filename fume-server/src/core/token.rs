use lspower::lsp::{Position, Range};
use parser::{Rule, Span};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range,
    pub token_type: TokenType,
    pub expression_type: ExpressionType,
    pub name: String,
    pub line_start: u32,
    pub length: u32,
}

impl Token {
    pub fn new(
        span: Span,
        name: String,
        token_type: TokenType,
        expression_type: ExpressionType,
    ) -> Self {
        let range = get_range(&span);

        Self {
            range,
            token_type,
            name,
            expression_type,
            line_start: range.start.line,
            length: range.end.character - range.start.character + 1,
        }
    }

    pub fn is_within_character_range(&self, character: u32) -> bool {
        let range = self.range;
        character >= range.start.character && character <= range.end.character
    }

    pub fn get_line_start(&self) -> u32 {
        self.line_start
    }
}

pub fn pair_rule_to_token(pair: &Pair<Rule>) -> Option<Token> {
    // TODO
    // add more rules
    let span = pair.as_span();

    match pair.as_rule() {
        Rule::library_name => {
            let library_name = pair.as_str().into();
            Some(Token::new(
                span,
                library_name,
                TokenType::Library,
                ExpressionType::Declaration,
            ))
        }
        Rule::var_name => {
            let var_name = pair.as_str().into();
            Some(Token::new(
                span,
                var_name,
                TokenType::Variable,
                ExpressionType::Declaration,
            ))
        }
        Rule::fn_decl_name => {
            let func_name = pair.as_str().into();
            Some(Token::new(
                span,
                func_name,
                TokenType::Function,
                ExpressionType::Declaration,
            ))
        }
        Rule::enum_name => {
            let enum_name = pair.as_str().into();
            Some(Token::new(
                span,
                enum_name,
                TokenType::Enum,
                ExpressionType::Declaration,
            ))
        }
        Rule::trait_name => {
            let trait_name = pair.as_str().into();
            Some(Token::new(
                span,
                trait_name,
                TokenType::Trait,
                ExpressionType::Declaration,
            ))
        }
        Rule::struct_name => {
            let struct_name = pair.as_str().into();
            Some(Token::new(
                span,
                struct_name,
                TokenType::Struct,
                ExpressionType::Declaration,
            ))
        }
        Rule::fn_name => {
            let fn_name = pair.as_str().into();
            Some(Token::new(
                span,
                fn_name,
                TokenType::Function,
                ExpressionType::Usage,
            ))
        }
        Rule::var_exp => {
            let var_name = pair.as_str().into();
            Some(Token::new(
                span,
                var_name,
                TokenType::Variable,
                ExpressionType::Usage,
            ))
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

// TODO
// add more types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Library,
    Variable,
    Function,
    Enum,
    Trait,
    Struct,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionType {
    Declaration,
    Usage,
}
