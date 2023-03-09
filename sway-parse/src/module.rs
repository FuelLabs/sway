use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::{
    attribute::{Annotated, Attribute, AttributeHashKind},
    brackets::SquareBrackets,
    keywords::{HashBangToken, Token},
    token::{DocComment, DocStyle},
    AttributeDecl, Module, ModuleKind, Parens, Punctuated,
};
use sway_error::parser_error::ParseErrorKind;
use sway_types::{constants::DOC_COMMENT_ATTRIBUTE_NAME, Ident};

impl Parse for ModuleKind {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if let Some(script_token) = parser.take() {
            Ok(Self::Script { script_token })
        } else if let Some(contract_token) = parser.take() {
            Ok(Self::Contract { contract_token })
        } else if let Some(predicate_token) = parser.take() {
            Ok(Self::Predicate { predicate_token })
        } else if let Some(library_token) = parser.take() {
            Ok(Self::Library { library_token })
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedModuleKind))
        }
    }
}

impl ParseToEnd for Annotated<Module> {
    fn parse_to_end<'a, 'e>(mut parser: Parser<'a, '_>) -> ParseResult<(Self, ParserConsumed<'a>)> {
        // Parse the attribute list.
        let mut attribute_list = Vec::new();
        while let Some(DocComment { .. }) = parser.peek() {
            let doc_comment = parser.parse::<DocComment>()?;
            // TODO: Use a Literal instead of an Ident when Attribute args
            // start supporting them and remove `Ident::new_no_trim`.
            let value = Ident::new_no_trim(doc_comment.content_span.clone());
            match &doc_comment.doc_style {
                DocStyle::Inner => attribute_list.push(AttributeDecl {
                    hash_kind: AttributeHashKind::Inner(HashBangToken::new(
                        doc_comment.span.clone(),
                    )),
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
                }),
                DocStyle::Outer => {
                    parser.emit_error(ParseErrorKind::ExpectedModuleDocComment);
                }
            }
        }
        let (kind, semicolon_token) = parser.parse()?;

        let (items, consumed) = parser.parse_to_end()?;

        let module = Annotated {
            attribute_list,
            value: Module {
                kind,
                semicolon_token,
                items,
            },
        };
        Ok((module, consumed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::*;
    use std::sync::Arc;

    fn parse_annotated_module(input: &str) -> Annotated<Module> {
        let handler = <_>::default();
        let ts = crate::token::lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
        Parser::new(&handler, &ts)
            .parse_to_end()
            .map(|(m, _)| m)
            .unwrap_or_else(|_| panic!("Parse error: {:?}", handler.consume().0))
    }

    #[test]
    fn parse_noop_script_module() {
        assert_ron_snapshot!(parse_annotated_module(r#"
            script;
        
            fn main() {
                ()
            }
        "#,), @r###"
        Annotated(
          attribute_list: [],
          value: Module(
            kind: Script(
              script_token: ScriptToken(
                span: (13, 19),
              ),
            ),
            semicolon_token: SemicolonToken(
              span: (19, 20),
            ),
            items: [
              Annotated(
                attribute_list: [],
                value: Fn(ItemFn(
                  fn_signature: FnSignature(
                    visibility: None,
                    fn_token: FnToken(
                      span: (42, 44),
                    ),
                    name: Ident(
                      to_string: "main",
                      span: (45, 49),
                    ),
                    generics: None,
                    arguments: Parens(
                      inner: Static(Punctuated(
                        value_separator_pairs: [],
                        final_value_opt: None,
                      )),
                      span: (49, 51),
                    ),
                    return_type_opt: None,
                    where_clause_opt: None,
                  ),
                  body: Braces(
                    inner: CodeBlockContents(
                      statements: [],
                      final_expr_opt: Some(Tuple(Parens(
                        inner: Nil,
                        span: (70, 72),
                      ))),
                    ),
                    span: (52, 86),
                  ),
                )),
              ),
            ],
          ),
        )
        "###);
    }
}
