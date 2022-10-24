use crate::{Parse, ParseResult, Parser};

use sway_ast::{
    keywords::{ClassToken, Keyword, OpenAngleBracketToken, StructToken, WhereToken},
    ItemStruct,
};
use sway_error::parser_error::ParseErrorKind;
use sway_types::Spanned;

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStruct> {
        // Parse `struct`, or recover on `class` as if `struct` was written.
        let struct_token = if let Some(ct) = parser.take::<ClassToken>() {
            parser.emit_error(ParseErrorKind::UnexpectedClass);
            StructToken::new(ct.span())
        } else {
            parser.parse()?
        };

        Ok(ItemStruct {
            visibility: None,
            struct_token,
            name: parser.parse()?,
            generics: parser.guarded_parse::<OpenAngleBracketToken, _>()?,
            where_clause_opt: parser.guarded_parse::<WhereToken, _>()?,
            fields: parser.parse()?,
        })
    }
}
