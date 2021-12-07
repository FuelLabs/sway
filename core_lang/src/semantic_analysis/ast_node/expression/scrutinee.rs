use crate::{Ident, Literal, Span};

#[derive(Debug, Clone)]
pub enum TypedScrutinee<'sc> {
    Unit {
        span: Span<'sc>,
    },
    Literal {
        value: Literal<'sc>,
        span: Span<'sc>,
    },
    Variable {
        name: Ident<'sc>,
        span: Span<'sc>,
    },
    StructScrutinee {
        struct_name: Ident<'sc>,
        fields: Vec<TypedStructScrutineeField<'sc>>,
        span: Span<'sc>,
    },
}

#[derive(Debug, Clone)]
pub struct TypedStructScrutineeField<'sc> {
    pub scrutinee: TypedScrutinee<'sc>,
}
