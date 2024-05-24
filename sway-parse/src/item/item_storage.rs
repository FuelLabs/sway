use crate::{Parse, ParseResult, Parser};

use sway_ast::{keywords::InToken, ItemStorage, Literal, StorageField};
use sway_error::parser_error::ParseErrorKind;
use sway_types::Spanned;

impl Parse for StorageField {
    fn parse(parser: &mut Parser) -> ParseResult<StorageField> {
        let name = parser.parse()?;
        let in_token : Option<InToken> = parser.take();
        let mut key_opt: Option<Literal> = None;
        if in_token.is_some() {
            key_opt = Some(parser.parse()?);
            return Err(parser.emit_error_with_span(
                ParseErrorKind::ExpectedStorageKeyU256,
                key_opt.unwrap().span(),
            ));
        }
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        let eq_token = parser.parse()?;
        let initializer = parser.parse()?;
        Ok(StorageField {
            name,
            in_token,
            key: key_opt,
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
