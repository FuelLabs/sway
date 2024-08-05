use sway_error::handler::Handler;

use crate::{priv_prelude::ParseToEnd, Parse, Parser};
use std::sync::Arc;

pub fn parse<T>(input: &str) -> T
where
    T: Parse,
{
    let handler = Handler::default();
    let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
    let r = Parser::new(&handler, &ts).parse();

    if handler.has_errors() || handler.has_warnings() {
        panic!("{:?}", handler.consume());
    }

    r.unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
}

pub fn parse_to_end<T>(input: &str) -> T
where
    T: ParseToEnd,
{
    let handler = <_>::default();
    let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
    let r = Parser::new(&handler, &ts).parse_to_end().map(|(m, _)| m);

    if handler.has_errors() || handler.has_warnings() {
        panic!("{:?}", handler.consume());
    }

    r.unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
}
