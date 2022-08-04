use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::ItemEnum;

impl Parse for ItemEnum {
    fn parse(parser: &mut Parser) -> ParseResult<ItemEnum> {
        let visibility = parser.take();
        let enum_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<OpenAngleBracketToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let where_clause_opt = match parser.peek::<WhereToken>() {
            Some(..) => {
                let where_clause = parser.parse()?;
                Some(where_clause)
            }
            None => None,
        };
        let fields = parser.parse()?;
        Ok(ItemEnum {
            visibility,
            enum_token,
            name,
            generics,
            where_clause_opt,
            fields,
        })
    }
}
