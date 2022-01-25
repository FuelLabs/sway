use crate::priv_prelude::*;

pub struct ExprTuple {
    pub elems: Parens<ExprTupleElems>,
}

pub enum ExprTupleElems {
    Unit,
    Many {
        head: Box<Expr>,
        comma_token: CommaToken,
        tail: Punctuated<Expr, CommaToken>,
    },
}

impl Spanned for ExprTuple {
    fn span(&self) -> Span {
        self.elems.span()
    }
}

pub fn expr_tuple() -> impl Parser<char, ExprTuple, Error = Cheap<char, Span>> + Clone {
    parens(padded(expr_tuple_elems()))
    .map(|elems| ExprTuple { elems })
}

pub fn expr_tuple_elems() -> impl Parser<char, ExprTupleElems, Error = Cheap<char, Span>> + Clone {
    let unit = {
        empty()
        .map(|()| ExprTupleElems::Unit)
    };
    let many = {
        expr()
        .map(Box::new)
        .then_optional_whitespace()
        .then(comma_token())
        .then_optional_whitespace()
        .then(punctuated(expr(), comma_token()))
        .map(|((head, comma_token), tail)| ExprTupleElems::Many { head, comma_token, tail })
    };

    many
    .or(unit)
}

