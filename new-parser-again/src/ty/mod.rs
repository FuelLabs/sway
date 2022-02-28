use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Ty {
    Path(PathType),
    Tuple(Parens<Punctuated<Ty, CommaToken>>),
    Array(SquareBrackets<TyArrayDescriptor>),
    Str {
        str_token: StrToken,
        length: SquareBrackets<Box<Expr>>,
    },
}

#[derive(Clone, Debug)]
pub struct TyArrayDescriptor {
    pub ty: Box<Ty>,
    pub semicolon_token: SemicolonToken,
    pub length: Box<Expr>,
}

impl Parse for Ty {
    fn parse(parser: &mut Parser) -> ParseResult<Ty> {
        if let Some(tuple) = Parens::try_parse(parser)? {
            return Ok(Ty::Tuple(tuple));
        };
        if let Some(descriptor) = SquareBrackets::try_parse(parser)? {
            return Ok(Ty::Array(descriptor));
        };
        if let Some(str_token) = parser.take() {
            let length = SquareBrackets::parse_all_inner(
                parser,
                |parser| parser.emit_error("unexpected tokens after str length"),
            )?;
            return Ok(Ty::Str { str_token, length })
        }
        if {
            parser.peek::<LessThanToken>().is_some() ||
            parser.peek::<DoubleColonToken>().is_some() ||
            parser.peek::<Ident>().is_some()
        } {
            let path_type = parser.parse()?;
            return Ok(Ty::Path(path_type));
        }
        Err(parser.emit_error("expected a type"))
    }
}

impl ParseToEnd for TyArrayDescriptor {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(TyArrayDescriptor, ParserConsumed<'a>)> {
        let ty = parser.parse()?;
        let semicolon_token = parser.parse()?;
        let length = parser.parse()?;
        let consumed = match parser.check_empty() {
            Some(consumed) => consumed,
            None => return Err(parser.emit_error("unexpected tokens after array length specifier")),
        };
        let descriptor = TyArrayDescriptor { ty, semicolon_token, length };
        Ok((descriptor, consumed))
    }
}

