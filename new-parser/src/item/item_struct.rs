use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemStruct {
    pub visibility: Option<PubToken>,
    pub struct_token: StructToken,
    pub name: Ident,
    pub generics: Option<Generics>,
    pub type_fields: Braces<TypeFields>,
}

impl Spanned for ItemStruct {
    fn span(&self) -> Span {
        match &self.visibility {
            Some(pub_token) => Span::join(pub_token.span(), self.type_fields.span()),
            None => Span::join(self.struct_token.span(), self.type_fields.span()),
        }
    }
}

pub fn item_struct() -> impl Parser<Output = ItemStruct> + Clone {
    pub_token()
    .then_whitespace()
    .optional()
    .then(struct_token())
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(
        generics()
        .then_optional_whitespace()
        .optional()
    )
    .then(braces(padded(type_fields())))
    .map(|((((visibility, struct_token), name), generics), type_fields)| {
        ItemStruct { visibility, struct_token, name, generics, type_fields }
    })
}

