use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemTypeAlias {
    pub visibility: Option<PubToken>,
    pub name: Ident,
    pub type_token: TypeToken,
    pub eq_token: EqToken,
    pub ty: Ty,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemTypeAlias {
    fn span(&self) -> Span {
        let start = self.type_token.span();
        let end = self.semicolon_token.span();
        Span::join(start, end)
    }
}
