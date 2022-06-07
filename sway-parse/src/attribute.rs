use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Annotated<T: Parse> {
    pub attribute_list: Vec<AttributeDecl>,
    pub value: T,
}

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

// Attributes can have any number of arguments:
//
//    #[attribute]
//    #[attribute()]
//    #[attribute(value)]
//    #[attribute(value0, value1, value2)]

#[derive(Clone, Debug)]
pub struct AttributeDecl {
    pub hash_token: HashToken,
    pub attribute: SquareBrackets<Attribute>,
}

impl Spanned for AttributeDecl {
    fn span(&self) -> Span {
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

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Parens<Punctuated<Ident, CommaToken>>>,
}

impl Spanned for Attribute {
    fn span(&self) -> Span {
        self.args
            .as_ref()
            .map(|args| Span::join(self.name.span(), args.span()))
            .unwrap_or_else(|| self.name.span())
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
