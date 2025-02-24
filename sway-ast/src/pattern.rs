use sway_error::handler::ErrorEmitted;

use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub enum Pattern {
    Or {
        lhs: Box<Pattern>,
        pipe_token: PipeToken,
        rhs: Box<Pattern>,
    },
    Wildcard {
        underscore_token: UnderscoreToken,
    },
    /// A pattern made of a single ident, which could either be a variable or an enum variant
    AmbiguousSingleIdent(Ident),
    Var {
        reference: Option<RefToken>, // TODO: (REFERENCES) Implement `ref`, `mut`, and `ref mut` when implementing matching of references.
        mutable: Option<MutToken>,
        name: Ident,
    },
    Literal(Literal),
    Constant(PathExpr),
    Constructor {
        path: PathExpr,
        args: Parens<Punctuated<Pattern, CommaToken>>,
    },
    Struct {
        path: PathExpr,
        fields: Braces<Punctuated<PatternStructField, CommaToken>>,
    },
    Tuple(Parens<Punctuated<Pattern, CommaToken>>),
    // to handle parser recovery: Error represents an incomplete Constructor
    Error(Box<[Span]>, #[serde(skip_serializing)] ErrorEmitted),
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Or {
                lhs,
                pipe_token,
                rhs,
            } => Span::join(Span::join(lhs.span(), &pipe_token.span()), &rhs.span()),
            Pattern::Wildcard { underscore_token } => underscore_token.span(),
            Pattern::Var {
                reference,
                mutable,
                name,
            } => match (reference, mutable) {
                (Some(ref_token), Some(mut_token)) => Span::join(
                    Span::join(ref_token.span(), &mut_token.span()),
                    &name.span(),
                ),
                (Some(ref_token), None) => Span::join(ref_token.span(), &name.span()),
                (None, Some(mut_token)) => Span::join(mut_token.span(), &name.span()),
                (None, None) => name.span(),
            },
            Pattern::AmbiguousSingleIdent(ident) => ident.span(),
            Pattern::Literal(literal) => literal.span(),
            Pattern::Constant(path_expr) => path_expr.span(),
            Pattern::Constructor { path, args } => Span::join(path.span(), &args.span()),
            Pattern::Struct { path, fields } => Span::join(path.span(), &fields.span()),
            Pattern::Tuple(pat_tuple) => pat_tuple.span(),
            Pattern::Error(spans, _) => spans
                .iter()
                .cloned()
                .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                .unwrap(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum PatternStructField {
    Rest {
        token: DoubleDotToken,
    },
    Field {
        field_name: Ident,
        pattern_opt: Option<(ColonToken, Box<Pattern>)>,
    },
}

impl Spanned for PatternStructField {
    fn span(&self) -> Span {
        use PatternStructField::*;
        match &self {
            Rest { token } => token.span(),
            Field {
                field_name,
                pattern_opt,
            } => match pattern_opt {
                Some((_colon_token, pattern)) => Span::join(field_name.span(), &pattern.span()),
                None => field_name.span(),
            },
        }
    }
}
