use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Submodule {
    pub mod_token: ModToken,
    pub name: Ident,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for Submodule {
    fn span(&self) -> Span {
        Span::join(self.mod_token.span(), self.semicolon_token.span())
    }
}
