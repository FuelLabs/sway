use crate::{Parse, ParseResult, Parser};

use sway_ast::punctuated::Punctuated;
use sway_ast::{WhereBound, WhereClause};

impl Parse for WhereClause {
    fn parse(parser: &mut Parser) -> ParseResult<WhereClause> {
        let where_token = parser.parse()?;
        let mut value_separator_pairs = Vec::new();
        let final_value_opt = loop {
            let ty_name = match parser.take() {
                Some(ty_name) => ty_name,
                None => break None,
            };
            let colon_token = parser.parse()?;
            let bounds = parser.parse()?;
            let where_bound = WhereBound {
                ty_name,
                colon_token,
                bounds,
            };
            match parser.take() {
                Some(comma_token) => value_separator_pairs.push((where_bound, comma_token)),
                None => break Some(Box::new(where_bound)),
            }
        };
        let bounds = Punctuated {
            value_separator_pairs,
            final_value_opt,
        };
        Ok(WhereClause {
            where_token,
            bounds,
        })
    }
}
