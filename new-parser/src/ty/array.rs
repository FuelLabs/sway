use crate::priv_prelude::*;

pub struct TyArray {
    pub descriptor: SquareBrackets<ArrayRepeatDescriptor<Ty>>,
}

impl Spanned for TyArray {
    fn span(&self) -> Span {
        self.descriptor.span()
    }
}

pub fn ty_array() -> impl Parser<char, TyArray, Error = Cheap<char, Span>> + Clone {
    square_brackets(leading_whitespace(array_repeat_descriptor(ty())).then_optional_whitespace())
    .map(|descriptor| TyArray { descriptor })
}

