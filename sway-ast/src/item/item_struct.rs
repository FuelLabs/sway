use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct ItemStruct {
    pub visibility: Option<PubToken>,
    pub struct_token: StructToken,
    pub name: Ident,
    pub generic_params_opt: Option<GenericParams>,
    pub where_clause_opt: Option<WhereClause>,
    pub fields: Braces<Punctuated<Annotated<TypeField>, CommaToken>>,
}

impl Spanned for ItemStruct {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.struct_token.span(),
        };
        let end = self.fields.span();
        Span::join(start, &end)
    }
}
