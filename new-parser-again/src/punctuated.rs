use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Punctuated<T, P> {
    pub value_separator_pairs: Vec<(T, P)>,
    pub final_value_opt: Option<Box<T>>,
}

impl<T, P> ParseToEnd for Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(Punctuated<T, P>, ParserConsumed<'a>)> {
        let mut value_separator_pairs = Vec::new();
        loop {
            if let Some(consumed) = parser.check_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: None,
                };
                return Ok((punctuated, consumed));
            }
            let value = parser.parse()?;
            if let Some(consumed) = parser.check_empty() {
                let punctuated = Punctuated {
                    value_separator_pairs,
                    final_value_opt: Some(Box::new(value)),
                };
                return Ok((punctuated, consumed));
            }
            let separator = parser.parse()?;
            value_separator_pairs.push((value, separator));
        }
    }
}

