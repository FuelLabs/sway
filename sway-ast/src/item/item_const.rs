use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct ItemConst {
    pub pub_token: Option<PubToken>,
    pub const_token: ConstToken,
    pub name: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
    pub eq_token_opt: Option<EqToken>,
    pub expr_opt: Option<Expr>,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemConst {
    fn span(&self) -> Span {
        let start = match &self.pub_token {
            Some(pub_token) => pub_token.span(),
            None => self.const_token.span(),
        };
        let end = match &self.expr_opt {
            Some(expr) => expr.span(),
            None => match &self.ty_opt {
                Some((_colon, ty)) => ty.span(),
                None => self.name.span(),
            },
        };
        Span::join(start, &end)
    }
}
