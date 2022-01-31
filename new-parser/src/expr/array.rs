use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ExprArrayRepeat {
    pub descriptor: SquareBrackets<ArrayRepeatDescriptor<Expr>>,
}

impl Spanned for ExprArrayRepeat {
    fn span(&self) -> Span {
        self.descriptor.span()
    }
}

pub fn expr_array_repeat() -> impl Parser<Output = ExprArrayRepeat> + Clone {
    square_brackets(
        padded(
            array_repeat_descriptor(lazy(|| expr()))
        )
    )
    .map(|descriptor| ExprArrayRepeat { descriptor })
}

