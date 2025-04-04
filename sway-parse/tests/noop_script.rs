use insta::*;

use crate::common::parse_file;

mod common;

#[test]
fn noop_script_file() {
    assert_ron_snapshot!(parse_file(r#"
      script;
      
      fn main() {
        ()
      }
    "#,), @r#"
    Some(Annotated(
      attributes: [],
      value: Module(
        kind: Script(
          script_token: ScriptToken(
            span: Span(
              src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
              start: 7,
              end: 13,
              source_id: None,
            ),
          ),
        ),
        semicolon_token: SemicolonToken(
          span: Span(
            src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
            start: 13,
            end: 14,
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
                    src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                    start: 28,
                    end: 30,
                    source_id: None,
                  ),
                ),
                name: BaseIdent(
                  name_override_opt: None,
                  span: Span(
                    src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                    start: 31,
                    end: 35,
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
                    src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                    start: 35,
                    end: 37,
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
                      src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                      start: 48,
                      end: 50,
                      source_id: None,
                    ),
                  ))),
                  span: Span(
                    src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                    start: 39,
                    end: 57,
                    source_id: None,
                  ),
                ),
                span: Span(
                  src: "\n      script;\n      \n      fn main() {\n        ()\n      }\n    ",
                  start: 38,
                  end: 58,
                  source_id: None,
                ),
              ),
            )),
          ),
        ],
      ),
    ))
    "#);
}
