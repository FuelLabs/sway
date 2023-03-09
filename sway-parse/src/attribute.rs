use crate::priv_prelude::{Peek, Peeker};
use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeDecl, AttributeHashKind};
use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{HashBangToken, HashToken, StorageToken, Token};
use sway_ast::punctuated::Punctuated;
use sway_ast::token::{DocComment, DocStyle};
use sway_error::parser_error::ParseErrorKind;
use sway_types::constants::DOC_COMMENT_ATTRIBUTE_NAME;
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
        }) = parser.peek()
        {
            let doc_comment = parser.parse::<DocComment>()?;
            // TODO: Use a Literal instead of an Ident when Attribute args
            // start supporting them and remove `Ident::new_no_trim`.
            let value = Ident::new_no_trim(doc_comment.content_span.clone());
            attribute_list.push(AttributeDecl {
                hash_kind: AttributeHashKind::Outer(HashToken::new(doc_comment.span.clone())),
                attribute: SquareBrackets::new(
                    Punctuated::single(Attribute {
                        name: Ident::new_with_override(
                            DOC_COMMENT_ATTRIBUTE_NAME.to_string(),
                            doc_comment.span.clone(),
                        ),
                        args: Some(Parens::new(
                            Punctuated::single(value),
                            doc_comment.content_span,
                        )),
                    }),
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
            hash_kind: parser.parse()?,
            attribute: parser.parse()?,
        })
    }
}

impl Parse for AttributeHashKind {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        match parser.take::<HashBangToken>() {
            Some(hash_bang_token) => Ok(AttributeHashKind::Inner(hash_bang_token)),
            None => match parser.take::<HashToken>() {
                Some(hash_token) => Ok(AttributeHashKind::Outer(hash_token)),
                None => Err(parser.emit_error(ParseErrorKind::ExpectedAnAttribute)),
            },
        }
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
    fn parse_to_end<'a, 'e>(mut parser: Parser<'a, '_>) -> ParseResult<(Self, ParserConsumed<'a>)> {
        let attrib = parser.parse()?;
        match parser.check_empty() {
            Some(consumed) => Ok((attrib, consumed)),
            None => Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAttribute)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::*;
    use std::sync::Arc;
    use sway_ast::ItemFn;

    fn parse_annotated<T>(input: &str) -> Annotated<T>
    where
        T: Parse,
    {
        let handler = <_>::default();
        let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
        Parser::new(&handler, &ts)
            .parse()
            .unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
    }

    #[test]
    fn parse_annotated_fn() {
        assert_ron_snapshot!(parse_annotated::<ItemFn>(r#"
            // I will be ignored.
            //! I will be ignored.
            /// This is a doc comment.
            #[storage(read)]
            fn main() {
                ()
            }
        "#,), @r###"
        Annotated(
          attribute_list: [
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: (82, 108),
              )),
              attribute: SquareBrackets(
                inner: Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: Some(Attribute(
                    name: Ident(
                      to_string: "doc-comment",
                      span: (82, 108),
                    ),
                    args: Some(Parens(
                      inner: Punctuated(
                        value_separator_pairs: [],
                        final_value_opt: Some(Ident(
                          to_string: " This is a doc comment.",
                          span: (85, 108),
                        )),
                      ),
                      span: (85, 108),
                    )),
                  )),
                ),
                span: (82, 108),
              ),
            ),
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: (121, 122),
              )),
              attribute: SquareBrackets(
                inner: Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: Some(Attribute(
                    name: Ident(
                      to_string: "storage",
                      span: (123, 130),
                    ),
                    args: Some(Parens(
                      inner: Punctuated(
                        value_separator_pairs: [],
                        final_value_opt: Some(Ident(
                          to_string: "read",
                          span: (131, 135),
                        )),
                      ),
                      span: (130, 136),
                    )),
                  )),
                ),
                span: (122, 137),
              ),
            ),
          ],
          value: ItemFn(
            fn_signature: FnSignature(
              visibility: None,
              fn_token: FnToken(
                span: (150, 152),
              ),
              name: Ident(
                to_string: "main",
                span: (153, 157),
              ),
              generics: None,
              arguments: Parens(
                inner: Static(Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: None,
                )),
                span: (157, 159),
              ),
              return_type_opt: None,
              where_clause_opt: None,
            ),
            body: Braces(
              inner: CodeBlockContents(
                statements: [],
                final_expr_opt: Some(Tuple(Parens(
                  inner: Nil,
                  span: (178, 180),
                ))),
              ),
              span: (160, 194),
            ),
          ),
        )
        "###);
    }
}
