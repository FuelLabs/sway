use crate::{Parse, ParseResult, Parser};

use sway_ast::ItemConst;

impl Parse for ItemConst {
    fn parse(parser: &mut Parser) -> ParseResult<ItemConst> {
        let pub_token = parser.take();
        let const_token = parser.parse()?;
        let name = parser.parse()?;
        let ty_opt = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            }
            None => None,
        };
        let eq_token_opt = parser.take();
        let expr_opt = match &eq_token_opt {
            Some(_eq) => Some(parser.parse()?),
            None => None,
        };
        // Use the default here since the braces parsing is expecting
        // a semicolon, that allows us to re-use the same parsing code
        // between associated consts and module-level consts.
        let semicolon_token = parser.peek().unwrap_or_default();
        Ok(ItemConst {
            pub_token,
            const_token,
            name,
            ty_opt,
            eq_token_opt,
            expr_opt,
            semicolon_token,
        })
    }
}
