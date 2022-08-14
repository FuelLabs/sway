use crate::{Parse, ParseResult, Parser};

use sway_ast::{ItemBreak, ItemContinue};

impl Parse for ItemBreak {
    fn parse(parser: &mut Parser) -> ParseResult<ItemBreak> {
        let break_token = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemBreak {
            break_token,
            semicolon_token,
        })
    }
}

impl Parse for ItemContinue {
    fn parse(parser: &mut Parser) -> ParseResult<ItemContinue> {
        let break_token = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemContinue {
            break_token,
            semicolon_token,
        })
    }
}
