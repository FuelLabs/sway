mod attribute;
mod brackets;
mod dependency;
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

use crate::priv_prelude::*;
pub use crate::{
    parse::Parse,
    parser::Parser,
    token::{lex, lex_commented},
};

use sway_ast::Module;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};

use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("Unable to parse: {}", self.0.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"))]
pub struct ParseFileError(pub Vec<CompileError>);

pub fn parse_file_standalone(
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Module, ParseFileError> {
    let handler = Handler::default();
    parse_file(&handler, src, path).map_err(|_| ParseFileError(handler.into_errors()))
}

pub fn parse_file(
    handler: &Handler,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Module, ErrorEmitted> {
    let token_stream = lex(handler, &src, 0, src.len(), path)?;
    match Parser::new(handler, &token_stream).parse_to_end() {
        Ok((module, _parser_consumed)) => Ok(module),
        Err(error) => Err(error),
    }
}
