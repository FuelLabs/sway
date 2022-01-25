use crate::priv_prelude::*;

pub struct ArrayRepeatDescriptor<T> {
    pub elem: Box<T>,
    pub semicolon_token: SemicolonToken,
    pub len: IntLiteral,
}

pub fn array_repeat_descriptor<P, T>(
    elem_parser: P,
) -> impl Parser<char, ArrayRepeatDescriptor<T>, Error = Cheap<char, Span>> + Clone
where
    P: Parser<char, T, Error = Cheap<char, Span>> + Clone + 'static,
{
    elem_parser
    .then_optional_whitespace()
    .then(semicolon_token())
    .then_optional_whitespace()
    .then(int_literal())
    .map(|((elem, semicolon_token), len)| {
        ArrayRepeatDescriptor {
            elem: Box::new(elem),
            semicolon_token,
            len,
        }
    })
}

