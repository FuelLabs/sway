use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct TypeFields {
    pub fields: Punctuated<TypeField, CommaToken>,
}

#[derive(Clone, Debug)]
pub struct TypeField {
    pub field_name: Ident,
    pub colon_token: ColonToken,
    pub field_ty: Ty,
}

impl Spanned for TypeFields {
    fn span(&self) -> Span {
        self.fields.span()
    }
}

impl Spanned for TypeField {
    fn span(&self) -> Span {
        Span::join(self.field_name.span(), self.field_ty.span())
    }
}

pub fn type_fields() -> impl Parser<Output = TypeFields> + Clone {
    punctuated(type_field(), optional_leading_whitespace(comma_token()))
    .map(|fields| {
        TypeFields { fields }
    })
}

pub fn type_field() -> impl Parser<Output = TypeField> + Clone {
    ident()
    .then_optional_whitespace()
    .then(colon_token())
    .then_optional_whitespace()
    .then(ty())
    .map(|((field_name, colon_token), field_ty)| {
        TypeField { field_name, colon_token, field_ty }
    })
}

impl TypeFields {
    pub fn iter(&self) -> impl Iterator<Item = &TypeField> {
        self.fields.iter()
    }
}

