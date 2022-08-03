use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeDecl};
use sway_ast::brackets::Parens;
use sway_ast::keywords::{HashToken, StorageToken};
use sway_types::Ident;

impl<T: Parse> Parse for Annotated<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let mut attribute_list = Vec::new();
        loop {
            if parser.peek::<HashToken>().is_some() {
                attribute_list.push(parser.parse()?);
            } else {
                break;
            }
        }
        let value = parser.parse()?;
        Ok(Annotated {
            attribute_list,
            value,
        })
    }
}

impl Parse for AttributeDecl {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let hash_token = parser.parse()?;
        let attribute = parser.parse()?;
        Ok(AttributeDecl {
            hash_token,
            attribute,
        })
    }
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let storage = parser.take::<StorageToken>();
        let name = if let Some(storage) = storage {
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
