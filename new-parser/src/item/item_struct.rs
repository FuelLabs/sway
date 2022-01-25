use crate::priv_prelude::*;

pub struct ItemStruct {
    pub struct_token: StructToken,
    pub name: Ident,
    pub type_fields: Braces<TypeFields>,
}

impl Spanned for ItemStruct {
    fn span(&self) -> Span {
        Span::join(self.struct_token.span(), self.type_fields.span())
    }
}

pub fn item_struct() -> impl Parser<Output = ItemStruct> + Clone {
    struct_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(braces(padded(type_fields())))
    .map(|((struct_token, name), type_fields)| ItemStruct { struct_token, name, type_fields })
}

