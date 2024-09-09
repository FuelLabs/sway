use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct ItemStorage {
    pub storage_token: StorageToken,
    pub entries: Braces<Punctuated<Annotated<StorageEntry>, CommaToken>>,
}

impl Spanned for ItemStorage {
    fn span(&self) -> Span {
        Span::join(self.storage_token.span(), &self.entries.span())
    }
}

#[derive(Clone, Debug, Serialize)]

pub struct StorageEntry {
    pub name: Ident,
    pub namespace: Option<Braces<Punctuated<Annotated<Box<StorageEntry>>, CommaToken>>>,
    pub field: Option<StorageField>,
}

impl Spanned for StorageEntry {
    fn span(&self) -> Span {
        if let Some(namespace) = &self.namespace {
            Span::join(self.name.span(), &namespace.span())
        } else if let Some(field) = &self.field {
            Span::join(self.name.span(), &field.span())
        } else {
            self.name.span()
        }
    }
}

#[derive(Clone, Debug, Serialize)]

pub struct StorageField {
    pub name: Ident,
    pub in_token: Option<InToken>,
    pub key_expr: Option<Expr>,
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
