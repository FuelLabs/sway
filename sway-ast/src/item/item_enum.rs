use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemEnum {
    pub visibility: Option<PubToken>,
    pub enum_token: EnumToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub where_clause_opt: Option<WhereClause>,
    pub fields: Braces<Punctuated<TypeField, CommaToken>>,
}

impl Spanned for ItemEnum {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.enum_token.span(),
        };
        let end = self.fields.span();
        Span::join(start, end)
    }
}
