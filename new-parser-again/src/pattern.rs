use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard {
        span: Span,
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

impl Parse for Pattern {
    fn parse(parser: &mut Parser) -> ParseResult<Pattern> {
        if let Some(mut_token) = parser.take() {
            let mutable = Some(mut_token);
            let name = parser.parse()?;
            return Ok(Pattern::Var { mutable, name });
        }
        if let Some(literal) = parser.peek() {
            return Ok(Pattern::Literal(literal));
        }
        if let Some(tuple) = Parens::try_parse(parser)? {
            return Ok(Pattern::Tuple(tuple));
        }

        let path = parser.parse::<PathExpr>()?;
        match path.try_into_ident() {
            Ok(name) => {
                if name.as_str() == "_" {
                    return Ok(Pattern::Wildcard { span: name.span() })
                }
                Ok(Pattern::Var { mutable: None, name })
            },
            Err(path) => {
                if let Some(args) = Parens::try_parse(parser)? {
                    return Ok(Pattern::Constructor { path, args });
                }
                if let Some(fields) = Braces::try_parse(parser)? {
                    return Ok(Pattern::Struct { path, fields });
                }
                Ok(Pattern::Constant(path))
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PatternStructField  {
    pub field_name: Ident,
    pub pattern_opt: Option<(ColonToken, Box<Pattern>)>,
}

impl Parse for PatternStructField {
    fn parse(parser: &mut Parser) -> ParseResult<PatternStructField> {
        let field_name = parser.parse()?;
        let pattern_opt = match parser.take() {
            Some(colon_token) => {
                let pattern = parser.parse()?;
                Some((colon_token, pattern))
            },
            None => None,
        };
        Ok(PatternStructField { field_name, pattern_opt })
    }
}

