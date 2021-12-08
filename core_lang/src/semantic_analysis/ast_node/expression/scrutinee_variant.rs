use crate::{Ident, Literal, Span};

use super::TypedScrutinee;

#[derive(Debug, Clone)]
pub(crate) enum TypedScrutineeVariant<'sc> {
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
        fields: Vec<TypedScrutinee<'sc>>,
        span: Span<'sc>,
    },
}
