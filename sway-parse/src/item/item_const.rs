use crate::{Parse, ParseResult, Parser};

use sway_ast::ItemConst;

impl Parse for ItemConst {
    fn parse(parser: &mut Parser) -> ParseResult<ItemConst> {
        let visibility = parser.take();
        let const_token = parser.parse()?;
        let name = parser.parse()?;
        let ty_opt = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            }
            None => None,
        };
        let eq_token = parser.parse()?;
        let expr = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemConst {
            visibility,
            const_token,
            name,
            ty_opt,
            eq_token,
            expr,
            semicolon_token,
        })
    }
}
