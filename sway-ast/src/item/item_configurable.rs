use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemConfigurable {
    pub configurable_token: ConfigurableToken,
    pub fields: Braces<Punctuated<Annotated<ConfigurableField>, CommaToken>>,
}

impl Spanned for ItemConfigurable {
    fn span(&self) -> Span {
        Span::join(self.configurable_token.span(), self.fields.span())
    }
}

#[derive(Clone, Debug)]
pub struct ConfigurableField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
    pub eq_token: EqToken,
    pub initializer: Expr,
}
