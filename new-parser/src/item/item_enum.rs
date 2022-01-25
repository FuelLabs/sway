use crate::priv_prelude::*;

pub struct ItemEnum {
    pub enum_token: EnumToken,
    pub name: Ident,
    pub type_fields: Braces<TypeFields>,
}

impl Spanned for ItemEnum {
    fn span(&self) -> Span {
        Span::join(self.enum_token.span(), self.type_fields.span())
    }
}

pub fn item_enum() -> impl Parser<Output = ItemEnum> + Clone {
    enum_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(braces(padded(type_fields())))
    .map(|((enum_token, name), type_fields)| ItemEnum { enum_token, name, type_fields })
}

