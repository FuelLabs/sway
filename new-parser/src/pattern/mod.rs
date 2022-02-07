use crate::priv_prelude::*;

mod pattern_struct;
mod tuple;

pub use pattern_struct::*;
pub use tuple::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard {
        span: Span,
    },
    Var {
        mutable: Option<MutToken>,
        name: Ident,
    },
    StringLiteral(StringLiteral),
    IntLiteral(IntLiteral),
    Constructor {
        path: PathExpr,
        args: Parens<Punctuated<Pattern, CommaToken>>,
    },
    Struct(PatternStruct),
    Tuple(PatternTuple),
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span } => span.clone(),
            Pattern::Var { mutable, name } => {
                match mutable {
                    Some(mut_token) => Span::join(mut_token.span(), name.span()),
                    None => name.span(),
                }
            },
            Pattern::StringLiteral(string_literal) => string_literal.span(),
            Pattern::IntLiteral(int_literal) => int_literal.span(),
            Pattern::Constructor { path, args } => {
                Span::join(path.span(), args.span())
            },
            Pattern::Struct(pattern_struct) => pattern_struct.span(),
            Pattern::Tuple(pattern_tuple) => pattern_tuple.span(),
        }
    }
}

pub fn pattern() -> impl Parser<Output = Pattern> + Clone {
    let constructor = {
        path_expr()
        .then_optional_whitespace()
        .then(parens(optional_leading_whitespace(
            punctuated(
                lazy(|| pattern()).then_optional_whitespace(),
                comma_token().then_optional_whitespace(),
            )
        )))
        .map(|(path, args)| Pattern::Constructor { path, args })
    };
    let pattern_struct = {
        pattern_struct()
        .map(|pattern_struct| Pattern::Struct(pattern_struct))
    };
    let var = {
        mut_token()
        .then_whitespace()
        .optional()
        .then(ident())
        .map(|(mutable, name)| {
            Pattern::Var { mutable, name }
        })
    };
    let wildcard = {
        keyword("_")
        .map_with_span(|(), span| Pattern::Wildcard { span })
    };
    let string_literal = {
        string_literal()
        .map(|string_literal| {
            Pattern::StringLiteral(string_literal)
        })
    };
    let int_literal = {
        int_literal()
        .map(Pattern::IntLiteral)
    };
    let tuple = {
        pattern_tuple()
        .map(Pattern::Tuple)
    };

    or! {
        constructor,
        pattern_struct,
        var,
        wildcard,
        string_literal,
        int_literal,
        tuple,
    }
    .try_map_with_span(|pattern_opt: Option<Pattern>, span| {
        pattern_opt.ok_or_else(|| ParseError::ExpectedPattern { span })
    })
}

