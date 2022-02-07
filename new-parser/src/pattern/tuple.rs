use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct PatternTuple {
    pub elems: Parens<Option<(Box<Pattern>, CommaToken, Punctuated<Pattern, CommaToken>)>>,
}

impl Spanned for PatternTuple {
    fn span(&self) -> Span {
        self.elems.span()
    }
}

pub fn pattern_tuple() -> impl Parser<Output = PatternTuple> + Clone {
    parens(
        optional_leading_whitespace(lazy(|| pattern()))
        .then_optional_whitespace()
        .then(comma_token())
        .then(punctuated(
            optional_leading_whitespace(lazy(|| pattern())),
            optional_leading_whitespace(comma_token()),
        ))
        .optional()
        .then_optional_whitespace()
    )
    .map(|parens: Parens<Option<_>>| {
        let elems = parens.map(|elems_opt| {
            elems_opt.map(|((head, head_token), tail)| (Box::new(head), head_token, tail))
        });
        PatternTuple { elems }
    })
}

