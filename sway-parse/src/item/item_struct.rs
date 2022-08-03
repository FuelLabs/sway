use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::ItemStruct;

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStruct> {
        let visibility = parser.take();
        let struct_token = parser.parse()?;
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
        Ok(ItemStruct {
            visibility,
            struct_token,
            name,
            generics,
            where_clause_opt,
            fields,
        })
    }
}
