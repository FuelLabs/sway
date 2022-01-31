use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Var(Ident),
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Var(ident) => ident.span(),
        }
    }
}

pub fn pattern() -> impl Parser<Output = Pattern> + Clone {
    ident()
    .map(Pattern::Var)
}

