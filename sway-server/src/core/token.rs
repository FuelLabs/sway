use super::token_type::{get_trait_details, TokenType, VariableDetails};
use crate::{
    core::token_type::{get_function_details, get_struct_details},
    utils::common::extract_var_body,
};
use core_lang::{
    AstNode, AstNodeContent, Declaration, Expression, Ident, Span, VariableDeclaration,
};
use lspower::lsp::{Position, Range};

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range,
    pub token_type: TokenType,
    pub name: String,
    pub line_start: u32,
    pub length: u32,
}

impl Token {
    pub fn new(span: &Span, name: String, token_type: TokenType) -> Self {
        let range = get_range_from_span(span);

        Self {
            range,
            name,
            token_type,
            line_start: range.start.line,
            length: range.end.character - range.start.character + 1,
        }
    }

    pub fn is_within_character_range(&self, character: u32) -> bool {
        let range = self.range;
        character >= range.start.character && character <= range.end.character
    }

    pub fn is_same_type(&self, other_token: &Token) -> bool {
        if other_token.token_type == self.token_type {
            true
        } else {
            match (&other_token.token_type, &self.token_type) {
                (TokenType::FunctionApplication, TokenType::FunctionDeclaration(_)) => true,
                (TokenType::FunctionDeclaration(_), TokenType::FunctionApplication) => true,
                _ => false,
            }
        }
    }

    pub fn get_line_start(&self) -> u32 {
        self.line_start
    }

    pub fn from_variable(var_dec: &VariableDeclaration) -> Self {
        let ident = &var_dec.name;
        let name = ident.primary_name;
        let var_body = extract_var_body(var_dec);

        Token::new(
            &ident.span,
            name.into(),
            TokenType::Variable(VariableDetails {
                is_mutable: var_dec.is_mutable,
                var_body,
            }),
        )
    }

    pub fn from_ident(ident: &Ident, token_type: TokenType) -> Self {
        Token::new(&ident.span, ident.primary_name.into(), token_type)
    }

    pub fn from_span(span: Span, token_type: TokenType) -> Self {
        Token::new(&span, span.as_str().into(), token_type)
    }

    pub fn is_initial_declaration(&self) -> bool {
        match self.token_type {
            TokenType::Reassignment | TokenType::FunctionApplication => false,
            _ => true,
        }
    }
}

pub fn traverse_node(node: AstNode, tokens: &mut Vec<Token>) {
    match node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, tokens),
        AstNodeContent::Expression(exp) => handle_expression(exp, tokens),
        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_declaration(declaration: Declaration, tokens: &mut Vec<Token>) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            tokens.push(Token::from_variable(&variable));
            handle_expression(variable.body, tokens);
        }
        Declaration::FunctionDeclaration(func_dec) => {
            let ident = &func_dec.name;
            let token = Token::from_ident(
                ident,
                TokenType::FunctionDeclaration(get_function_details(&func_dec)),
            );
            tokens.push(token);

            for node in func_dec.body.contents {
                traverse_node(node, tokens);
            }
        }
        Declaration::Reassignment(reassignment) => {
            let token_type = TokenType::Reassignment;
            let token = match *reassignment.lhs {
                // a reassignment's lhs can _only_ be a variable expression or
                // struct field a subfield expression
                Expression::SubfieldExpression { span, .. } => Token::from_span(span, token_type),
                Expression::VariableExpression { name, .. } => Token::from_ident(&name, token_type),
                _ => {
                    unreachable!("any other reassignment lhs is invalid and cannot be constructed.")
                }
            };
            tokens.push(token);

            handle_expression(reassignment.rhs, tokens);
        }

        Declaration::TraitDeclaration(trait_dec) => {
            let ident = &trait_dec.name;
            let token = Token::from_ident(ident, TokenType::Trait(get_trait_details(&trait_dec)));
            tokens.push(token);

            // todo
            // traverse methods: Vec<FunctionDeclaration<'sc>> field as well ?
        }
        Declaration::StructDeclaration(struct_dec) => {
            let ident = &struct_dec.name;
            let token =
                Token::from_ident(ident, TokenType::Struct(get_struct_details(&struct_dec)));
            tokens.push(token);
        }
        Declaration::EnumDeclaration(enum_dec) => {
            let ident = enum_dec.name;
            let token = Token::from_ident(&ident, TokenType::Enum);
            tokens.push(token);
        }
        _ => {}
    };
}

fn handle_expression(exp: Expression, tokens: &mut Vec<Token>) {
    match exp {
        Expression::CodeBlock { span: _, contents } => {
            let nodes = contents.contents;

            for node in nodes {
                traverse_node(node, tokens);
            }
        }
        Expression::FunctionApplication {
            name,
            span: _span,
            arguments: _arguments,
        } => {
            let ident = name.suffix;
            let token = Token::from_ident(&ident, TokenType::FunctionApplication);
            tokens.push(token);

            // TODO
            // perform a for/in on arguments ?
        }
        // TODO
        // handle other expressions
        _ => {}
    }
}

fn get_range_from_span(span: &Span) -> Range {
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
