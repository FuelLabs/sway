use crate::priv_prelude::{Peek, Peeker};
use crate::{Parse, ParseBracket, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::attribute::{Annotated, Attribute, AttributeArg, AttributeDecl, AttributeHashKind};
use sway_ast::brackets::Parens;
use sway_ast::keywords::{EqToken, HashBangToken, HashToken, StorageToken};
use sway_ast::literal::LitBool;
use sway_ast::token::{DocComment, DocStyle};
use sway_ast::Literal;
use sway_error::parser_error::ParseErrorKind;
use sway_types::{Ident, Span, Spanned};

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

impl Parse for Vec<AttributeDecl> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let mut attributes = Vec::new();

        loop {
            if let Some(DocComment { .. }) = parser.peek() {
                let doc_comment = parser.parse::<DocComment>()?;
                let doc_comment_attr_decl = match doc_comment.doc_style {
                    DocStyle::Outer => AttributeDecl::new_outer_doc_comment(
                        doc_comment.span,
                        doc_comment.content_span,
                    ),
                    DocStyle::Inner => AttributeDecl::new_inner_doc_comment(
                        doc_comment.span,
                        doc_comment.content_span,
                    ),
                };
                attributes.push(doc_comment_attr_decl);
                continue;
            }

            // This will parse both `#` and `#!` attributes.
            if let Some(attr_decl) = parser.guarded_parse::<HashToken, _>()? {
                attributes.push(attr_decl);
                continue;
            }

            break;
        }

        Ok(attributes)
    }
}

impl<T: Parse> Parse for Annotated<T> {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let attributes = parser.parse::<Vec<AttributeDecl>>()?;

