use crate::{Parse, ParseResult, Parser};

use sway_ast::{
    attribute::Annotated,
    keywords::{ColonToken, InToken},
    Braces, CommaToken, Expr, ItemStorage, Punctuated, StorageEntry, StorageField,
};
use sway_types::BaseIdent;

impl Parse for StorageEntry {
    fn parse(parser: &mut Parser) -> ParseResult<StorageEntry> {
        let name: BaseIdent = parser.parse()?;
        let mut field = None;
        let mut namespace = None;
        if parser.peek::<ColonToken>().is_some() || parser.peek::<InToken>().is_some() {
            let mut f: StorageField = parser.parse()?;
            f.name = name.clone();
            field = Some(f);
        } else {
            let n: Braces<Punctuated<Annotated<Box<StorageEntry>>, CommaToken>> = parser.parse()?;
            namespace = Some(n);
        }
        Ok(StorageEntry {
            name,
            namespace,
            field,
        })
    }
}

impl Parse for StorageField {
    fn parse(parser: &mut Parser) -> ParseResult<StorageField> {
        let name = BaseIdent::dummy(); // Name will be overridden in StorageEntry parse.
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
        let entries = parser.parse()?;
        Ok(ItemStorage {
            storage_token,
            entries,
        })
    }
}
