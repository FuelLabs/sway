use crate::priv_prelude::*;

pub struct ExprTuple {
    pub elems: Parens<Option<(Box<Expr>, CommaToken, Punctuated<Expr, CommaToken>)>>,
}

impl Spanned for ExprTuple {
    fn span(&self) -> Span {
        self.elems.span()
    }
}

pub fn expr_tuple() -> impl Parser<Output = ExprTuple> + Clone {
    parens(padded(
        lazy(|| expr())
        .then(comma_token())
        .then(punctuated(lazy(|| expr()), comma_token()))
        .optional()
    ))
    .map(|parens: Parens<Option<_>>| {
        let elems = parens.map(|elems_opt| {
            elems_opt.map(|((head, head_token), tail)| (Box::new(head), head_token, tail))
        });
        ExprTuple { elems }
    })
}

