use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStorage {
    pub storage_token: StorageToken,
    pub fields: Braces<Punctuated<StorageField, CommaToken>>,
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

impl Parse for StorageField {
    fn parse(parser: &mut Parser) -> ParseResult<StorageField> {
        let name = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        let eq_token = parser.parse()?;
        let initializer = parser.parse()?;
        Ok(StorageField {
            name,
            colon_token,
            ty,
            eq_token,
            initializer,
        })
    }
}

impl Parse for ItemStorage {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStorage> {
        let storage_token = parser.parse()?;
        let fields = parser.parse()?;
        Ok(ItemStorage {
            storage_token,
            fields,
        })
    }
}
