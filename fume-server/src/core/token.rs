use core_lang::{
    AstNode, AstNodeContent, Declaration, Expression, Ident, Span, VariableDeclaration,
};
use lspower::lsp::{Position, Range};

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range,
    pub content_type: ContentType,
    pub name: String,
    pub line_start: u32,
    pub length: u32,
}

impl Token {
    pub fn new(span: Span, name: String, content_type: ContentType) -> Self {
        let range = get_range_from_span(&span);

        Self {
            range,
            name,
            content_type,
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

    pub fn from_variable(variable: &VariableDeclaration) -> Self {
        let ident = &variable.name;
        let span = ident.span.clone();
        let name = ident.primary_name;
        // todo
        // we could add type of variable as well? from type_ascription: TypeInfo field
        Token::new(
            span,
            name.into(),
            ContentType::Declaration(DeclarationType::Variable),
        )
    }

    pub fn from_ident(ident: Ident, content_type: ContentType) -> Self {
        Token::new(ident.span.clone(), ident.primary_name.into(), content_type)
    }

    pub fn is_initial_declaration(&self) -> bool {
        if let ContentType::Declaration(ref dec) = self.content_type {
            if &DeclarationType::Reassignment == dec {
                return false;
            }
            return true;
        }

        false
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
        Declaration::FunctionDeclaration(func) => {
            let ident = func.name;
            let token =
                Token::from_ident(ident, ContentType::Declaration(DeclarationType::Function));
            tokens.push(token);

            for node in func.body.contents {
                traverse_node(node, tokens);
            }
        }
        Declaration::Reassignment(reassignment) => {
            let ident = reassignment.lhs;
            let token = Token::from_ident(
                ident,
                ContentType::Declaration(DeclarationType::Reassignment),
            );
            tokens.push(token);

            handle_expression(reassignment.rhs, tokens);
        }

        Declaration::TraitDeclaration(trait_dec) => {
            let ident = trait_dec.name;
            let token = Token::from_ident(ident, ContentType::Declaration(DeclarationType::Trait));
            tokens.push(token);

            // todo
            // traverse methods: Vec<FunctionDeclaration<'sc>> field as well ?
        }
        Declaration::StructDeclaration(struct_dec) => {
            let ident = struct_dec.name;
            let token = Token::from_ident(ident, ContentType::Declaration(DeclarationType::Struct));
            tokens.push(token);
        }
        Declaration::EnumDeclaration(enum_dec) => {
            let ident = enum_dec.name;
            let token = Token::from_ident(ident, ContentType::Declaration(DeclarationType::Enum));
            tokens.push(token);
        }
        _ => {}
    };
}

fn handle_expression(exp: Expression, tokens: &mut Vec<Token>) {
    match exp {
        Expression::CodeBlock {
            span: _span,
            contents,
        } => {
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
            let token = Token::from_ident(
                ident,
                ContentType::Expression(ExpressionType::FunctionApplication),
            );
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeclarationType {
    Library,
    Variable,
    Function,
    Reassignment,
    Enum,
    Trait,
    Struct,
    ImplTrait,
    ImplSelf,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionType {
    FunctionApplication,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Declaration(DeclarationType),
    Expression(ExpressionType),
}
