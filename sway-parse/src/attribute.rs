use crate::priv_prelude::*;

// Attributes can have any number of arguments:
//
//    #[attribute]
//    #[attribute()]
//    #[attribute(value)]
//    #[attribute(value0, value1, value2)]

pub struct AttributeDecl {
    pub hash_token: HashToken,
    pub attribute: SquareBrackets<Attribute>,
}

impl AttributeDecl {
    pub fn span(&self) -> Span {
        Span::join(self.hash_token.span(), self.attribute.span())
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

#[derive(Debug)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Parens<Punctuated<Ident, CommaToken>>>,
}

impl Attribute {
    pub fn span(&self) -> Span {
        self.args
            .as_ref()
            .map(|args| Span::join(self.name.span().clone(), args.span()))
            .unwrap_or_else(|| self.name.span().clone())
    }
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let name = parser.parse()?;
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
