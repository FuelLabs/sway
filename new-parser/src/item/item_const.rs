use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemConst {
    pub const_token: ConstToken,
    pub name: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
    pub eq_token: EqToken,
    pub expr: Expr,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemConst {
    fn span(&self) -> Span {
        Span::join(self.const_token.span(), self.semicolon_token.span())
    }
}

pub fn item_const() -> impl Parser<Output = ItemConst> + Clone {
    const_token()
    .then_whitespace()
    .commit()
    .then(ident())
    .then_optional_whitespace()
    .then(
        colon_token()
        .then_optional_whitespace()
        .then(ty())
        .then_optional_whitespace()
        .optional()
    )
    .then(eq_token())
    .then_optional_whitespace()
    .then(expr())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|(((((const_token, name), ty_opt), eq_token), expr), semicolon_token)| {
        ItemConst { const_token, name, ty_opt, eq_token, expr, semicolon_token }
    })
}

