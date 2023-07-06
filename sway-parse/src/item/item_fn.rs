use crate::{Parse, ParseResult, Parser};

use sway_ast::ItemFn;

impl Parse for ItemFn {
    fn parse(parser: &mut Parser) -> ParseResult<ItemFn> {
        let fn_signature = parser.parse()?;
        let body = parser.parse()?;
        Ok(ItemFn { fn_signature, body })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::parse;
    use sway_ast::ItemFn;

    #[test]
    fn test_parse_fn() {
        let input = parse::<ItemFn>(
            r#"
        fn f() -> bool {
            false
        }
        "#,
        );
        dbg!(input);
        assert!(true)
    }
}
