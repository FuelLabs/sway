use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeDecl};
use sway_ast::brackets::Parens;
use sway_ast::keywords::{HashToken, StorageToken};
use sway_types::Ident;

impl<T: Parse> Parse for Annotated<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        // Parse the attribute list.
        let mut attribute_list = Vec::new();
        while let Some(attr) = parser.guarded_parse::<HashToken, _>()? {
            attribute_list.push(attr);
        }

        // Parse the `T` value.
        let value = parser.parse()?;

        Ok(Annotated {
            attribute_list,
            value,
        })
    }
}

impl Parse for AttributeDecl {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        Ok(AttributeDecl {
            hash_token: parser.parse()?,
            attribute: parser.parse()?,
        })
    }
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let name = if let Some(storage) = parser.take::<StorageToken>() {
            Ident::from(storage)
        } else {
            parser.parse()?
        };
        let args = Parens::try_parse(parser)?;
        Ok(Attribute { name, args })
    }
}

impl ParseToEnd for Attribute {
    fn parse_to_end<'a, 'e>(mut parser: Parser<'a, 'e>) -> ParseResult<(Self, ParserConsumed<'a>)> {
        let attrib = parser.parse()?;
        match parser.check_empty() {
            Some(consumed) => Ok((attrib, consumed)),
            None => Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAttribute)),
        }
    }
}
