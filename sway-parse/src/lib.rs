mod attribute;
mod brackets;
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
mod submodule;
mod token;
mod ty;
mod where_clause;

use crate::priv_prelude::*;
pub use crate::{
    keywords::RESERVED_KEYWORDS,
    parse::Parse,
    parser::Parser,
    token::{lex, lex_commented},
};

use sway_ast::{
    attribute::Annotated,
    token::{DocComment, DocStyle},
    Module, ModuleKind,
};
use sway_error::handler::{ErrorEmitted, Handler};

use std::{path::PathBuf, sync::Arc};

pub fn parse_file(
    handler: &Handler,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Annotated<Module>, ErrorEmitted> {
    let ts = lex(handler, &src, 0, src.len(), path)?;
    Parser::new(handler, &ts).parse_to_end().map(|(m, _)| m)
}

pub fn parse_module_kind(
    handler: &Handler,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<ModuleKind, ErrorEmitted> {
    let ts = lex(handler, &src, 0, src.len(), path)?;
    let mut parser = Parser::new(handler, &ts);
    while let Some(DocComment {
        doc_style: DocStyle::Inner,
        ..
    }) = parser.peek()
    {
        parser.parse::<DocComment>()?;
    }
    parser.parse()
}

#[cfg(test)]
mod test_utils {
    use crate::{priv_prelude::ParseToEnd, Parse, Parser};
    use std::sync::Arc;

    pub fn parse<T>(input: &str) -> T
    where
        T: Parse,
    {
        let handler = <_>::default();
        let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
        Parser::new(&handler, &ts)
            .parse()
            .unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
    }

    pub fn parse_to_end<T>(input: &str) -> T
    where
        T: ParseToEnd,
    {
        let handler = <_>::default();
        let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
        Parser::new(&handler, &ts)
            .parse_to_end()
            .map(|(m, _)| m)
            .unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
    }
}
