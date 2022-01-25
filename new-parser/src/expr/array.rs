use crate::priv_prelude::*;

pub struct ExprArrayRepeat {
    pub descriptor: SquareBrackets<ArrayRepeatDescriptor<Expr>>,
}

impl Spanned for ExprArrayRepeat {
    fn span(&self) -> Span {
        self.descriptor.span()
    }
}

pub fn expr_array_repeat() -> impl Parser<char, ExprArrayRepeat, Error = Cheap<char, Span>> + Clone {
    square_brackets(leading_whitespace(array_repeat_descriptor(expr())).then_optional_whitespace())
    .map(|descriptor| ExprArrayRepeat { descriptor })
}

