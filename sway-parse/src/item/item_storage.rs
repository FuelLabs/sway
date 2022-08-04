use crate::{Parse, ParseResult, Parser};

use sway_ast::{ItemStorage, StorageField};

impl Parse for StorageField {
    fn parse(parser: &mut Parser) -> ParseResult<StorageField> {
        let name = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        let eq_token = parser.parse()?;
        let initializer = parser.parse()?;
        Ok(StorageField {
            name,
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
