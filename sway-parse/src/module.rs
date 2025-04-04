use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::{attribute::Annotated, Module, ModuleKind};
use sway_error::parser_error::ParseErrorKind;

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
        let attributes = parser.parse()?;
        let (kind, semicolon_token) = parser.parse()?;
        let (items, consumed) = parser.parse_to_end()?;
        let module = Annotated {
            attributes,
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
    use crate::test_utils::parse_to_end;
    use insta::*;

    #[test]
    fn parse_noop_script_module() {
        assert_ron_snapshot!(parse_to_end::<Annotated<Module>>(r#"
            script;
        
            fn main() {
                ()
            }
        "#,), @r#"
        Annotated(
          attributes: [],
          value: Module(
            kind: Script(
              script_token: ScriptToken(
                span: Span(
                  src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                  start: 13,
                  end: 19,
                  source_id: None,
                ),
              ),
            ),
            semicolon_token: SemicolonToken(
              span: Span(
                src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                start: 19,
                end: 20,
                source_id: None,
              ),
            ),
            items: [
              Annotated(
                attributes: [],
                value: Fn(ItemFn(
                  fn_signature: FnSignature(
                    visibility: None,
                    fn_token: FnToken(
                      span: Span(
                        src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                        start: 42,
                        end: 44,
                        source_id: None,
                      ),
                    ),
                    name: BaseIdent(
                      name_override_opt: None,
                      span: Span(
                        src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                        start: 45,
                        end: 49,
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
                        src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                        start: 49,
                        end: 51,
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
                          src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                          start: 70,
                          end: 72,
                          source_id: None,
                        ),
                      ))),
                      span: Span(
                        src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                        start: 53,
                        end: 85,
                        source_id: None,
                      ),
                    ),
                    span: Span(
                      src: "\n            script;\n        \n            fn main() {\n                ()\n            }\n        ",
                      start: 52,
                      end: 86,
                      source_id: None,
                    ),
                  ),
                )),
              ),
            ],
          ),
        )
        "#);
    }
}
