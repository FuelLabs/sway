use crate::priv_prelude::*;

pub struct Path {
    leading_double_colon_opt: Option<DoubleColonToken>,
    prefix: Ident,
    suffix: Vec<(DoubleColonToken, Ident)>,
}

impl Spanned for Path {
    fn span(&self) -> Span {
        let first = match &self.leading_double_colon_opt {
            Some(double_colon_token) => double_colon_token.span(),
            None => self.prefix.span(),
        };
        let last = match self.suffix.last() {
            Some((_, ident)) => ident.span(),
            None => self.prefix.span(),
        };
        Span::join(first, last)
    }
}

pub fn path() -> impl Parser<char, Path, Error = Cheap<char, Span>> + Clone {
    double_colon_token()
    .then_optional_whitespace()
    .or_not()
    .then(ident())
    .then(
        leading_whitespace(
            double_colon_token()
            .then_optional_whitespace()
            .then(ident())
        )
        .repeated()
    )
    .map(|((leading_double_colon_opt, prefix), suffix)| {
        Path { leading_double_colon_opt, prefix, suffix }
    })
}
