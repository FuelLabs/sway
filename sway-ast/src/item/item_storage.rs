use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct ItemStorage {
    pub storage_token: StorageToken,
    pub fields: Braces<Punctuated<Annotated<StorageField>, CommaToken>>,
}

impl Spanned for ItemStorage {
    fn span(&self) -> Span {
        Span::join(self.storage_token.span(), self.fields.span())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StorageField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
    pub eq_token: EqToken,
    pub initializer: Expr,
}

impl Spanned for StorageField {
    fn span(&self) -> Span {
        Span::join_all([
            self.name.span(),
            self.colon_token.span(),
            self.ty.span(),
            self.eq_token.span(),
            self.initializer.span(),
        ])
    }
}
