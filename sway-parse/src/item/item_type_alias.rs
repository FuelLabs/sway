use crate::{Parse, ParseResult, Parser};

use sway_ast::ItemTypeAlias;

impl Parse for ItemTypeAlias {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTypeAlias> {
        let visibility = parser.take();
        let type_token = parser.parse()?;
        let name = parser.parse()?;
        let eq_token = parser.parse()?;
        let ty = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemTypeAlias {
            visibility,
            name,
            type_token,
            eq_token,
            ty,
            semicolon_token,
        })
    }
}
