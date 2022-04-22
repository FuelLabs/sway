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

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { underscore_token } => underscore_token.span(),
            Pattern::Var { mutable, name } => match mutable {
                Some(mut_token) => Span::join(mut_token.span(), name.span().clone()),
                None => name.span().clone(),
            },
            Pattern::Literal(literal) => literal.span(),
            Pattern::Constant(path_expr) => path_expr.span(),
            Pattern::Constructor { path, args } => Span::join(path.span(), args.span()),
            Pattern::Struct { path, fields } => Span::join(path.span(), fields.span()),
            Pattern::Tuple(pat_tuple) => pat_tuple.span(),
        }
    }
}

impl Parse for Pattern {
    fn parse(parser: &mut Parser) -> ParseResult<Pattern> {
        if let Some(mut_token) = parser.take() {
            let mutable = Some(mut_token);
            let name = parser.parse()?;
            return Ok(Pattern::Var { mutable, name });
        }
        if let Some(literal) = parser.take() {
            return Ok(Pattern::Literal(literal));
        }
        if let Some(tuple) = Parens::try_parse(parser)? {
            return Ok(Pattern::Tuple(tuple));
        }
        if let Some(underscore_token) = parser.take() {
            return Ok(Pattern::Wildcard { underscore_token });
        }

        let path = parser.parse::<PathExpr>()?;
        if let Some(args) = Parens::try_parse(parser)? {
            return Ok(Pattern::Constructor { path, args });
        }
        if let Some(fields) = Braces::try_parse(parser)? {
            return Ok(Pattern::Struct { path, fields });
        }
        match path.try_into_ident() {
            Ok(name) => Ok(Pattern::Var {
                mutable: None,
                name,
            }),
            Err(path) => Ok(Pattern::Constant(path)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PatternStructField {
    pub field_name: Ident,
    pub pattern_opt: Option<(ColonToken, Box<Pattern>)>,
}

impl PatternStructField {
    pub fn span(&self) -> Span {
        match &self.pattern_opt {
            Some((_colon_token, pattern)) => {
                Span::join(self.field_name.span().clone(), pattern.span())
            }
            None => self.field_name.span().clone(),
        }
    }
}

impl Parse for PatternStructField {
    fn parse(parser: &mut Parser) -> ParseResult<PatternStructField> {
        let field_name = parser.parse()?;
        let pattern_opt = match parser.take() {
            Some(colon_token) => {
                let pattern = parser.parse()?;
                Some((colon_token, pattern))
            }
            None => None,
        };
        Ok(PatternStructField {
            field_name,
            pattern_opt,
        })
    }
}
