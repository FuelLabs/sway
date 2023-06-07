use crate::priv_prelude::{Peek, Peeker};
use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeArg, AttributeDecl, AttributeHashKind};
use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{EqToken, HashBangToken, HashToken, StorageToken, Token};
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
            let name = Ident::new_no_trim(doc_comment.content_span.clone());
            attribute_list.push(AttributeDecl {
                hash_kind: AttributeHashKind::Outer(HashToken::new(doc_comment.span.clone())),
                attribute: SquareBrackets::new(Punctuated::single(Attribute {
                    name: Ident::new_with_override(
                        DOC_COMMENT_ATTRIBUTE_NAME.to_owned(),
                        doc_comment.span.clone(),
                    ),
                    args: Some(Parens::new(Punctuated::single(AttributeArg {
                        name,
                        value: None,
                    }))),
                })),
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

impl Parse for AttributeArg {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let name = parser.parse()?;
        match parser.take::<EqToken>() {
            Some(_) => {
                let value = parser.parse()?;
                Ok(AttributeArg {
                    name,
                    value: Some(value),
                })
            }
            None => Ok(AttributeArg { name, value: None }),
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
    use crate::test_utils::parse;
    use insta::*;
    use sway_ast::ItemFn;

    #[test]
    fn parse_annotated_fn() {
        assert_ron_snapshot!(parse::<Annotated<ItemFn>>(r#"
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
                        final_value_opt: Some(AttributeArg(
                          name: Ident(
                            to_string: " This is a doc comment.",
                            span: (85, 108),
                          ),
                          value: None,
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
                        final_value_opt: Some(AttributeArg(
                          name: Ident(
                            to_string: "read",
                            span: (131, 135),
                          ),
                          value: None,
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

    #[test]
    fn parse_attribute() {
        assert_ron_snapshot!(parse::<Attribute>(r#"
            name(arg1, arg2 = "value", arg3)
        "#,), @r###"
        Attribute(
          name: Ident(
            to_string: "name",
            span: (13, 17),
          ),
          args: Some(Parens(
            inner: Punctuated(
              value_separator_pairs: [
                (AttributeArg(
                  name: Ident(
                    to_string: "arg1",
                    span: (18, 22),
                  ),
                  value: None,
                ), CommaToken(
                  span: (22, 23),
                )),
                (AttributeArg(
                  name: Ident(
                    to_string: "arg2",
                    span: (24, 28),
                  ),
                  value: Some(String(LitString(
                    span: (31, 38),
                    parsed: "value",
                  ))),
                ), CommaToken(
                  span: (38, 39),
                )),
              ],
              final_value_opt: Some(AttributeArg(
                name: Ident(
                  to_string: "arg3",
                  span: (40, 44),
                ),
                value: None,
              )),
            ),
            span: (17, 45),
          )),
        )
        "###);
    }
}
