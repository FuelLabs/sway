pub use crate::priv_prelude::*;

pub enum Pattern {}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match *self {
        }
    }
}

pub fn pattern() -> impl Parser<char, Pattern, Error = Cheap<char, Span>> + Clone {
    chumsky::primitive::todo()
}

