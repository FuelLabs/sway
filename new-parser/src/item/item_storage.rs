use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStorage {
    pub storage_token: StorageToken,
    pub fields: Braces<Punctuated<StorageField, CommaToken>>,
}

#[derive(Clone, Debug)]
pub struct StorageField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
    pub eq_token: EqToken,
    pub expr: Expr,
}

impl Spanned for ItemStorage {
    fn span(&self) -> Span {
        Span::join(self.storage_token.span(), self.fields.span())
    }
}

impl Spanned for StorageField {
    fn span(&self) -> Span {
        Span::join(self.name.span(), self.expr.span())
    }
}

pub fn item_storage() -> impl Parser<Output = ItemStorage> + Clone {
    storage_token()
    .then_optional_whitespace()
    .then(braces(
        punctuated(
            optional_leading_whitespace(storage_field()),
            optional_leading_whitespace(comma_token()),
        )
        .then_optional_whitespace()
    ))
    .map(|(storage_token, fields)| {
        ItemStorage { storage_token, fields }
    })
}

pub fn storage_field() -> impl Parser<Output = StorageField> + Clone {
    ident()
    .then_optional_whitespace()
    .then(colon_token())
    .then_optional_whitespace()
    .then(ty())
    .then_optional_whitespace()
    .then(eq_token())
    .then_optional_whitespace()
    .then(expr())
    .map(|((((name, colon_token), ty), eq_token), expr)| {
        StorageField { name, colon_token, ty, eq_token, expr }
    })
}

