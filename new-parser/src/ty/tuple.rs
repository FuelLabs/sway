use crate::priv_prelude::*;

#[derive(Debug, Clone)]
pub struct TyTuple {
    pub elems: Parens<Punctuated<Ty, CommaToken>>,
}

impl Spanned for TyTuple {
    fn span(&self) -> Span {
        self.elems.span()
    }
}

pub fn ty_tuple() -> impl Parser<Output = TyTuple> + Clone {
    parens(padded(punctuated(lazy(|| ty()), comma_token())))
    .map(|elems| TyTuple { elems })
}

