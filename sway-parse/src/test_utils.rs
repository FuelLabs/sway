use crate::{priv_prelude::ParseToEnd, Parse, Parser};
use sway_error::handler::Handler;
use sway_features::ExperimentalFeatures;

pub fn parse<T>(input: &str) -> T
where
    T: Parse,
{
    let handler = Handler::default();
    let ts = crate::token::lex(&handler, input.into(), 0, input.len(), None).unwrap();
    let r = Parser::new(&handler, &ts, ExperimentalFeatures::default()).parse();

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
    let ts = crate::token::lex(&handler, input.into(), 0, input.len(), None).unwrap();
    let r = Parser::new(&handler, &ts, ExperimentalFeatures::default())
        .parse_to_end()
        .map(|(m, _)| m);

    if handler.has_errors() || handler.has_warnings() {
        panic!("{:?}", handler.consume());
    }

    r.unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
}
