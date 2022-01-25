use crate::priv_prelude::*;

pub struct TyTuple {
    pub elems: Parens<Punctuated<Ty, CommaToken>>,
}

impl Spanned for TyTuple {
    fn span(&self) -> Span {
        self.elems.span()
    }
}

pub fn ty_tuple() -> impl Parser<char, TyTuple, Error = Cheap<char, Span>> + Clone {
    //parens(leading_whitespace(tuple_descriptor(ty(), empty())).then_optional_whitespace())
    //.map(|descriptor| TyTuple { descriptor })
    parens(padded(punctuated(ty(), comma_token())))
    .map(|elems| TyTuple { elems })
}

