use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStruct {
    pub visibility: Option<PubToken>,
    pub struct_token: StructToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub where_clause_opt: Option<WhereClause>,
    pub fields: Braces<Punctuated<TypeField, CommaToken>>,
}

impl ItemStruct {
    pub fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.struct_token.span(),
        };
        let end = self.fields.span();
        Span::join(start, end)
    }
}

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser) -> ParseResult<ItemStruct> {
        let visibility = parser.take();
        let struct_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<OpenAngleBracketToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let where_clause_opt = match parser.peek::<WhereToken>() {
            Some(..) => {
                let where_clause = parser.parse()?;
                Some(where_clause)
            }
            None => None,
        };
        let fields = parser.parse()?;
        Ok(ItemStruct {
            visibility,
            struct_token,
            name,
            generics,
            where_clause_opt,
            fields,
        })
    }
}
