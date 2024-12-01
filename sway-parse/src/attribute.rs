use crate::priv_prelude::{Peek, Peeker};
use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeArg, AttributeDecl, AttributeHashKind};
use sway_ast::brackets::{Parens, SquareBrackets};
use sway_ast::keywords::{EqToken, HashBangToken, HashToken, StorageToken, Token};
use sway_ast::literal::LitBool;
use sway_ast::punctuated::Punctuated;
use sway_ast::token::{DocComment, DocStyle};
use sway_ast::Literal;
use sway_error::parser_error::ParseErrorKind;
use sway_types::constants::DOC_COMMENT_ATTRIBUTE_NAME;
use sway_types::{Ident, Spanned};

impl Peek for DocComment {
    fn peek(peeker: Peeker<'_>) -> Option<DocComment> {
        peeker.peek_doc_comment().ok().cloned()
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
                attribute: SquareBrackets::new(
                    Punctuated::single(Attribute {
                        name: Ident::new_with_override(
                            DOC_COMMENT_ATTRIBUTE_NAME.to_string(),
                            doc_comment.span.clone(),
                        ),
                        args: Some(Parens::new(
                            Punctuated::single(AttributeArg { name, value: None }),
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

        if parser.check_empty().is_some() {
            let error = parser.emit_error(ParseErrorKind::ExpectedAnItemAfterDocComment);
            Err(error)
        } else {
            // Parse the `T` value.
            let value = match parser.parse_with_recovery() {
                Ok(value) => value,
                Err(r) => {
                    let (spans, error) =
                        r.recover_at_next_line_with_fallback_error(ParseErrorKind::InvalidItem);
                    if let Some(error) = T::error(spans, error) {
                        error
                    } else {
                        Err(error)?
                    }
                }
            };

            Ok(Annotated {
                attribute_list,
                value,
            })
        }
    }

    fn error(
        spans: Box<[sway_types::Span]>,
        error: sway_error::handler::ErrorEmitted,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        T::error(spans, error).map(|value| Annotated {
            attribute_list: vec![],
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
                let value = match parser.take::<Ident>() {
                    Some(ident) if ident.as_str() == "true" => Literal::Bool(LitBool {
                        span: ident.span(),
                        kind: sway_ast::literal::LitBoolType::True,
                    }),
                    Some(ident) if ident.as_str() == "false" => Literal::Bool(LitBool {
                        span: ident.span(),
                        kind: sway_ast::literal::LitBoolType::False,
                    }),
                    _ => parser.parse()?,
                };

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
        "#,), @r#"
        Annotated(
          attribute_list: [
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 82,
                  end: 108,
                  source_id: None,
                ),
              )),
              attribute: SquareBrackets(
                inner: Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: Some(Attribute(
                    name: BaseIdent(
                      name_override_opt: Some("doc-comment"),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 82,
                        end: 108,
                        source_id: None,
                      ),
                      is_raw_ident: false,
                    ),
                    args: Some(Parens(
                      inner: Punctuated(
                        value_separator_pairs: [],
                        final_value_opt: Some(AttributeArg(
                          name: BaseIdent(
                            name_override_opt: None,
                            span: Span(
                              src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                              start: 85,
                              end: 108,
                              source_id: None,
                            ),
                            is_raw_ident: false,
                          ),
                          value: None,
                        )),
                      ),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 85,
                        end: 108,
                        source_id: None,
                      ),
                    )),
                  )),
                ),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 82,
                  end: 108,
                  source_id: None,
                ),
              ),
            ),
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 121,
                  end: 122,
                  source_id: None,
                ),
              )),
              attribute: SquareBrackets(
                inner: Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: Some(Attribute(
                    name: BaseIdent(
                      name_override_opt: None,
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 123,
                        end: 130,
                        source_id: None,
                      ),
                      is_raw_ident: false,
                    ),
                    args: Some(Parens(
                      inner: Punctuated(
                        value_separator_pairs: [],
                        final_value_opt: Some(AttributeArg(
                          name: BaseIdent(
                            name_override_opt: None,
                            span: Span(
                              src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                              start: 131,
                              end: 135,
                              source_id: None,
                            ),
                            is_raw_ident: false,
                          ),
                          value: None,
                        )),
                      ),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 130,
                        end: 136,
                        source_id: None,
                      ),
                    )),
                  )),
                ),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 122,
                  end: 137,
                  source_id: None,
                ),
              ),
            ),
          ],
          value: ItemFn(
            fn_signature: FnSignature(
              visibility: None,
              fn_token: FnToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 150,
                  end: 152,
                  source_id: None,
                ),
              ),
              name: BaseIdent(
                name_override_opt: None,
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 153,
                  end: 157,
                  source_id: None,
                ),
                is_raw_ident: false,
              ),
              generics: None,
              arguments: Parens(
                inner: Static(Punctuated(
                  value_separator_pairs: [],
                  final_value_opt: None,
                )),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 157,
                  end: 159,
                  source_id: None,
                ),
              ),
              return_type_opt: None,
              where_clause_opt: None,
            ),
            body: Braces(
              inner: CodeBlockContents(
                statements: [],
                final_expr_opt: Some(Tuple(Parens(
                  inner: Nil,
                  span: Span(
                    src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                    start: 178,
                    end: 180,
                    source_id: None,
                  ),
                ))),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 161,
                  end: 193,
                  source_id: None,
                ),
              ),
              span: Span(
                src: "\n            // I will be ignored.\n            //! I will be ignored.\n            /// This is a doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                start: 160,
                end: 194,
                source_id: None,
              ),
            ),
          ),
        )
        "#);
    }

    #[test]
    fn parse_attribute() {
        assert_ron_snapshot!(parse::<Attribute>(r#"
            name(arg1, arg2 = "value", arg3)
        "#,), @r#"
        Attribute(
          name: BaseIdent(
            name_override_opt: None,
            span: Span(
              src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
              start: 13,
              end: 17,
              source_id: None,
            ),
            is_raw_ident: false,
          ),
          args: Some(Parens(
            inner: Punctuated(
              value_separator_pairs: [
                (AttributeArg(
                  name: BaseIdent(
                    name_override_opt: None,
                    span: Span(
                      src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                      start: 18,
                      end: 22,
                      source_id: None,
                    ),
                    is_raw_ident: false,
                  ),
                  value: None,
                ), CommaToken(
                  span: Span(
                    src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                    start: 22,
                    end: 23,
                    source_id: None,
                  ),
                )),
                (AttributeArg(
                  name: BaseIdent(
                    name_override_opt: None,
                    span: Span(
                      src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                      start: 24,
                      end: 28,
                      source_id: None,
                    ),
                    is_raw_ident: false,
                  ),
                  value: Some(String(LitString(
                    span: Span(
                      src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                      start: 31,
                      end: 38,
                      source_id: None,
                    ),
                    parsed: "value",
                  ))),
                ), CommaToken(
                  span: Span(
                    src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                    start: 38,
                    end: 39,
                    source_id: None,
                  ),
                )),
              ],
              final_value_opt: Some(AttributeArg(
                name: BaseIdent(
                  name_override_opt: None,
                  span: Span(
                    src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
                    start: 40,
                    end: 44,
                    source_id: None,
                  ),
                  is_raw_ident: false,
                ),
                value: None,
              )),
            ),
            span: Span(
              src: "\n            name(arg1, arg2 = \"value\", arg3)\n        ",
              start: 17,
              end: 45,
              source_id: None,
            ),
          )),
        )
        "#);
    }
}
