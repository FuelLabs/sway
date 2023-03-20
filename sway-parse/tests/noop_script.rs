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
    "#,), @r###"
    Some(Annotated(
      attribute_list: [],
      value: Module(
        kind: Script(
          script_token: ScriptToken(
            span: (7, 13),
          ),
        ),
        semicolon_token: SemicolonToken(
          span: (13, 14),
        ),
        items: [
          Annotated(
            attribute_list: [],
            value: Fn(ItemFn(
              fn_signature: FnSignature(
                visibility: None,
                fn_token: FnToken(
                  span: (28, 30),
                ),
                name: Ident(
                  to_string: "main",
                  span: (31, 35),
                ),
                generics: None,
                arguments: Parens(
                  inner: Static(Punctuated(
                    value_separator_pairs: [],
                    final_value_opt: None,
                  )),
                  span: (35, 37),
                ),
                return_type_opt: None,
                where_clause_opt: None,
              ),
              body: Braces(
                inner: CodeBlockContents(
                  statements: [],
                  final_expr_opt: Some(Tuple(Parens(
                    inner: Nil,
                    span: (48, 50),
                  ))),
                ),
                span: (38, 58),
              ),
            )),
          ),
        ],
      ),
    ))
    "###);
}
