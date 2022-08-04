use crate::{Parse, ParseResult, Parser};

use sway_ast::ItemFn;

impl Parse for ItemFn {
    fn parse(parser: &mut Parser) -> ParseResult<ItemFn> {
        let fn_signature = parser.parse()?;
        let body = parser.parse()?;
        Ok(ItemFn { fn_signature, body })
    }
}
