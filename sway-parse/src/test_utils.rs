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
