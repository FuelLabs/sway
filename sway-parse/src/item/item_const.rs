use crate::{Parse, ParseResult, Parser};

use sway_ast::{keywords::ColonToken, ItemConst, Ty};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ItemConst {
    fn parse(parser: &mut Parser) -> ParseResult<ItemConst> {
        let visibility = parser.take();
        let const_token = parser.parse()?;
        let name = parser.parse()?;
        let ty_opt: Option<(ColonToken, Ty)> = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            }
            None => None,
        };
        let (_, ty) = ty_opt
            .ok_or_else(|| parser.emit_error(ParseErrorKind::ExpectedTypeAnnotationForConstants))?;
        let eq_token_opt = parser.take();
        let expr_opt = match &eq_token_opt {
            Some(_eq) => Some(parser.parse()?),
            None => None,
        };
        // Use the default here since the braces parsing is expecting
        // a semicolon, that allows us to re-use the same parsing code
        // between associated consts and module-level consts.
        let semicolon_token = parser.peek().unwrap_or_default();
        Ok(ItemConst {
            visibility,
            const_token,
            name,
            ty,
            eq_token_opt,
            expr_opt,
            semicolon_token,
        })
    }
}
