use crate::priv_prelude::*;

#[derive(Debug, Clone)]
pub struct TyArray {
    pub descriptor: SquareBrackets<ArrayRepeatDescriptor<Ty>>,
}

impl Spanned for TyArray {
    fn span(&self) -> Span {
        self.descriptor.span()
    }
}

pub fn ty_array() -> impl Parser<Output = TyArray> + Clone {
    square_brackets(padded(
        array_repeat_descriptor(lazy(|| ty()))
    ))
    .map(|descriptor| TyArray { descriptor })
}

