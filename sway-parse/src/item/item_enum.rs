use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::ItemEnum;

impl Parse for ItemEnum {
    fn parse(parser: &mut Parser) -> ParseResult<ItemEnum> {
        Ok(ItemEnum {
            visibility: parser.take(),
            enum_token: parser.parse()?,
            name: parser.parse()?,
            generics: parser.guarded_parse::<OpenAngleBracketToken, _>()?,
            where_clause_opt: parser.guarded_parse::<WhereToken, _>()?,
            fields: parser.parse()?,
        })
    }
}
