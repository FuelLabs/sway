use crate::priv_prelude::{Peek, Peeker};
use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeDecl};
use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{HashToken, StorageToken, Token};
use sway_ast::punctuated::Punctuated;
use sway_ast::token::{DocComment, DocStyle};
use sway_types::Ident;

impl Peek for DocComment {
    fn peek(peeker: Peeker<'_>) -> Option<DocComment> {
        peeker.peek_doc_comment().ok().map(Clone::clone)
    }
}

impl Parse for DocComment {
    fn parse(parser: &mut Parser) -> ParseResult<DocComment> {
        match parser.take::<DocComment>() {
            Some(doc_comment) => Ok(doc_comment),
            None => Err(parser.emit_error(ParseErrorKind::ExpectedDocComment)),
        }
    }
}

impl<T: Parse> Parse for Annotated<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        // Parse the attribute list.
        let mut attribute_list = Vec::new();
        while let Some(DocComment {
            doc_style: DocStyle::Outer,
            ..
        }) = parser.peek::<DocComment>()
        {
            let doc_comment = parser.parse::<DocComment>()?;
            // TODO: Use a Literal instead of an Ident when Attribute args
            // start supporting them and remove `Ident::new_no_trim`.
            let value = Ident::new_no_trim(doc_comment.content_span.clone());
            attribute_list.push(AttributeDecl {
                hash_token: HashToken::new(doc_comment.span.clone()),
                attribute: SquareBrackets::new(
                    Attribute {
                        name: Ident::new_with_override("doc", doc_comment.span.clone()),
                        args: Some(Parens::new(
                            Punctuated::single(value),
                            doc_comment.content_span,
                        )),
                    },
                    doc_comment.span,
                ),
            });
        }
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
