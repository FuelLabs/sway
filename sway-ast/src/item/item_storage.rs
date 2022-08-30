use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStorage {
    pub storage_token: StorageToken,
    pub fields: Braces<Punctuated<Annotated<StorageField>, CommaToken>>,
}

impl Spanned for ItemStorage {
    fn span(&self) -> Span {
        Span::join(self.storage_token.span(), self.fields.span())
    }
}

#[derive(Clone, Debug)]
pub struct StorageField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
    pub eq_token: EqToken,
    pub initializer: Expr,
}
