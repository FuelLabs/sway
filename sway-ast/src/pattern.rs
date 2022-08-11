use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard {
        underscore_token: UnderscoreToken,
    },
    Var {
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
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { underscore_token } => underscore_token.span(),
            Pattern::Var { mutable, name } => match mutable {
                Some(mut_token) => Span::join(mut_token.span(), name.span()),
                None => name.span(),
            },
            Pattern::Literal(literal) => literal.span(),
            Pattern::Constant(path_expr) => path_expr.span(),
            Pattern::Constructor { path, args } => Span::join(path.span(), args.span()),
            Pattern::Struct { path, fields } => Span::join(path.span(), fields.span()),
            Pattern::Tuple(pat_tuple) => pat_tuple.span(),
        }
    }
}

#[derive(Clone, Debug)]
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
                Some((_colon_token, pattern)) => Span::join(field_name.span(), pattern.span()),
                None => field_name.span(),
            },
        }
    }
}
