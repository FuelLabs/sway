use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::punctuated::Punctuated;

impl<T, P> Parse for Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse(mut parser: &mut Parser) -> ParseResult<Punctuated<T, P>> {
        let mut value_separator_pairs = Vec::new();
        loop {
            if parser.is_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: None,
                };
                return Ok(punctuated);
            }
            let value = parser.parse()?;
            if parser.is_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: Some(Box::new(value)),
                };
                return Ok(punctuated);
            }
            let separator = parser.parse()?;
            value_separator_pairs.push((value, separator));
        }
    }
}
