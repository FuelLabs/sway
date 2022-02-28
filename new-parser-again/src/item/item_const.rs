use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemConst {
    pub visibility: Option<PubToken>,
    pub const_token: ConstToken,
    pub name: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
    pub eq_token: EqToken,
    pub expr: Expr,
    pub semicolon_token: SemicolonToken,
}

impl Parse for ItemConst {
    fn parse(parser: &mut Parser) -> ParseResult<ItemConst> {
        let visibility = parser.take();
        let const_token = parser.parse()?;
        let name = parser.parse()?;
        let ty_opt = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            },
            None => None,
        };
        let eq_token = parser.parse()?;
        let expr = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemConst { visibility, const_token, name, ty_opt, eq_token, expr, semicolon_token })
    }
}