        if parser.check_empty().is_some() {
            // Provide a dedicated error message for the case when we have
            // inner doc comments (`//!`) at the end of the module (because
            // there are no items after the comments).
            let error = if attributes
                .iter()
                .all(|attr| attr.is_inner() && attr.is_doc_comment())
            {
                // Show the error on the complete doc comment.
                let first_doc_line = attributes.first().expect(
                    "parsing `Annotated` guarantees that `attributes` have at least one element",
                );
                let last_doc_line = attributes.last().expect(
                    "parsing `Annotated` guarantees that `attributes` have at least one element",
                );
                let span = Span::join(first_doc_line.span(), &last_doc_line.span().start_span());
                parser.emit_error_with_span(
                    ParseErrorKind::ExpectedInnerDocCommentAtTheTopOfFile,
                    span,
                )
            } else {
                let is_only_documented = attributes.iter().all(|attr| attr.is_doc_comment());
                parser.emit_error(ParseErrorKind::ExpectedAnAnnotatedElement { is_only_documented })
            };
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

            Ok(Annotated { attributes, value })
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
            attributes: vec![],
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
            //! This is a misplaced inner doc comment.
            /// This is an outer doc comment.
            #[storage(read)]
            fn main() {
                ()
            }
        "#,), @r#"
        Annotated(
          attributes: [
            AttributeDecl(
              hash_kind: Inner(HashBangToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 47,
                  end: 89,
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
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 47,
                        end: 89,
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
                              src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                              start: 50,
                              end: 89,
                              source_id: None,
                            ),
                            is_raw_ident: false,
                          ),
                          value: None,
                        )),
                      ),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 50,
                        end: 89,
                        source_id: None,
                      ),
                    )),
                  )),
                ),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 47,
                  end: 89,
                  source_id: None,
                ),
              ),
            ),
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 102,
                  end: 135,
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
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 102,
                        end: 135,
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
                              src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                              start: 105,
                              end: 135,
                              source_id: None,
                            ),
                            is_raw_ident: false,
                          ),
                          value: None,
                        )),
                      ),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 105,
                        end: 135,
                        source_id: None,
                      ),
                    )),
                  )),
                ),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 102,
                  end: 135,
                  source_id: None,
                ),
              ),
            ),
            AttributeDecl(
              hash_kind: Outer(HashToken(
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 148,
                  end: 149,
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
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 150,
                        end: 157,
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
                              src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                              start: 158,
                              end: 162,
                              source_id: None,
                            ),
                            is_raw_ident: false,
                          ),
                          value: None,
                        )),
                      ),
                      span: Span(
                        src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                        start: 157,
                        end: 163,
                        source_id: None,
                      ),
                    )),
                  )),
                ),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 149,
                  end: 164,
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
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 177,
                  end: 179,
                  source_id: None,
                ),
              ),
              name: BaseIdent(
                name_override_opt: None,
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 180,
                  end: 184,
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
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 184,
                  end: 186,
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
                    src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                    start: 205,
                    end: 207,
                    source_id: None,
                  ),
                ))),
                span: Span(
                  src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                  start: 188,
                  end: 220,
                  source_id: None,
                ),
              ),
              span: Span(
                src: "\n            // I will be ignored.\n            //! This is a misplaced inner doc comment.\n            /// This is an outer doc comment.\n            #[storage(read)]\n            fn main() {\n                ()\n            }\n        ",
                start: 187,
                end: 221,
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

    #[test]
    fn parse_fuzz_attribute() {
        assert_ron_snapshot!(parse::<Attribute>(r#"
            fuzz
        "#,), @r#"
        Attribute(
          name: BaseIdent(
            name_override_opt: None,
            span: Span(
              src: "\n            fuzz\n        ",
              start: 13,
              end: 17,
              source_id: None,
            ),
            is_raw_ident: false,
          ),
          args: None,
        )
        "#);
    }

    #[test]
    fn parse_fuzz_param_attribute() {
        assert_ron_snapshot!(parse::<Attribute>(r#"
            fuzz_param(name = "input1", iteration = 100)
        "#,), @r#"
        Attribute(
          name: BaseIdent(
            name_override_opt: None,
            span: Span(
              src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
              start: 13,
              end: 23,
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
                      src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
                      start: 24,
                      end: 28,
                      source_id: None,
                    ),
                    is_raw_ident: false,
                  ),
                  value: Some(String(LitString(
                    span: Span(
                      src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
                      start: 31,
                      end: 39,
                      source_id: None,
                    ),
                    parsed: "input1",
                  ))),
                ), CommaToken(
                  span: Span(
                    src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
                    start: 39,
                    end: 40,
                    source_id: None,
                  ),
                )),
              ],
              final_value_opt: Some(AttributeArg(
                name: BaseIdent(
                  name_override_opt: None,
                  span: Span(
                    src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
                    start: 41,
                    end: 50,
                    source_id: None,
                  ),
                  is_raw_ident: false,
                ),
                value: Some(Int(LitInt(
                  span: Span(
                    src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
                    start: 53,
                    end: 56,
                    source_id: None,
                  ),
                  parsed: [
                    100,
                  ],
                  ty_opt: None,
                  is_generated_b256: false,
                ))),
              )),
            ),
            span: Span(
              src: "\n            fuzz_param(name = \"input1\", iteration = 100)\n        ",
              start: 23,
              end: 57,
              source_id: None,
            ),
          )),
        )
        "#);
    }

    #[test]
    fn parse_fuzz_param_min_max_attribute() {
        assert_ron_snapshot!(parse::<Attribute>(r#"
            fuzz_param(name = "input2", min_val = 0, max_val = 255)
        "#,), @r#"
        Attribute(
          name: BaseIdent(
            name_override_opt: None,
            span: Span(
              src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
              start: 13,
              end: 23,
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
                      src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                      start: 24,
                      end: 28,
                      source_id: None,
                    ),
                    is_raw_ident: false,
                  ),
                  value: Some(String(LitString(
                    span: Span(
                      src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                      start: 31,
                      end: 39,
                      source_id: None,
                    ),
                    parsed: "input2",
                  ))),
                ), CommaToken(
                  span: Span(
                    src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                    start: 39,
                    end: 40,
                    source_id: None,
                  ),
                )),
                (AttributeArg(
                  name: BaseIdent(
                    name_override_opt: None,
                    span: Span(
                      src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                      start: 41,
                      end: 48,
                      source_id: None,
                    ),
                    is_raw_ident: false,
                  ),
                  value: Some(Int(LitInt(
                    span: Span(
                      src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                      start: 51,
                      end: 52,
                      source_id: None,
                    ),
                    parsed: [],
                    ty_opt: None,
                    is_generated_b256: false,
                  ))),
                ), CommaToken(
                  span: Span(
                    src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                    start: 52,
                    end: 53,
                    source_id: None,
                  ),
                )),
              ],
              final_value_opt: Some(AttributeArg(
                name: BaseIdent(
                  name_override_opt: None,
                  span: Span(
                    src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                    start: 54,
                    end: 61,
                    source_id: None,
                  ),
                  is_raw_ident: false,
                ),
                value: Some(Int(LitInt(
                  span: Span(
                    src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
                    start: 64,
                    end: 67,
                    source_id: None,
                  ),
                  parsed: [
                    255,
                  ],
                  ty_opt: None,
                  is_generated_b256: false,
                ))),
              )),
            ),
            span: Span(
              src: "\n            fuzz_param(name = \"input2\", min_val = 0, max_val = 255)\n        ",
              start: 23,
              end: 68,
              source_id: None,
            ),
          )),
        )
        "#);
    }
}
