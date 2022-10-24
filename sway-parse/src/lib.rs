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
use sway_error::handler::{ErrorEmitted, Handler};

use std::{path::PathBuf, sync::Arc};

pub fn parse_file(
    handler: &Handler,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Module, ErrorEmitted> {
    let ts = lex(handler, &src, 0, src.len(), path)?;
    Parser::new(handler, &ts).parse_to_end().map(|(m, _)| m)
}
