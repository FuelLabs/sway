use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemConst {
    pub visibility:      Option<PubToken>,
    pub const_token:     ConstToken,
    pub name:            Ident,
    pub ty_opt:          Option<(ColonToken, Ty)>,
    pub eq_token:        EqToken,
    pub expr:            Expr,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemConst {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.const_token.span(),
        };
        let end = self.semicolon_token.span();
        Span::join(start, end)
    }
}
