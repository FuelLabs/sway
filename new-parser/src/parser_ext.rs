use crate::priv_prelude::*;

pub trait ParserExt<O>: Parser<char, O> + Sized {
    fn then_whitespace(self) -> ThenIgnore<Self, Padding<char, Self::Error>, O, ()> {
        self
        .then_ignore(whitespace())
    }

    fn then_optional_whitespace(self) -> ThenIgnore<Self, OrNot<Padding<char, Self::Error>>, O, Option<()>> {
        self
        .then_ignore(whitespace().or_not())
    }
}

impl<P, O> ParserExt<O> for P
where
    P: Parser<char, O> + Sized,
{}

pub fn leading_whitespace<P, T>(parser: P) -> impl Parser<char, T, Error = P::Error> + Clone
where
    P: Parser<char, T> + Clone,
{
    whitespace()
    .or_not()
    .then(parser)
    .map(|(_opt, value)| value)
}

pub fn padded<P, T>(parser: P) -> impl Parser<char, T, Error = P::Error> + Clone
where
    P: Parser<char, T> + Clone,
{
    leading_whitespace(parser.then_optional_whitespace())
}

pub fn never() -> impl Parser<char, Infallible, Error = Cheap<char, Span>> + Clone {
    empty().try_map(|(), span| Err(Cheap::expected_input_found(span, [], None)))
}
