use crate::priv_prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Ty {
    Path(PathType),
    Tuple(Parens<TyTupleDescriptor>),
    Array(SquareBrackets<TyArrayDescriptor>),
    Str {
        str_token: StrToken,
        length: SquareBrackets<Box<Expr>>,
    },
    Infer {
        underscore_token: UnderscoreToken,
    },
}

impl Spanned for Ty {
    fn span(&self) -> Span {
        match self {
            Ty::Path(path_type) => path_type.span(),
            Ty::Tuple(tuple_type) => tuple_type.span(),
            Ty::Array(array_type) => array_type.span(),
            Ty::Str { str_token, length } => Span::join(str_token.span(), length.span()),
            Ty::Infer { underscore_token } => underscore_token.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyTupleDescriptor {
    Nil,
    Cons {
        head: Box<Ty>,
        comma_token: CommaToken,
        tail: Punctuated<Ty, CommaToken>,
    },
}

impl TyTupleDescriptor {
    pub fn to_tys(self) -> Vec<Ty> {
        match self {
            TyTupleDescriptor::Nil => vec![],
            TyTupleDescriptor::Cons { head, tail, .. } => {
                let mut tys = vec![*head];
                for ty in tail.into_iter() {
                    tys.push(ty);
                }
                tys
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TyArrayDescriptor {
    pub ty: Box<Ty>,
    pub semicolon_token: SemicolonToken,
    pub length: Box<Expr>,
}

impl Parse for Ty {
    fn parse(parser: &mut Parser) -> ParseResult<Ty> {
        // parse parens carefully, such that only patterns of (ty) are parsed as ty,
        // and patterns of (ty,) are parsed as one-artity tuples with one element ty
        if let Some((mut parser, span)) = parser.enter_delimited(Delimiter::Parenthesis) {
            if let Some(consumed) = parser.check_empty() {
                return Ok(Ty::Tuple(Parens::new(
                    TyTupleDescriptor::Nil,
                    span,
                    consumed,
                )));
            }
            let head = parser.parse()?;
            if let Some(comma_token) = parser.take() {
                let (tail, consumed) = parser.parse_to_end()?;
                let tuple = TyTupleDescriptor::Cons {
                    head,
                    comma_token,
                    tail,
                };
                return Ok(Ty::Tuple(Parens::new(tuple, span, consumed)));
            }
            if parser.check_empty().is_some() {
                return Ok(*head);
            }
            return Err(parser
                .emit_error(ParseErrorKind::ExpectedCommaOrCloseParenInTupleOrParenExpression));
        }
        if let Some(descriptor) = SquareBrackets::try_parse(parser)? {
            return Ok(Ty::Array(descriptor));
        };
        if let Some(str_token) = parser.take() {
            let length = SquareBrackets::parse_all_inner(parser, |mut parser| {
                parser.emit_error(ParseErrorKind::UnexpectedTokenAfterStrLength)
            })?;
            return Ok(Ty::Str { str_token, length });
        }
        if let Some(underscore_token) = parser.take() {
            return Ok(Ty::Infer { underscore_token });
        }
        if parser.peek::<OpenAngleBracketToken>().is_some()
            || parser.peek::<DoubleColonToken>().is_some()
            || parser.peek::<Ident>().is_some()
        {
            let path_type = parser.parse()?;
            return Ok(Ty::Path(path_type));
        }
        Err(parser.emit_error(ParseErrorKind::ExpectedType))
    }
}

impl ParseToEnd for TyArrayDescriptor {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(TyArrayDescriptor, ParserConsumed<'a>)> {
        let ty = parser.parse()?;
        let semicolon_token = parser.parse()?;
        let length = parser.parse()?;
        let consumed = match parser.check_empty() {
            Some(consumed) => consumed,
            None => {
                return Err(parser.emit_error(ParseErrorKind::UnexpectedTokenAfterArrayTypeLength))
            }
        };
        let descriptor = TyArrayDescriptor {
            ty,
            semicolon_token,
            length,
        };
        Ok((descriptor, consumed))
    }
}
