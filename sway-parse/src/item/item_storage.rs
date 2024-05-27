use crate::{Parse, ParseResult, Parser};

use sway_ast::{keywords::InToken, Expr, ItemStorage, StorageField};

impl Parse for StorageField {
    fn parse(parser: &mut Parser) -> ParseResult<StorageField> {
        let name = parser.parse()?;
        let in_token: Option<InToken> = parser.take();
        let mut key_opt: Option<Expr> = None;
        if in_token.is_some() {
            key_opt = Some(parser.parse()?);
        }
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        let eq_token = parser.parse()?;
        let initializer = parser.parse()?;
        Ok(StorageField {
            name,
            in_token,
            key_expr: key_opt,
            colon_token,
            ty,
            eq_token,
            initializer,
        })
    }
}

impl Parse for ItemStorage {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStorage> {
        let storage_token = parser.parse()?;
        let fields = parser.parse()?;
        Ok(ItemStorage {
            storage_token,
            fields,
        })
    }
}
