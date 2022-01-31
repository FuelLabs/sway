use crate::priv_prelude::*;

pub enum Pattern {}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match *self {
        }
    }
}

pub fn pattern() -> impl Parser<Output = Pattern> + Clone {
    todo()
}

