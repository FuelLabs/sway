mod attribute;
mod brackets;
mod dependency;
mod error;
mod expr;
mod generics;
pub mod handler;
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

use crate::handler::Handler;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseFileError {
    Lex(LexError),
    Parse(ErrorEmitted),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
pub enum ParseFileErrorStandalone {
    #[error(transparent)]
    Lex(LexError),
    #[error("Unable to parse: {}", .0.iter().map(|x| x.kind.to_string()).collect::<Vec<String>>().join("\n"))]
    Parse(Vec<ParseError>),
}

pub fn parse_file_standalone(
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Module, ParseFileErrorStandalone> {
    let handler = Handler::default();
    parse_file(&handler, src, path).map_err(|err| match err {
        ParseFileError::Lex(l) => ParseFileErrorStandalone::Lex(l),
        ParseFileError::Parse(_) => ParseFileErrorStandalone::Parse(handler.into_errors()),
    })
}

pub fn parse_file(
    handler: &Handler,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Module, ParseFileError> {
    let token_stream = match lex(&src, 0, src.len(), path) {
        Ok(token_stream) => token_stream,
        Err(error) => return Err(ParseFileError::Lex(error)),
    };
    match Parser::new(&token_stream, handler).parse_to_end() {
        Ok((module, _parser_consumed)) => Ok(module),
        Err(error) => Err(ParseFileError::Parse(error)),
    }
}
