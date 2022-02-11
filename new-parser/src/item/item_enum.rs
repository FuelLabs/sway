use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemEnum {
    pub visibility: Option<PubToken>,
    pub enum_token: EnumToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub type_fields: Braces<TypeFields>,
}

impl Spanned for ItemEnum {
    fn span(&self) -> Span {
        match &self.visibility {
            Some(pub_token) => Span::join(pub_token.span(), self.type_fields.span()),
            None => Span::join(self.enum_token.span(), self.type_fields.span()),
        }
    }
}

pub fn item_enum() -> impl Parser<Output = ItemEnum> + Clone {
    pub_token()
    .then_whitespace()
    .optional()
    .then(enum_token())
    .then_whitespace()
    .commit()
    .then(ident())
    .then_optional_whitespace()
    .then(
        generic_params()
        .then_optional_whitespace()
        .optional()
    )
    .then(braces(padded(type_fields())))
    .map(|((((visibility, enum_token), name), generics), type_fields)| {
        ItemEnum { visibility, enum_token, name, generics, type_fields }
    })
}

