use crate::{Parse, ParseResult, Parser};

use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::ItemStruct;

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStruct> {
        Ok(ItemStruct {
            visibility: parser.take(),
            struct_token: parser.parse()?,
            name: parser.parse()?,
            generics: parser.guarded_parse::<OpenAngleBracketToken, _>()?,
            where_clause_opt: parser.guarded_parse::<WhereToken, _>()?,
            fields: parser.parse()?,
        })
    }
}
