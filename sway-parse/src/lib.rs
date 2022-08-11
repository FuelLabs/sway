mod attribute;
mod brackets;
mod dependency;
mod error;
mod expr;
mod generics;
mod item;
mod keywords;
mod literal;
mod module;
mod parse;
mod parser;
mod path;
mod pattern;
mod priv_prelude;
mod punctuated;
mod token;
mod ty;
mod where_clause;

pub use crate::{
    error::{ParseError, ParseErrorKind},
    parse::Parse,
    parser::Parser,
    token::LexError,
    token::{lex, lex_commented},
};

use sway_ast::Module;

use crate::priv_prelude::*;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
pub enum ParseFileError {
    #[error(transparent)]
    Lex(LexError),
    #[error("Unable to parse: {}", .0.iter().map(|x| x.kind.to_string()).collect::<Vec<String>>().join("\n"))]
    Parse(Vec<ParseError>),
}

pub fn parse_file(src: Arc<str>, path: Option<Arc<PathBuf>>) -> Result<Module, ParseFileError> {
    let token_stream = match lex(&src, 0, src.len(), path) {
        Ok(token_stream) => token_stream,
        Err(error) => return Err(ParseFileError::Lex(error)),
    };
    let mut errors = Vec::new();
    let parser = Parser::new(&token_stream, &mut errors);
    let module = match parser.parse_to_end() {
        Ok((module, _parser_consumed)) => module,
        Err(_error_emitted) => return Err(ParseFileError::Parse(errors)),
    };
    Ok(module)
}
